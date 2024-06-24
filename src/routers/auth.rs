use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};

use crate::{
    db::Db,
    error::CpassError,
    hashing::Argon,
    jwt::generate::{claims_from_headers, create_token},
    AppState,
};

use super::models::{CreateUserRequest, LoginRequest, UpdateUserRequest, User};

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

/// Update a user information
#[utoipa::path(
    put,
    path = "/api/v1/auth/user",
    tag = "Auth",
    request_body = UpdateUserRequest,
    responses(
        (status = 204, description = "User is updated"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn update_user(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<StatusCode, Response<String>> {
    let mut conn = state.pool.conn().await?;
    let UpdateUserRequest {
        email,
        username,
        password,
    } = request;

    let user_id = claims_from_headers(&headers)?.sub;

    let password = password
        .map(|pass| Argon::hash_password(pass.as_bytes()))
        .transpose()?;

    let _ = sqlx::query!(
        r#"
        UPDATE users
        SET
            email = COALESCE($1, email),
            username = COALESCE($2, username),
            password = COALESCE($3, password)
        WHERE id = $4
        "#,
        email,
        username,
        password,
        user_id
    )
    .execute(&mut *conn)
    .await
    .map_err(|err| match err {
        sqlx::Error::Database(err) if err.is_unique_violation() => {
            CpassError::UserAlreadyExists(email.unwrap())
        }
        err => CpassError::DatabaseError(err),
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete a user
#[utoipa::path(
    delete,
    path = "/api/v1/auth/user",
    tag = "Auth",
    responses(
        (status = 204, description = "User is deleted"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn delete_user(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, Response<String>> {
    let mut conn = state.pool.conn().await?;
    let user_id = claims_from_headers(&headers)?.sub;

    let _ = sqlx::query!(
        r#"
        DELETE FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .execute(&mut *conn)
    .await
    .map_err(CpassError::DatabaseError);

    Ok(StatusCode::NO_CONTENT)
}
