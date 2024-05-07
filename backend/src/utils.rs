use anyhow::Result;
use axum::http::{HeaderMap, StatusCode};
use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use ring::rand::{SecureRandom, SystemRandom};
use serde::de::DeserializeOwned;

pub fn failed<T>(_: T) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

pub fn hash_password(sprng: &SystemRandom, password: &str) -> Result<String> {
    let mut salt: [u8; 16] = [0; 16];
    let _ = sprng.fill(&mut salt);
    let config = argon2::Config::owasp5();
    Ok(argon2::hash_encoded(password.as_bytes(), &salt, &config)?)
}

pub fn check_password(encoded: &str, pass: &[u8]) -> Result<bool> {
    Ok(argon2::verify_encoded(encoded, pass)?)
}

pub fn validate_token<T>(
    token: &HeaderMap,
    jwt_decoding_key: &DecodingKey,
) -> Result<TokenData<T>, StatusCode>
where
    T: DeserializeOwned,
{
    let jwt_token = token
        .get("X-Auth-Token")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(failed)?;

    let token_data = decode::<T>(
        jwt_token,
        jwt_decoding_key,
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(token_data)
}
