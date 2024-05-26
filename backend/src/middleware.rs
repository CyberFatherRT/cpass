use std::sync::Arc;

use axum::{
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
    let exp = validate_token::<Claims>(&req.headers(), &state.jwt_decoding_key)?
        .claims
        .exp;

    if exp <= get_current_timestamp() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}
