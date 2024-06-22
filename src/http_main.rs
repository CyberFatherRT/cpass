mod db;
mod error;
mod hashing;
mod jwt;

use std::env;

use axum::{http::StatusCode, routing::get, Router};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::Level;

mod routers;

#[derive(Clone)]
struct AppState {
    pub pool: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .compact()
        .with_max_level(Level::DEBUG)
        .init();

    let db_url = env::var("DATABASE_URL")?;
    let http_addr = env::var("HTTP_ADDR")?;
    let pool = PgPool::connect(&db_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app_state = AppState { pool };

    let auth_app = routers::get_auth_service(app_state.clone());
    let pass_app = routers::get_pass_service(app_state.clone());

    let app = Router::new()
        .route("/api/healthcheck", get(StatusCode::OK))
        .nest("/api/v1/pass", pass_app)
        .nest("/api/v1/auth", auth_app);

    let listener = TcpListener::bind(http_addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
