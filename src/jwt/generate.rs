use std::env;

use axum::http::HeaderMap;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use rand::RngCore;
use tonic::metadata::MetadataMap;
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

pub fn claims_from_headers(headers: &impl Map) -> Result<Claims, CpassError> {
    if !headers.contains_key("authorization") {
        return Err(CpassError::InvalidRequest(
            "No authorization token was found".to_string(),
        ));
    }

    headers
        .get("authorization")
        .map_err(|_| CpassError::InvalidRequest("Wrong authorization Bearer format".to_string()))?
        .unwrap()
        .parse()
}

pub fn generate_bytes(number: usize) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![0; number];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut buf);
    buf
}

trait Map {
    fn get(&self, key: &str) -> anyhow::Result<Option<&str>>;
    fn contains_key(&self, key: &str) -> bool;
}

impl Map for HeaderMap {
    fn get(&self, key: &str) -> anyhow::Result<Option<&str>> {
        Ok(self.get(key).map(|x| x.to_str()).transpose()?)
    }

    fn contains_key(&self, key: &str) -> bool {
        self.contains_key(key)
    }
}

impl Map for MetadataMap {
    fn get(&self, key: &str) -> anyhow::Result<Option<&str>> {
        Ok(self.get(key).map(|x| x.to_str()).transpose()?)
    }

    fn contains_key(&self, key: &str) -> bool {
        self.contains_key(key)
    }
}
