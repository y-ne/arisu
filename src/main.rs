mod db;
mod error;
mod handlers;
mod models;

use axum::{
    Router,
    routing::{get, post},
};
use tower_sessions::{Expiry, SessionManagerLayer, cookie::time::Duration};
use tower_sessions_sqlx_store::PostgresStore;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let pool = db::connect().await;

    let session_store = PostgresStore::new(pool.clone());
    session_store
        .migrate()
        .await
        .expect("session migration failed");

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));

    let auth_routes = Router::new()
        .route("/register", post(handlers::auth::register))
        .route("/login", post(handlers::auth::login))
        .route("/me", get(handlers::auth::me))
        .route("/logout", post(handlers::auth::logout));

    let app = Router::new()
        .route("/", get(handlers::root::root))
        .nest("/auth", auth_routes)
        .layer(session_layer)
        .with_state(pool);

    let port = std::env::var("PORT").unwrap_or_else(|_| "4000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
