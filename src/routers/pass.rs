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
        SELECT id, password, name, website, username, description
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

    let response: Json<Password> = Password {
        uuid: row.id,
        name: row.name,
        password: row.password,
        website: row.website,
        username: row.username,
        description: row.description,
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
        SELECT id, password, name, website, username, description
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
            website: x.website,
            username: x.username,
            description: x.description,
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
        website,
        username,
        description,
    } = request;

    let owner_id = claims_from_headers(&headers)?.sub;

    let _ = sqlx::query!(
        r#"
        INSERT INTO passwords(owner_id, name, password, website, username, description)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        owner_id,
        name,
        password,
        website,
        username,
        description,
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
        website,
        username,
        description,
    } = request;

    let _ = sqlx::query!(
        r#"
        UPDATE passwords
        SET
            name = COALESCE($1, name),
            password = COALESCE($2, password),
            website = COALESCE($3, website),
            username = COALESCE($4, username),
            description = COALESCE($5, description)
        WHERE id = $6 AND owner_id = $7
        "#,
        name,
        password,
        website,
        username,
        description,
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
