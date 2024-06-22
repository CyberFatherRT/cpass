use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Response,
};

use crate::{db::Db, error::CpassError, hashing::Argon, jwt::generate::create_token, AppState};

use super::models::{LoginRequest, User};

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
