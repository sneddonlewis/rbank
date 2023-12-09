use axum::{async_trait, Extension, Router};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{Method, StatusCode};
use axum::middleware::from_extractor;
use axum::routing::get;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use base64::Engine;
use base64::engine::general_purpose;
use chrono::{serde::ts_seconds, DateTime, Utc, Duration};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use openssl::rsa::Rsa;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/login", get(login))
        .route_layer(from_extractor::<AuthorizationMiddleware>())
        .route("/new", get(create_account));

    let listener = TcpListener::bind("localhost:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router)
        .await
        .unwrap();
}

const KID: &str = "quack";

struct AuthorizationMiddleware;

#[async_trait]
impl<S> FromRequestParts<S> for AuthorizationMiddleware where S: Send + Sync {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if parts.method == Method::OPTIONS {
            return Ok(Self);
        }
        let Ok(TypedHeader(Authorization(bearer))) = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
        else {
            eprintln!("Could not get Authorization header from request");
            return Err(StatusCode::UNAUTHORIZED);
        };

        let Some(jwks) = parts.extensions.get::<Jwks>()
        else {
            eprintln!("Could not find the JWK layer, did you forget to add it?");
            return Err(StatusCode::UNAUTHORIZED);
        };
        // try moving to inside if let
        match check_auth(bearer, &jwks) {
            Ok(auth) => {
                parts.extensions.insert(auth);
                Ok(Self)
            },
            Err(error) => {
                eprintln!("{error:?}");
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}

const ALGORITHM: Algorithm = jsonwebtoken::Algorithm::RS256;

#[derive(Clone, Debug)]
struct Jwks(Vec<Jwk>);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Jwk {
    encryption_algorithm: Algorithm,
    exponent: String,
    key_id: String,
    key_type: String,
    modulus_value: String,
    intended_use: String,
}

fn find_jwk<'a>(token: &'_ str, jwks: &'a [Jwk]) -> Option<&'a Jwk> {
    let headers = jsonwebtoken::decode_header(token).unwrap();
    jwks.iter().find(|jwk| {
        if let Some(key_id) = &headers.kid {
            &jwk.key_id == key_id
        } else {
            false
        }
    })
}

fn jwt_decode(token: &str, jwk: &Jwk) -> Result<Claims, String> {
    let validation = Validation::new(ALGORITHM);

    let decode_key = &DecodingKey::from_rsa_components(
        &jwk.modulus_value,
        &jwk.exponent,
    ).unwrap();
    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        decode_key,
        &validation,
    ).unwrap();

    Ok(decoded.claims)
}

fn jwt_encode(acc: &str, private_key: &[u8]) -> Result<String, ()> {
    let exp = Utc::now() + Duration::weeks(52);
    let claims = Claims {
        card_num: acc.to_string(),
        exp,
    };

    let mut header = jsonwebtoken::Header::new(ALGORITHM);
    header.kid = Some(KID.to_string());

    let encoding_key = &EncodingKey::from_rsa_pem(private_key).unwrap();
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        encoding_key,
    ).unwrap();
    Ok(token)
}

fn get_jwk(pub_key: &[u8]) -> Jwk {
    let rsa = Rsa::public_key_from_pem(pub_key).unwrap();
    Jwk {
        encryption_algorithm: ALGORITHM,
        key_id: KID.to_string(),
        key_type: "RSA".to_string(),
        intended_use: "sig".to_string(),
        modulus_value: general_purpose::URL_SAFE_NO_PAD.encode(rsa.n().to_vec()),
        exponent: general_purpose::URL_SAFE_NO_PAD.encode(rsa.e().to_vec()),
    }
}

fn check_auth(bearer: Bearer, jwks: &Jwks) -> Result<Authorized, String> {
    if let Some(jwk) = find_jwk(bearer.token(), &jwks.0) {
        let claims = jwt_decode(bearer.token(), jwk)?;
        Ok(Authorized(claims))
    } else {
        Err("JWK not found".to_string())
    }
}

#[derive(Debug, Clone)]
struct Authorized(pub Claims);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    card_num: String,
    #[serde(with = "ts_seconds")]
    exp: DateTime<Utc>,
}

async fn login(Extension(claims): Extension<Authorized>) -> String {
    let num = claims.0.card_num;
    format!("{num}")
}

async fn create_account() -> &'static str {
    let token = jwt_encode("4000001111111111", KEY_PRIV.as_bytes()).unwrap();
    "New Account Route"
}

const KEY_PUB: &str = r#"-----BEGIN PUBLIC KEY-----
MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAzdue26JnkXQ2x/f6TpEm
ysKEohmuPf40KoZT4x3zMO0sIas7UwBzjR9+PZ8wTFNhiOmf6IksT0xeI4MVNK+z
ewLWGsT7dRW/2iC956/WbhvkkZXlHK2qHpgx9t2DJ8fnPjUthvTrojPGEbsGvU+S
aGwP9jCfy6KPfkgFMNO2w+LwxnDErUVXE1Jr0jhNmkAFtewVW/8V7dFRYYPSOoip
rBEdtwVSnZb3uq+Zxk+EFeUvoSxcR7UXBdWnO/9/TEVZti+VzJwRTICpAvaSjXem
XYuVHmjxOmN0OhXr5EDJMbGa4OLXPW7j36fdcTZr1jOGv7QT+bcKcd+oSTVyS7ng
GtMRDWdLYQv+B+PZLX+3DED8WVIFn+vq3ci8en2L+JRqvUFIgjkaxjmk7SaRYlsu
+a2sl26ugYoFttdKEM6U1SOeDkKZlK14PaUf2az/vyX3drEHowrwtnvF6l7m9cho
g80RI4aCM29bGrb6+8/Z7WqOVhPqYHN0mF/SmaDbYo+4qakpzVWRdOnoCCG/URZU
DzEX1NdYwdBCk22yKA80/b7oWyI3BtIYpTMZWZf0HjaYb2zGAH3ysUWhhAG+pj/9
g0SWDty89gDqAr2sIxO0RzdiEXWk9UG6P+lY6uqpcVXBs27HwqcHVoevavMyUg8h
bnMtE4bIgcpUYWkZhMSzEA0CAwEAAQ==
-----END PUBLIC KEY-----"#;

const KEY_PRIV: &str = r#"
-----BEGIN RSA PRIVATE KEY-----
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
-----END RSA PRIVATE KEY-----"#;