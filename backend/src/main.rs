mod db;
mod error;
mod hashing;
mod jwt;
mod proto;

use crate::proto::{
    auth::AuthService, auth_proto::auth_server::AuthServer, pass::PassService,
    pass_proto::pass_server::PassServer,
};
use axum::{http::StatusCode, routing::get, Router};
#[cfg(feature = "swagger")]
use routers::openapi::ApiDoc;
use sqlx::PgPool;
use tokio::{net::TcpListener, spawn, try_join};
use tonic::transport::Server;
use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level};
#[cfg(feature = "swagger")]
use utoipa::OpenApi;
#[cfg(feature = "swagger")]
use utoipa_swagger_ui::SwaggerUi;

mod routers;

#[derive(Clone)]
struct AppState {
    pub pool: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    if dotenvy::var("LOGGING").is_ok() {
        tracing_subscriber::fmt()
            .compact()
            .with_target(true)
            .with_max_level(Level::DEBUG)
            .init();
    }

    let db_url = dotenvy::var("DATABASE_URL")?;
    let http_addr = dotenvy::var("HTTP_ADDR")?;
    let grpc_addr = dotenvy::var("GRPC_ADDR")?.parse()?;
    let pool = PgPool::connect(&db_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    let grpc = Server::builder()
        .add_service(reflection)
        .add_service(AuthServer::new(AuthService::new(pool.clone())))
        .add_service(PassServer::new(PassService::new(pool.clone())))
        .serve(grpc_addr);

    let app_state = AppState { pool };

    let auth_app = routers::get_auth_service(app_state.clone());
    let pass_app = routers::get_pass_service(app_state.clone());

    let app = Router::new()
        .route("/api/healthcheck", get(StatusCode::OK))
        .nest("/api/v1/pass", pass_app)
        .nest("/api/v1/auth", auth_app)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    #[cfg(feature = "swagger")]
    let app = app
        .merge(SwaggerUi::new("/api/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()));

    info!("HTTP server listening on {}", http_addr);
    info!("gRPC server listening on {}", grpc_addr);

    let listener = TcpListener::bind(http_addr).await?;
    let http = axum::serve(listener, app);

    let _ = try_join!(spawn(async { http.await }), spawn(grpc));

    Ok(())
}
