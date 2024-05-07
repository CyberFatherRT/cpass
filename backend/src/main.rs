mod routers;
mod structs;
mod utils;

use std::{env, sync::Arc};

use axum::{
    routing::{delete, post, put},
    Router,
};
use jsonwebtoken::{DecodingKey, EncodingKey};
use ring::rand::SystemRandom;
use routers::{create_user, delete_user, login, update_user};
use sqlx::PgPool;
use tokio::net::TcpListener;

pub struct AppState {
    db: PgPool,
    sprng: SystemRandom,
    jwt_encoding_key: EncodingKey,
    jwt_decoding_key: DecodingKey,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_url = env::var("DATABASE_URL")?;
    let jwt_secret = env::var("JWT_SECRET")?;
    let port = env::var("PORT")?;

    let conn = sqlx::PgPool::connect(&db_url).await?;

    let app_state = AppState {
        db: conn,
        sprng: SystemRandom::new(),
        jwt_encoding_key: EncodingKey::from_secret(jwt_secret.as_bytes()),
        jwt_decoding_key: DecodingKey::from_secret(jwt_secret.as_bytes()),
    };

    let app = Router::new()
        .route("/login", post(login))
        .route("/create_user", post(create_user))
        .route("/update_user", put(update_user))
        .route("/delete_user", delete(delete_user))
        .with_state(Arc::new(app_state));

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;
    Ok(())
}
