mod password_routers;
mod structs;
mod tag_routers;

use std::sync::Arc;

use crate::{middleware::auth_middleware, AppState};
use axum::{
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Router,
};
use password_routers::{
    add_password, delete_password, get_all_passwords, get_password, update_password,
};
use tag_routers::{add_tags, delete_tags, set_tags};

pub fn get_pass_service(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/passwords", get(get_all_passwords))
        .route("/password", post(add_password))
        .route("/password/:id", get(update_password))
        .route("/password/:id", put(get_password))
        .route("/password/:id", delete(delete_password))
        .route("/tag/:id", post(add_tags))
        .route("/tag/:id", put(set_tags))
        .route("/tag/:id", delete(delete_tags))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}
