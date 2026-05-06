mod db;
mod handlers;

use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let pool = db::connect().await;

    let app = Router::new()
        .route("/", get(handlers::root::root))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind port 3000");

    println!("listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.expect("server crashed");
}
