pub mod auth;
mod models;
pub mod openapi;
pub mod pass;

use std::sync::Arc;

use crate::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

use self::{
    auth::{create_user, delete_user, login, update_user},
    pass::{add_password, get_password, get_passwords},
};

pub fn get_auth_service(app_state: AppState) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/user", post(create_user))
        .route("/user", put(update_user))
        .route("/user", delete(delete_user))
        .with_state(Arc::new(app_state))
}

pub fn get_pass_service(app_state: AppState) -> Router {
    Router::new()
        .route("/passwords", get(get_passwords))
        .route("/password", post(add_password))
        .route("/password/:id", get(get_password))
        .with_state(Arc::new(app_state))
}
