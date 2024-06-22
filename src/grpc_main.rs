mod db;
mod error;
mod hashing;
mod jwt;
mod proto;

use std::env;

use crate::proto::{
    auth::AuthService, auth_proto::auth_server::AuthServer, pass::PassService,
    pass_proto::pass_server::PassServer,
};
use sqlx::PgPool;
use tonic::transport::Server;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .compact()
        .with_max_level(Level::DEBUG)
        .init();

    let db_url = env::var("DATABASE_URL")?;
    let grpc_addr = env::var("GRPC_ADDR")?.parse()?;
    let pool = PgPool::connect(&db_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    info!("gRPC server listening on {}", grpc_addr);

    let _ = Server::builder()
        .add_service(reflection)
        .add_service(AuthServer::new(AuthService::new(pool.clone())))
        .add_service(PassServer::new(PassService::new(pool.clone())))
        .serve(grpc_addr)
        .await;

    Ok(())
}
