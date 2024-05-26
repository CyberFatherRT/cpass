use std::sync::Arc;

use super::structs::{ChangeUser, CreateUser, DeleteUser, LoginUser};
use crate::{
    structs::{Claims, User},
    utils::{check_password, failed, hash_password, validate_token},
    AppState,
};
use axum::{
    body::Body,
    extract::{Json, State},
    http::{HeaderMap, Response, StatusCode},
    response::Redirect,
};
use jsonwebtoken::{encode, get_current_timestamp, Algorithm, Header};
use serde_json::json;
use uuid::Uuid;

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

    let _ = sqlx::query!(
        r"
        INSERT INTO users(email, username, password, password_hint)
        VALUES ($1, $2, $3, $4)
        ",
        email,
        username,
        password,
        password_hint
    )
    .execute(&mut *conn)
    .await
    .map_err(failed)?;

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::from("User is created."))
        .unwrap())
}

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
            .body(Body::from(
                json!({
                    "password_hint": user.password_hint
                })
                .to_string(),
            ))
            .unwrap());
    }

    let claims = Claims {
        id: user.id.to_string(),
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

pub async fn update_user(
    request: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChangeUser>,
) -> Result<Response<Body>, StatusCode> {
    let mut conn = state.db.acquire().await.map_err(failed)?;
    let token_data = validate_token::<Claims>(&request, &state.jwt_decoding_key)?;

    let user_id = token_data.claims.id;
    let user_id: Uuid = user_id.parse().map_err(failed)?;

    let ChangeUser {
        email,
        password,
        username,
    } = payload;

    let row = sqlx::query!(
        r"SELECT email, password, username FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(failed)?;

    let email = email.unwrap_or(row.email);
    let password = password.unwrap_or(row.password);
    let username = username.unwrap_or(row.username);

    let hashed_password = hash_password(&state.srng, &password).map_err(failed)?;

    let _ = sqlx::query!(
        r"UPDATE users
          SET email = $2, username = $3, password = $4
          WHERE id = $1
        ",
        user_id,
        email,
        username,
        hashed_password
    )
    .execute(&mut *conn)
    .await
    .map_err(failed)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("successfully update user"))
        .unwrap())
}

pub async fn delete_user(
    request: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DeleteUser>,
) -> Result<Redirect, StatusCode> {
    let mut conn = state.db.acquire().await.map_err(failed)?;
    let DeleteUser { email } = payload;
    let _ = validate_token::<Claims>(&request, &state.jwt_decoding_key)?;

    let _ = sqlx::query!(r"DELETE FROM users WHERE email = $1", email)
        .execute(&mut *conn)
        .await
        .map_err(failed)?;

    Ok(Redirect::permanent("/"))
}
