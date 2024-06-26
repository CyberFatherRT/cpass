use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};

use super::models::{AddPasswordRequest, Password, UpdatePasswordRequest};
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

    let row = sqlx::query!(
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
            CpassError::NotFound("Password with that id not found".to_string())
        }
        _ => CpassError::DatabaseError(err),
    })?;

    let uuid = row.id;
    let name = row.name;
    let password = row.password;
    let salt = row.salt;
    let website = row.website;
    let username = row.username;
    let description = row.description;
    let tags = row.tags;

    let response: Json<Password> = Password {
        uuid,
        name,
        password,
        salt,
        website,
        username,
        description,
        tags,
    }
    .into();

    Ok((StatusCode::OK, response))
}

/// Get all passwords
#[utoipa::path(
    get,
    path = "/api/v1/pass/passwords",
    tag = "Password",
    responses(
        (status = 200, description = "Returns all passwords", body = Vec<Password>),
    )
)]
pub async fn get_passwords(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<Password>>), Response<String>> {
    let mut conn = state.pool.conn().await?;
    let owner_id = claims_from_headers(&headers)?.sub;

    let passwords = sqlx::query!(
        r#"
        SELECT id, password, name, salt, website, username, description, tags
        FROM passwords
        WHERE owner_id = $1
        "#,
        owner_id
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(CpassError::DatabaseError)?;

    let response: Json<Vec<Password>> = passwords
        .into_iter()
        .map(|x| Password {
            uuid: x.id,
            name: x.name,
            password: x.password,
            salt: x.salt,
            website: x.website,
            username: x.username,
            description: x.description,
            tags: x.tags,
        })
        .collect::<Vec<Password>>()
        .into();

    Ok((StatusCode::OK, response))
}

/// Add a password
#[utoipa::path(
    post,
    path = "/api/v1/pass/password",
    tag = "Password",
    request_body = AddPasswordResponse,
    responses(
        (status = 201, description = "Password created"),
    )
)]
pub async fn add_password(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(request): Json<AddPasswordRequest>,
) -> Result<StatusCode, Response<String>> {
    let mut conn = state.pool.conn().await?;
    let AddPasswordRequest {
        name,
        password,
        salt,
        website,
        username,
        description,
        tags,
    } = request;

    let owner_id = claims_from_headers(&headers)?.sub;

    let _ = sqlx::query!(
        r#"
        INSERT INTO passwords(owner_id, name, password, salt, website, username, description, tags)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        owner_id,
        name,
        password,
        salt,
        website,
        username,
        description,
        &tags
    )
    .execute(&mut *conn)
    .await
    .map_err(CpassError::DatabaseError)?;

    Ok(StatusCode::CREATED)
}

/// Update a password by id
#[utoipa::path(
    put,
    path = "/api/v1/pass/password/{id}",
    tag = "Password",
    request_body = AddPassword,
    responses(
        (status = 204, description = "Password updated"),
    )
)]
pub async fn update_password(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(pass_id): Path<uuid::Uuid>,
    Json(request): Json<UpdatePasswordRequest>,
) -> Result<StatusCode, Response<String>> {
    let mut conn = state.pool.conn().await?;
    let owner_id = claims_from_headers(&headers)?.sub;

    let UpdatePasswordRequest {
        name,
        password,
        salt,
        website,
        username,
        description,
        tags,
    } = request;

    let _ = sqlx::query!(
        r#"
        UPDATE passwords
        SET
            name = COALESCE($1, name),
            password = COALESCE($2, password),
            salt = COALESCE($3, salt),
            website = COALESCE($4, website),
            username = COALESCE($5, username),
            description = COALESCE($6, description),
            tags = COALESCE($7, tags)
        WHERE id = $8 AND owner_id = $9
        "#,
        name,
        password,
        salt,
        website,
        username,
        description,
        &tags.unwrap_or_default(),
        pass_id,
        owner_id
    )
    .execute(&mut *conn)
    .await
    .map_err(CpassError::DatabaseError);

    Ok(StatusCode::NO_CONTENT)
}

/// Delete a password by id
#[utoipa::path(
    delete,
    path = "/api/v1/pass/password/{id}",
    tag = "Password",
    responses(
        (status = 204, description = "Password deleted"),
    )
)]
pub async fn delete_password(
    headers: HeaderMap,
    Path(pass_id): Path<uuid::Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, Response<String>> {
    let mut conn = state.pool.conn().await?;
    let owner_id = claims_from_headers(&headers)?.sub;

    let _ = sqlx::query!(
        r#"
        DELETE FROM passwords
        WHERE id = $1 AND owner_id = $2
        "#,
        pass_id,
        owner_id
    )
    .execute(&mut *conn)
    .await
    .map_err(CpassError::DatabaseError)?;

    Ok(StatusCode::NO_CONTENT)
}
