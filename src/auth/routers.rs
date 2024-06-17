use std::sync::Arc;

use super::structs::{CreateUser, DeleteUser, LoginUser, UpdateUser};
use crate::{
    structs::{Claims, User},
    utils::{check_password, failed, hash_password, validate_token},
    AppState,
};
use axum::{
    body::Body,
    extract::{Json, State},
    http::{HeaderMap, Response, StatusCode},
};
use jsonwebtoken::{encode, get_current_timestamp, Algorithm, Header};
use serde_json::json;
use sqlx::error::ErrorKind;

/// Create a new user
#[utoipa::path(
    post,
    path = "/api/v1/auth/user",
    tag = "Auth",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User is created", body = AuthUserResponse),
        (status = 409, description = "User already exists"),
    )
)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUser>,
) -> Result<Response<Body>, StatusCode> {
    let mut conn = state.db.acquire().await.map_err(failed)?;

    let CreateUser {
        email,
        username,
        password,
        password_hint,
    } = payload;

    let password = hash_password(&state.srng, &password).map_err(failed)?;

    let res = sqlx::query!(
        r"
        INSERT INTO users(email, username, password, password_hint)
        VALUES ($1, $2, $3, $4)
        RETURNING id, email, username
        ",
        email,
        username,
        password,
        password_hint
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|err| {
        let err = err.into_database_error().unwrap();
        if err.kind() == ErrorKind::UniqueViolation {
            return StatusCode::CONFLICT;
        }
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let claims = Claims {
        id: res.id,
        exp: get_current_timestamp() + 60 * 60,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &state.jwt_encoding_key,
    )
    .map_err(failed)?;

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": res.email,
                "username": res.username,
                "token": token
            })
            .to_string(),
        ))
        .unwrap())
}

/// Login a user
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Auth",
    request_body = LoginUser,
    responses(
        (status = 200, description = "User is logged in", body = AuthUserResponse),
        (status = 401, description = "Unauthorized", body = LoginUnauthorized),
        (status = 404, description = "User not found"),
    )
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginUser>,
) -> Result<Response<Body>, StatusCode> {
    let mut conn = state.db.acquire().await.map_err(failed)?;

    let LoginUser { email, password } = payload;

    let user = sqlx::query_as!(User, r"SELECT * FROM users WHERE email = $1", email)
        .fetch_optional(&mut *conn)
        .await
        .map_err(failed)?;

    if user.is_none() {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("No user with that email and password"))
            .unwrap());
    }

    let user = user.unwrap();
    let are_you_sus = check_password(&user.password, password.as_bytes()).map_err(failed)?;

    if !are_you_sus {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "password_hint": user.password_hint
                })
                .to_string(),
            ))
            .unwrap());
    }

    let claims = Claims {
        id: user.id,
        exp: get_current_timestamp() + 60 * 60,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &state.jwt_encoding_key,
    )
    .map_err(failed)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": user.email,
                "username": user.username,
                "token": token
            })
            .to_string(),
        ))
        .unwrap())
}

/// Update a user information
#[utoipa::path(
    put,
    path = "/api/v1/auth/user",
    tag = "Auth",
    request_body = UpdateUser,
    responses(
        (status = 200, description = "User is updated"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn update_user(
    request: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateUser>,
) -> Result<Response<Body>, StatusCode> {
    let mut conn = state.db.acquire().await.map_err(failed)?;
    let user_id = validate_token::<Claims>(&request, &state.jwt_decoding_key)?
        .claims
        .id;

    let UpdateUser {
        email,
        password,
        username,
        password_hint,
    } = payload;

    let row = sqlx::query!(
        r"
        SELECT email, password, username, password_hint
        FROM users WHERE id = $1
        ",
        user_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(failed)?;

    let email = email.unwrap_or(row.email);
    let password = password.unwrap_or(row.password);
    let username = username.unwrap_or(row.username);
    let password_hint = password_hint.unwrap_or_default();

    let hashed_password = hash_password(&state.srng, &password).map_err(failed)?;

    let _ = sqlx::query!(
        r"
        UPDATE users
        SET email = $2, username = $3, password = $4, password_hint = $5
        WHERE id = $1
        ",
        user_id,
        email,
        username,
        hashed_password,
        password_hint
    )
    .execute(&mut *conn)
    .await
    .map_err(failed)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("successfully update user"))
        .unwrap())
}

/// Delete a user
#[utoipa::path(
    delete,
    path = "/api/v1/auth/user",
    tag = "Auth",
    request_body = DeleteUser,
    responses(
        (status = 204, description = "User is deleted"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn delete_user(
    request: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DeleteUser>,
) -> Result<StatusCode, StatusCode> {
    let mut conn = state.db.acquire().await.map_err(failed)?;
    let DeleteUser { email } = payload;
    let _ = validate_token::<Claims>(&request, &state.jwt_decoding_key)?;

    let _ = sqlx::query!(r"DELETE FROM users WHERE email = $1", email)
        .execute(&mut *conn)
        .await
        .map_err(failed)?;

    Ok(StatusCode::NO_CONTENT)
}
