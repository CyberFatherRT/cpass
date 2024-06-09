use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    Json,
};
use serde_json::json;

use crate::{
    structs::{Claims, Password},
    utils::{encrypt, failed, validate_token},
    AppState,
};

use super::structs::AddPassword;

/// Get all passwords
#[utoipa::path(
    get,
    path = "/api/v1/pass/passwords",
    responses(
        (status = 200, description = "Returns all passwords", body = [Vec<Password>]),
    )
)]
pub async fn get_all_passwords(
    request: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Password>>, StatusCode> {
    let mut conn = state.db.acquire().await.map_err(failed)?;

    let id = validate_token::<Claims>(&request, &state.jwt_decoding_key)?
        .claims
        .id;

    let rows = sqlx::query_as!(
        Password,
        r"
        SELECT *,
               ARRAY(SELECT content FROM tags WHERE password_id = id) as tags
        FROM passwords WHERE owner_id = $1
        ",
        id
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(failed)?;

    Ok(Json(rows))
}

/// Get a password by id
#[utoipa::path(
    get,
    path = "/api/v1/pass/password/{id}",
    responses(
        (status = 200, description = "Returns the password", body = [Password]),
        (status = 404, description = "Password not found"),
    )
)]
pub async fn get_password(
    request: HeaderMap,
    Path(id): Path<uuid::Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Password>, StatusCode> {
    let mut conn = state.db.acquire().await.map_err(failed)?;

    let user_id = validate_token::<Claims>(&request, &state.jwt_decoding_key)?
        .claims
        .id;

    let password = sqlx::query_as!(
        Password,
        r"
        SELECT *,
               ARRAY(SELECT content FROM tags WHERE tags.password_id = password_id) as tags
        FROM passwords WHERE owner_id = $1 and id = $2
        ",
        user_id,
        id,
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(failed)?;

    match password {
        Some(password) => Ok(Json(password)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Add a password
#[utoipa::path(
    post,
    path = "/api/v1/pass/password",
    request_body = AddPassword,
    responses(
        (status = 201, description = "Password created", body = [AddPasswordResponse]),
    )
)]
pub async fn add_password(
    request: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddPassword>,
) -> Result<Response<Body>, StatusCode> {
    let mut transaction = state.db.begin().await.map_err(failed)?;
    let id = validate_token::<Claims>(&request, &state.jwt_decoding_key)?
        .claims
        .id;

    let AddPassword {
        password,
        name,
        website,
        username,
        description,
        master_password,
        tags,
    } = payload;

    let (encrypted_password, salt) =
        encrypt(&state.srng, password.as_bytes(), master_password.as_bytes()).map_err(failed)?;

    let password_id = sqlx::query!(
        r#"
        INSERT INTO passwords(owner_id, password, salt, name, website, username, description)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id;
        "#,
        id,
        encrypted_password,
        salt,
        name,
        website,
        username,
        description
    )
    .fetch_one(&mut *transaction)
    .await;

    if let Err(e) = password_id {
        let _ = transaction.rollback().await;
        return Err(failed(e));
    }

    let password_id = password_id.unwrap();

    let res = sqlx::query!(
        r#"
        INSERT INTO tags (password_id, content)
        SELECT $1, unnest($2::text[])
        ON CONFLICT (password_id, content) DO NOTHING
        "#,
        password_id.id,
        &tags.unwrap_or_default()
    )
    .execute(&mut *transaction)
    .await;

    if let Err(e) = res {
        let _ = transaction.rollback().await;
        return Err(failed(e));
    }

    let _ = transaction.commit().await;

    let response = Response::builder()
        .status(StatusCode::CREATED)
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "password_id": password_id.id
            })
            .to_string(),
        ))
        .unwrap();

    Ok(response)
}

/// Update a password by id
#[utoipa::path(
    put,
    path = "/api/v1/pass/password/{id}",
    request_body = AddPassword,
    responses(
        (status = 204, description = "Password updated"),
        (status = 404, description = "Password not found"),
    )
)]
pub async fn update_password(
    request: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<AddPassword>,
) -> Result<StatusCode, StatusCode> {
    let mut conn = state.db.acquire().await.map_err(failed)?;

    let user_id = validate_token::<Claims>(&request, &state.jwt_decoding_key)?
        .claims
        .id;

    let (encrypted_password, salt) = encrypt(
        &state.srng,
        payload.password.as_bytes(),
        payload.master_password.as_bytes(),
    )
    .map_err(failed)?;

    let res = sqlx::query!(
        r#"
        UPDATE passwords
        SET password = $1,
            salt = $2,
            name = $3,
            website = $4,
            username = $5,
            description = $6
        WHERE owner_id = $7 AND id = $8
        RETURNING id
        "#,
        encrypted_password,
        salt,
        payload.name,
        payload.website,
        payload.username,
        payload.description,
        user_id,
        id
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(failed)?;

    match res {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Delete a password by id
#[utoipa::path(
    delete,
    path = "/api/v1/pass/password/{id}",
    responses(
        (status = 204, description = "Password deleted"),
        (status = 404, description = "Password not found"),
    )
)]
pub async fn delete_password(
    request: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, StatusCode> {
    let mut conn = state.db.acquire().await.map_err(failed)?;

    let user_id = validate_token::<Claims>(&request, &state.jwt_decoding_key)?
        .claims
        .id;

    let res = sqlx::query!(
        r#"
        DELETE FROM passwords
        WHERE owner_id = $1 AND id = $2
        RETURNING id
        "#,
        user_id,
        id
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(failed)?;

    match res {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(StatusCode::NOT_FOUND),
    }
}
