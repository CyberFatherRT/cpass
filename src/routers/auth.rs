use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Response,
};

use crate::{db::Db, error::CpassError, hashing::Argon, jwt::generate::create_token, AppState};

use super::models::{CreateUserRequest, LoginRequest, User};

/// Login a user
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "User is logged in", body = User),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<(StatusCode, Json<User>), Response<String>> {
    let mut conn = state.pool.conn().await?;
    let LoginRequest { email, password } = request;

    let user = sqlx::query!(
        r#"
        SELECT * FROM users
        WHERE email = $1
        "#,
        email
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => CpassError::InvalidUsernameOrPassword,
        err => CpassError::DatabaseError(err),
    })?;

    match Argon::verify(password.as_bytes(), &user.password) {
        Ok(false) => return Err(CpassError::InvalidUsernameOrPassword.into()),
        Err(e) => return Err(e.into()),
        _ => {}
    }

    let token = create_token(&user.id)?;

    let response: Json<User> = User {
        email,
        token,
        username: user.username,
    }
    .into();

    Ok((StatusCode::OK, response))
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/api/v1/auth/user",
    tag = "Auth",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User is created", body = User),
        (status = 409, description = "User already exists"),
    )
)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), Response<String>> {
    let mut conn = state.pool.conn().await?;
    let CreateUserRequest {
        email,
        username,
        password,
    } = request;

    let hash = Argon::hash_password(password.as_bytes())?;

    let res = sqlx::query!(
        r#"
        INSERT INTO users(email, username, password)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        email,
        username,
        hash
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|err| match err {
        sqlx::Error::Database(err) if err.is_unique_violation() => {
            CpassError::UserAlreadyExists(email.to_string())
        }
        err => CpassError::DatabaseError(err),
    })?;

    let token = create_token(&res.id)?;

    let response: Json<User> = User {
        email,
        token,
        username,
    }
    .into();

    Ok((StatusCode::CREATED, response))
}
