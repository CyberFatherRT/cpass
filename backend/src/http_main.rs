mod db;
mod error;
mod hashing;
mod jwt;

use std::fs::read_to_string;

use axum::{http::StatusCode, routing::get, Router};
#[cfg(feature = "swagger")]
use routers::openapi::ApiDoc;
use sqlx::PgPool;
use tokio::net::TcpListener;
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

    let db_url = match dotenvy::var("DB_PASSWORD_FILE") {
        Ok(file) => format!(
            "postgres://postgres:{}@db:5432/cpass",
            read_to_string(file)?
        ),
        Err(_) => dotenvy::var("DATABASE_URL")?,
    };
    let addr = dotenvy::var("ADDR")?;
    let pool = PgPool::connect(&db_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app_state = AppState { pool };

    let auth_app = routers::get_auth_service(app_state.clone());
    let pass_app = routers::get_pass_service(app_state.clone());

    let app = Router::new()
        .route("/api/healthcheck", get(StatusCode::CREATED))
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

    info!("HTTP server listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
