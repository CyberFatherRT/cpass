pub mod auth;
mod models;
pub mod pass;

use std::sync::Arc;

use crate::AppState;
use axum::{
    routing::{post, put},
    Router,
};

use self::auth::{create_user, login, update_user};

pub fn get_auth_service(app_state: AppState) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/user", post(create_user))
        .route("/user", put(update_user))
        .with_state(Arc::new(app_state))
}

pub fn get_pass_service(app_state: AppState) -> Router {
    Router::new().with_state(app_state)
}
