use std::env;

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use ring::rand::{SecureRandom, SystemRandom};
use uuid::Uuid;

use crate::error::CpassError;

use super::models::Claims;

pub fn create_token(user_id: &Uuid) -> Result<String, CpassError> {
    let secret = match env::var("JWT_SECRET") {
        Ok(data) => data.as_bytes().to_vec(),
        Err(_) => generate_bytes(32),
    };

    let claims = Claims::new(user_id);

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(&secret),
    )
    .map_err(CpassError::InvalidToken)
}

fn generate_bytes(number: usize) -> Vec<u8> {
    let mut vec = Vec::with_capacity(number);
    let _ = SystemRandom::new().fill(&mut vec);
    vec
}
