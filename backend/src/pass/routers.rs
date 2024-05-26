use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Response, StatusCode},
    Json,
};
use serde_json::json;

use crate::{
    pass::structs::{AddPassword, AddTagsToPassword},
    structs::{Claims, Password},
    utils::{encrypt, failed, validate_token},
    AppState,
};

pub async fn get_all_passwords(
    request: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Response<Body>, StatusCode> {
    let mut conn = state.db.acquire().await.map_err(failed)?;

    let id = validate_token::<Claims>(&request, &state.jwt_decoding_key)?
        .claims
        .id;

    let id: uuid::Uuid = id.parse().map_err(failed)?;

    let _rows = sqlx::query_as!(Password, r"SELECT * FROM passwords WHERE owner_id = $1", id)
        .fetch_all(&mut *conn)
        .await
        .map_err(failed)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("hello, from pass"))
        .unwrap())
}

pub async fn add_password(
    request: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddPassword>,
) -> Result<Response<Body>, StatusCode> {
    let mut conn = state.db.acquire().await.map_err(failed)?;
    let Claims { id, .. } = validate_token::<Claims>(&request, &state.jwt_decoding_key)?.claims;
    let AddPassword {
        password,
        name,
        website,
        username,
        description,
        master_password,
    } = payload;

    let id: uuid::Uuid = id.parse().map_err(failed)?;

    let encrypted_password = encrypt(&state.srng, password.as_bytes(), master_password.as_bytes());
    let encrypted_password = encrypted_password.map_err(failed)?;

    let password_id = sqlx::query!(
        r#"
        INSERT INTO passwords(owner_id, password, name, website, username, description)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id;
        "#,
        id,
        encrypted_password,
        name,
        website,
        username,
        description
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(failed)?;

    let response = Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::from(
            json!({
                "password_id":  password_id.id.to_string()
            })
            .to_string(),
        ))
        .unwrap();

    Ok(response)
}

pub async fn add_tags_to_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddTagsToPassword>,
) -> Result<StatusCode, StatusCode> {
    let AddTagsToPassword { id, tags } = payload;

    let mut transaction = state.db.begin().await.map_err(failed)?;
    let id: uuid::Uuid = id.parse().map_err(failed)?;

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
                let _ = transaction.rollback();
                return Err(failed(e));
            }
        }
    }

    transaction.commit().await.map_err(failed)?;

    Ok(StatusCode::CREATED)
}
