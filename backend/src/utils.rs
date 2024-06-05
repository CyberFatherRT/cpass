use std::fmt::Debug;

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    AeadCore, Aes256Gcm, Key,
};
use anyhow::{bail, Result};
use axum::http::{HeaderMap, StatusCode};
use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use ring::rand::{self, SecureRandom, SystemRandom};
use serde::de::DeserializeOwned;

pub fn failed<T: Debug>(err: T) -> StatusCode {
    println!("ERROR: {err:?}");
    StatusCode::INTERNAL_SERVER_ERROR
}

pub fn hash_password(srng: &SystemRandom, password: &str) -> Result<String> {
    let mut salt: [u8; 16] = [0; 16];
    let _ = srng.fill(&mut salt);
    let config = argon2::Config::owasp2();
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

pub fn encrypt(srng: &SystemRandom, password: &[u8], master_password: &[u8]) -> Result<String> {
    let mut salt: [u8; 16] = [0; 16];
    let _ = srng.fill(&mut salt);
    let config = argon2::Config::owasp2();
    let derivated_password = argon2::hash_raw(master_password, &salt, &config)?;

    let key = Key::<Aes256Gcm>::from_slice(&derivated_password);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let cipher = Aes256Gcm::new(key);

    let ciphered_data = cipher.encrypt(&nonce, password).map_err(|e| e.to_string());
    let ciphered_data = match ciphered_data {
        Ok(data) => data,
        Err(e) => bail!(e),
    };

    let mut encrypted_data: Vec<u8> = nonce.to_vec();
    encrypted_data.extend_from_slice(&ciphered_data);

    Ok(hex::encode(encrypted_data))
}

pub fn generate_bytes(number: usize) -> Vec<u8> {
    let mut vec = Vec::with_capacity(number);
    let _ = rand::SystemRandom::new().fill(&mut vec);
    vec
}
