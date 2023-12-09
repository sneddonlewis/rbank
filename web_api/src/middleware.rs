use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{Method, StatusCode};
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use crate::auth::{check_auth, Jwks};

pub struct AuthorizationMiddleware;

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
