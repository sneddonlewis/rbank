use axum::{async_trait, Extension, Router};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{Method, StatusCode};
use axum::middleware::from_extractor;
use axum::routing::get;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use chrono::{serde::ts_seconds, DateTime, Utc, Duration};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
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
    "New Account Route"
}
