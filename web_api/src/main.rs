mod middleware;
mod auth;

use axum::{Extension, Router};
use axum::extract::FromRequestParts;
use axum::http::{HeaderMap, Response};
use axum::middleware::from_extractor;
use axum::routing::get;
use base64::Engine;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use crate::auth::{encode_token, get_public_jwk, Jwks};

use crate::middleware::AuthorizationMiddleware;

#[tokio::main]
async fn main() {
    let jwks = Jwks(vec![get_public_jwk()]);
    let router = Router::new()
        .route("/login", get(login))
        .route_layer(from_extractor::<AuthorizationMiddleware>())
        .route("/new", get(create_account))
        .layer(Extension(jwks));

    let listener = TcpListener::bind("localhost:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router)
        .await
        .unwrap();
}
async fn login(Extension(claims): Extension<crate::auth::Authorized>) -> Response<String> {
    let token = encode_token("4000001111111111".to_string());
    let num = claims.0.card_num;
    let mut response = Response::new(num);

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", format!("Bearer {}", token).parse().unwrap());
    response.headers_mut().extend(headers);

    response
}

async fn create_account() -> Response<String> {
    let token = encode_token("4000001111111111".to_string());

    let mut response = Response::new("New Account Route".to_string());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", format!("Bearer {}", token).parse().unwrap());
    response.headers_mut().extend(headers);

    response
}
