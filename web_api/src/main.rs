mod middleware;
mod auth;
mod view_models;

use axum::{Extension, Json, Router};
use axum::extract::FromRequestParts;
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};
use axum::middleware::from_extractor;
use axum::response::{IntoResponse};
use axum::routing::{get, post};
use base64::Engine;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use crate::auth::{encode_token, get_public_jwk, Jwks};

use crate::middleware::AuthorizationMiddleware;
use crate::view_models::{Account, AccountAuthView, AccountDetailView};

#[tokio::main]
async fn main() {
    let jwks = Jwks(vec![get_public_jwk()]);
    let router = Router::new()
        .route("/account", get(account))
        .route_layer(from_extractor::<AuthorizationMiddleware>())
        .route("/new", get(create_account))
        .route("/login", post(login))
        .layer(Extension(jwks));

    let listener = TcpListener::bind("localhost:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router)
        .await
        .unwrap();
}

async fn account(Extension(claims): Extension<auth::Authorized>) -> impl IntoResponse {
    let acc = Account::new();
    let num = claims.0.card_num;
    if acc.card_number == num {
        Json(AccountDetailView::from(acc)).into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED
        ).into_response()
    }
}

async fn login(Json(request): Json<AccountAuthView>)-> impl IntoResponse {
    // todo validate card number and pin
    let token = encode_token(request.card_number.to_string());
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        HeaderValue::try_from(token).unwrap()
    );
    (
        headers,
    )
}

async fn create_account() -> impl IntoResponse {
    let acc= Account::new();
    let view: AccountAuthView = AccountAuthView::from(acc);
    Json(view)
}
