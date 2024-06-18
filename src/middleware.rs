use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::get_current_timestamp;

use crate::{structs::Claims, utils::validate_token, AppState};

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let exp = validate_token::<Claims>(req.headers(), &state.jwt_decoding_key)?
        .claims
        .exp;

    if exp <= get_current_timestamp() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}

pub async fn error_middlweware(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let result = next.run(req).await;
    let headers = result.headers();
    if !headers.contains_key("X-Log-Me") {
        return Ok(result);
    }
    // TODO: Log the request
    Ok(result)
}
