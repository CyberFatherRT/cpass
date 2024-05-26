mod auth;
mod middleware;
mod pass;
mod structs;
mod utils;

use std::{env, sync::Arc};

use axum::{http::StatusCode, routing::get, Router};
use jsonwebtoken::{DecodingKey, EncodingKey};
use ring::rand::SystemRandom;
use sqlx::PgPool;
use tokio::net::TcpListener;

pub struct AppState {
    db: PgPool,
    srng: SystemRandom,
    jwt_encoding_key: EncodingKey,
    jwt_decoding_key: DecodingKey,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let port = env::var("PORT")?;
    let db_url = env::var("DATABASE_URL")?;
    let jwt_secret = env::var("JWT_SECRET")?;

    let conn = sqlx::PgPool::connect(&db_url).await?;

    let app_state = Arc::new(AppState {
        db: conn,
        srng: SystemRandom::new(),
        jwt_encoding_key: EncodingKey::from_secret(jwt_secret.as_bytes()),
        jwt_decoding_key: DecodingKey::from_secret(jwt_secret.as_bytes()),
    });

    let auth_app = auth::get_auth_service(app_state.clone());
    let pass_app = pass::get_pass_service(app_state.clone());
    let app = Router::new()
        .route("/api/healthcheck", get(StatusCode::OK))
        .nest("/api/v1/pass", pass_app)
        .nest("/api/v1/auth", auth_app);

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr).await?;

    println!("INFO: server listen on :{}", port);

    axum::serve(listener, app).await?;
    Ok(())
}
