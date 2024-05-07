mod auth;

use std::env;

use axum::Router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = env::var("PORT")?;

    let auth_app = auth::get_auth_service().await?;
    let app = Router::new().nest("/api/v1/auth", auth_app);

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;
    Ok(())
}
