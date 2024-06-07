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
    pass::structs::Tags,
    structs::Claims,
    utils::{failed, validate_token},
    AppState,
};

pub async fn add_tags(
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

pub async fn set_tags(
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

    let mut transaction = state.db.begin().await.map_err(failed)?;

    let res = sqlx::query!(
        r#"
        DELETE FROM tags
        WHERE password_id = $1
        "#,
        id
    )
    .execute(&mut *transaction)
    .await;

    if let Err(e) = res {
        let _ = transaction.rollback();
        return Err(failed(e));
    }

    let _ = sqlx::query!(
        r#"
        INSERT INTO tags (password_id, content)
        SELECT $1, unnest($2::text[])
        "#,
        id,
        &payload.tags
    )
    .execute(&mut *transaction)
    .await;

    if let Err(e) = res {
        let _ = transaction.rollback();
        return Err(failed(e));
    }

    let _ = transaction.commit().await.map_err(failed)?;

    Ok(StatusCode::NO_CONTENT)
}
