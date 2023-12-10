mod middleware;
mod auth;
mod view_models;
mod account_repo;

use std::sync::Arc;
use axum::{Extension, Json, Router};
use axum::extract::{State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::middleware::from_extractor;
use axum::response::{IntoResponse};
use axum::routing::{get, post};
use tokio::net::TcpListener;
use crate::account_repo::{AccountRepoImpl, DynAccountRepo};
use crate::auth::{encode_token, get_public_jwk, Jwks};

use crate::middleware::AuthorizationMiddleware;
use crate::view_models::{AccountAuthView, AccountDetailView};

#[tokio::main]
async fn main() {
    let account_repo = Arc::new(AccountRepoImpl) as DynAccountRepo;

    let jwks = Jwks(vec![get_public_jwk()]);

    let router = Router::new()
        .route("/account", get(account))
        .route_layer(from_extractor::<AuthorizationMiddleware>())
        .route("/new", get(create_account))
        .route("/login", post(login))
        .layer(Extension(jwks))
        .with_state(account_repo);

    let listener = TcpListener::bind("localhost:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router)
        .await
        .unwrap();
}


async fn account(
    Extension(claims): Extension<auth::Authorized>,
    State(account_repo): State<DynAccountRepo>,
) -> impl IntoResponse {
    let acc = account_repo.create()
        .await
        .unwrap();
    let num = claims.0.card_num;
    if acc.card_number == num {
        Json(AccountDetailView::from(acc)).into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED
        ).into_response()
    }
}

async fn login(
    State(account_repo): State<DynAccountRepo>,
    Json(request): Json<AccountAuthView>,
)-> impl IntoResponse {
    let acc = account_repo.find(request.card_number.clone())
        .await
        .unwrap();

    if acc.pin == request.pin {
        let token = encode_token(request.card_number.clone());
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            HeaderValue::try_from(token).unwrap()
        );
        (
            headers,
        ).into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED
        ).into_response()
    }
}

async fn create_account(State(account_repo): State<DynAccountRepo>) -> impl IntoResponse {
    let acc= account_repo.create()
        .await
        .unwrap();
    let view: AccountAuthView = AccountAuthView::from(acc);
    Json(view)
}
