mod auth;
mod middleware;
mod pass;
mod structs;
mod utils;

use std::{env, sync::Arc};

use axum::{http::StatusCode, middleware::from_fn, routing::get, Router};
use jsonwebtoken::{DecodingKey, EncodingKey};
use ring::rand::SystemRandom;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::{
    compression::{predicate::SizeAbove, CompressionLayer},
    trace::TraceLayer,
};
use tracing::{info, Level};
use utils::generate_bytes;

pub struct AppState {
    db: PgPool,
    srng: SystemRandom,
    jwt_encoding_key: EncodingKey,
    jwt_decoding_key: DecodingKey,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_max_level(Level::DEBUG)
        .init();

    let port = env::var("PORT")?;
    let db_url = env::var("DATABASE_URL")?;
    let jwt_secret = match env::var("JWT_SECRET") {
        Ok(data) => data.as_bytes().to_vec(),
        Err(_) => generate_bytes(32).to_vec(),
    };

    let conn = sqlx::PgPool::connect(&db_url).await?;

    let app_state = Arc::new(AppState {
        db: conn,
        srng: SystemRandom::new(),
        jwt_encoding_key: EncodingKey::from_secret(&jwt_secret),
        jwt_decoding_key: DecodingKey::from_secret(&jwt_secret),
    });

    let auth_app = auth::get_auth_service(app_state.clone());
    let pass_app = pass::get_pass_service(app_state.clone());
    let app = Router::new()
        .route("/api/healthcheck", get(StatusCode::OK))
        .nest("/api/v1/pass", pass_app)
        .nest("/api/v1/auth", auth_app)
        .layer(from_fn(middleware::error_middlweware))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new().compress_when(SizeAbove::default()));

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr).await?;

    info!("server listening on 0.0.0.0:{}", port);

    axum::serve(listener, app).await?;
    Ok(())
}
