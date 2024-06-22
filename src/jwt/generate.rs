use std::env;

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use rand::RngCore;
use tonic::{metadata::MetadataMap, Status};
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

pub fn claims_from_metadata(metadata: &MetadataMap) -> Result<Claims, Status> {
    if !metadata.contains_key("authorization") {
        return Err(Status::unauthenticated("No authorization token was found"));
    }

    metadata
        .get("authorization")
        .unwrap()
        .to_str()
        .map_err(|_| Status::invalid_argument("Wrong authorization Bearer format"))?
        .parse()
}

pub fn generate_bytes(number: usize) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![0; number];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut buf);
    buf
}
