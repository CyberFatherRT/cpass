use std::env;

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use ring::rand::{SecureRandom, SystemRandom};
use uuid::Uuid;

use crate::error::CpassError;

use super::models::Claims;

lazy_static! {
    static ref SECRET: Vec<u8> = match env::var("JWT_SECRET") {
        Ok(data) => data.as_bytes().to_vec(),
        Err(_) => generate_bytes(32),
    };
}

pub fn create_token(user_id: &Uuid) -> Result<String, CpassError> {
    let claims = Claims::new(user_id);

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(&SECRET),
    )
    .map_err(CpassError::InvalidToken)
}

pub fn validate_token(token: &str) -> Result<Claims, CpassError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(&SECRET),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
    .map_err(CpassError::InvalidToken)
}

pub fn generate_bytes(number: usize) -> Vec<u8> {
    let mut vec = Vec::with_capacity(number);
    let _ = SystemRandom::new().fill(&mut vec);
    vec
}
