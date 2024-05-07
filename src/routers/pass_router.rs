use axum::routing::{delete, get, post, put, Router};

async fn index() -> String {
    "Hello, World".to_string()
}

pub fn get_pass_router() -> Router {
    Router::new()
        .route("/get_passwords", get(index))
        .route("/add_password", post(index))
        .route("/update_password", put(index))
        .route("/delete_password", delete(index))
        .route("/add_tag", post(index))
        .route("/delete_tag", delete(index))
}
