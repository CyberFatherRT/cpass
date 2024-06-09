pub mod routers;
pub mod structs;

use std::sync::Arc;

use axum::{
    routing::{delete, post, put},
    Router,
};
use routers::{create_user, delete_user, login, update_user};

use crate::AppState;

pub fn get_auth_service(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/user", post(create_user))
        .route("/user", put(update_user))
        .route("/user", delete(delete_user))
        .with_state(state)
}
