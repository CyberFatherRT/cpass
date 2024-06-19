use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const JWT_ISSUER: &str = "authentication";
const JWT_EXPIRY_HOURS: i64 = 1;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub iss: String,
    pub sub: Uuid,
    pub iat: i64,
    pub exp: i64,
}

impl Claims {
    pub fn new(user_id: &Uuid) -> Self {
        let iat = Utc::now();
        let exp = iat + Duration::hours(JWT_EXPIRY_HOURS);

        Claims {
            iss: JWT_ISSUER.to_string(),
            sub: *user_id,
            iat: iat.timestamp(),
            exp: exp.timestamp(),
        }
    }
}
