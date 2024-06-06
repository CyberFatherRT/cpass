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
    pass::structs::{AddPassword, AddTagsToPassword},
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

    for tag in tags.unwrap_or_default() {
        let res = sqlx::query!(
            r#"
            INSERT INTO tags (password_id, content)
            VALUES ($1, $2)
            "#,
            password_id.id,
            tag
        )
        .execute(&mut *transaction)
        .await;

        match res {
            Ok(_) => continue,
            Err(e) => {
                let _ = transaction.rollback();
                return Err(failed(e));
            }
        }
    }

    transaction.commit().await.map_err(failed)?;

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
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<AddTagsToPassword>,
) -> Result<StatusCode, StatusCode> {
    let AddTagsToPassword { tags } = payload;

    let mut transaction = state.db.begin().await.map_err(failed)?;

    for tag in tags {
        let res = sqlx::query!(
            r#"
            INSERT INTO tags (password_id, content)
            VALUES ($1, $2)
            "#,
            id,
            tag
        )
        .execute(&mut *transaction)
        .await;

        match res {
            Ok(_) => continue,
            Err(e) => {
                let e = e.into_database_error().unwrap();
                if e.kind() == ErrorKind::UniqueViolation {
                    continue;
                } else {
                    let _ = transaction.rollback();
                    return Err(failed(e));
                }
            }
        }
    }

    transaction.commit().await.map_err(failed)?;

    Ok(StatusCode::CREATED)
}
