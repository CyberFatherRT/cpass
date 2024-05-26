pub mod routers;
mod structs;

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
        .route("/create_user", post(create_user))
        .route("/update_user", put(update_user))
        .route("/delete_user", delete(delete_user))
        .with_state(state)
}
