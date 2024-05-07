mod routers;
mod db;

use std::env;

use axum::routing::{get, Router};
use dotenv::dotenv;
use routers::{auth_router::get_auth_router, pass_router::get_pass_router};
use tokio::net::TcpListener;

async fn index() -> String {
    "Hello, World!".to_string()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let app = Router::new()
        .route("/", get(index))
        .nest_service("/api/v1/pass", get_pass_router())
        .nest_service("/api/v1/auth", get_auth_router());

    let addr = format!("0.0.0.0:{}", env::var("addr")?);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
