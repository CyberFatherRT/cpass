use axum::routing::{get, Router};

async fn index() -> String {
    "Hello, World".to_string()
}

pub fn get_auth_router() -> Router {
    Router::new()
        .route("/delete_user", get(index))
        .route("/create_user", get(index))
        .route("/login", get(index))
}
