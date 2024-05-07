use axum::routing::{get, post, Router};

async fn index() -> String {
    "Hello, World".to_string()
}

async fn create_user() {}

pub fn get_auth_router() -> Router {
    Router::new()
        .route("/delete_user", get(index))
        .route("/create_user", post(create_user))
        .route("/login", get(index))
}
