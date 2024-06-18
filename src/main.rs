mod auth;
mod middleware;
mod openapi;
mod pass;
mod proto;
mod structs;
mod utils;

use std::{env, sync::Arc};

use crate::proto::{
    auth::AuthService, auth_proto::auth_server::AuthServer, pass::PassService,
    pass_proto::pass_server::PassServer, tag::TagService, tag_proto::tag_server::TagServer,
};
use axum::{http::StatusCode, middleware::from_fn, routing::get, Router};
use jsonwebtoken::{DecodingKey, EncodingKey};
use openapi::ApiDoc;
use ring::rand::SystemRandom;
use sqlx::PgPool;
use tokio::{net::TcpListener, try_join};
use tonic::transport::Server;
use tower_http::{
    compression::{predicate::SizeAbove, CompressionLayer},
    trace::TraceLayer,
};
use tracing::{info, Level};
use utils::generate_bytes;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub struct AppState {
    db: PgPool,
    srng: SystemRandom,
    jwt_encoding_key: EncodingKey,
    jwt_decoding_key: DecodingKey,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_max_level(Level::DEBUG)
        .init();

    let db_url = env::var("DATABASE_URL")?;
    let http_addr = env::var("HTTP_ADDR")?;
    let grpc_addr = env::var("GRPC_ADDR")?;
    let jwt_secret = match env::var("JWT_SECRET") {
        Ok(data) => data.as_bytes().to_vec(),
        Err(_) => generate_bytes(32).to_vec(),
    };

    let conn = PgPool::connect(&db_url).await?;
    sqlx::migrate!("./migrations").run(&conn).await?;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    let grpc_server = Server::builder()
        .add_service(reflection)
        .add_service(AuthServer::new(AuthService::default()))
        .add_service(PassServer::new(PassService::default()))
        .add_service(TagServer::new(TagService::default()))
        .serve(grpc_addr.parse()?);

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
        .merge(SwaggerUi::new("/api/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .layer(from_fn(middleware::error_middlweware))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new().compress_when(SizeAbove::default()));

    let listener = TcpListener::bind(&http_addr).await?;
    let http_server = axum::serve(listener, app);

    info!("gRPC server listening on {}", grpc_addr);
    info!("HTTP server listening on {}", http_addr);

    try_join!(
        tokio::spawn(async { http_server.await }),
        tokio::spawn(grpc_server)
    );

    Ok(())
}
