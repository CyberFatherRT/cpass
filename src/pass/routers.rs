use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Response, StatusCode},
    Json,
};
use serde_json::json;
use sqlx::error::ErrorKind;

use crate::{
    pass::structs::{AddPassword, Tags},
    structs::{Claims, Password},
    utils::{encrypt, failed, validate_token},
    AppState,
};

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
        let _ = transaction.rollback();
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
        let _ = transaction.rollback();
        return Err(failed(e));
    }

    let response = Response::builder()
        .status(StatusCode::CREATED)
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "password_id":  password_id.id
            })
            .to_string(),
        ))
        .unwrap();

    Ok(response)
}

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
        "#,
        user_id,
        id
    )
    .execute(&mut *conn)
    .await;

    match res {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn add_tags_to_password(
    request: HeaderMap,
    Path(id): Path<uuid::Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Tags>,
) -> Result<Response<Body>, StatusCode> {
    let Tags { tags } = payload;

    let mut conn = state.db.acquire().await.map_err(failed)?;
    let user_id = validate_token::<Claims>(&request, &state.jwt_decoding_key)?
        .claims
        .id;

    let is_owner = sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM passwords WHERE id = $1 and owner_id = $2) as is_owner
        "#,
        id,
        user_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(failed)?;

    if is_owner.is_owner.unwrap_or_default() == false {
        return Err(StatusCode::FORBIDDEN);
    }

    let tags = sqlx::query!(
        r#"
        INSERT INTO tags (password_id, content)
        SELECT $1, unnest($2::text[])
        ON CONFLICT (password_id, content) DO NOTHING
        RETURNING content
        "#,
        id,
        &tags
    )
    .fetch_all(&mut *conn)
    .await;

    if let Err(e) = tags {
        let e = e.into_database_error().unwrap();
        if e.kind() == ErrorKind::UniqueViolation {
            return Err(StatusCode::CONFLICT);
        }
        return Err(failed(e));
    };

    let tags = tags
        .unwrap()
        .iter()
        .map(|tag| tag.content.clone())
        .collect::<Vec<String>>();

    if tags.is_empty() {
        let response = Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();
        return Ok(response);
    }

    let response = Response::builder()
        .status(StatusCode::CREATED)
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "added_tags": tags
            })
            .to_string(),
        ))
        .unwrap();

    Ok(response)
}

pub async fn delete_tags(
    request: HeaderMap,
    Path(id): Path<uuid::Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Tags>,
) -> Result<StatusCode, StatusCode> {
    let mut conn = state.db.acquire().await.map_err(failed)?;

    let user_id = validate_token::<Claims>(&request, &state.jwt_decoding_key)?
        .claims
        .id;

    let is_owner = sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM passwords WHERE id = $1 and owner_id = $2) as is_owner
        "#,
        id,
        user_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(failed)?;

    if is_owner.is_owner.unwrap_or_default() == false {
        return Err(StatusCode::FORBIDDEN);
    }

    let _ = sqlx::query!(
        r#"
        DELETE FROM tags
        WHERE password_id = $1 AND content = ANY($2)
        "#,
        id,
        &payload.tags
    )
    .execute(&mut *conn)
    .await
    .map_err(failed)?;

    todo!()
}
