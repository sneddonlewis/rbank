use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/login", get(login))
        .route("/new", get(create_account));

    let listener = TcpListener::bind("localhost:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router)
        .await
        .unwrap();
}

async fn login() -> &'static str {
    "Log In Route"
}

async fn create_account() -> &'static str {
    "New Account Route"
}
