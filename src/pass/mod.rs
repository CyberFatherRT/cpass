mod routers;
mod structs;

use std::sync::Arc;

use crate::{middleware::auth_middleware, AppState};
use axum::{
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Router,
};
use routers::{
    add_password, add_tags_to_password, delete_password, get_all_passwords, get_password,
};

pub fn get_pass_service(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/passwords", get(get_all_passwords))
        .route("/password", post(add_password))
        .route("/password/:id", put(add_tags_to_password))
        .route("/password/:id", get(get_password))
        .route("/password/:id", delete(delete_password))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}
