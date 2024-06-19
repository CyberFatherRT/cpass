use crate::{error::CpassError, jwt::generate::generate_bytes};

pub struct Argon;

impl Argon {
    pub fn hash_password(password: &[u8]) -> Result<String, CpassError> {
        let config = argon2::Config::rfc9106();
        let salt = generate_bytes(16);

        argon2::hash_encoded(password, &salt, &config).map_err(CpassError::HashingError)
    }

    pub fn verify(password: &[u8], hash: &str) -> Result<bool, CpassError> {
        argon2::verify_encoded(hash, password).map_err(CpassError::HashingError)
    }
}
