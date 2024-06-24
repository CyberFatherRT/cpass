use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};

use super::models::Password;
use crate::{db::Db, error::CpassError, jwt::generate::claims_from_headers, AppState};

/// Get a password by id
#[utoipa::path(
    get,
    path = "/api/v1/pass/password/{id}",
    tag = "Password",
    responses(
        (status = 200, description = "Returns the password", body = Password),
        (status = 404, description = "Password not found"),
    )
)]
pub async fn get_password(
    headers: HeaderMap,
    Path(pass_id): Path<uuid::Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Password>), Response<String>> {
    let mut conn = state.pool.conn().await?;
    let owner_id = claims_from_headers(&headers)?.sub;

    let password = sqlx::query!(
        r#"
        SELECT id, password, name, salt, website, username, description, tags
        FROM passwords
        WHERE id = $1 AND owner_id = $2
        "#,
        pass_id,
        owner_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => {
            CpassError::InvalidRequest("Password with that id not found".to_string())
        }
        _ => CpassError::DatabaseError(err),
    })?;

    let uuid = password.id;
    let name = password.name;
    let encrypted_password = password.password;
    let salt = password.salt;
    let website = password.website;
    let username = password.username;
    let description = password.description;
    let tags = password.tags;

    let response: Json<Password> = Password {
        uuid,
        name,
        encrypted_password,
        salt,
        website,
        username,
        description,
        tags,
    }
    .into();

    Ok((StatusCode::OK, response))
}
