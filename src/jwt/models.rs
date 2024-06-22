use std::str::FromStr;

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tonic::Status;
use uuid::Uuid;

use super::generate::validate_token;

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

impl FromStr for Claims {
    type Err = tonic::Status;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let claims: Vec<_> = s.split(' ').collect();
        let token = claims.get(1).ok_or(Status::invalid_argument(
            "Wrong authorization Bearer format",
        ))?;
        Ok(validate_token(token)?)
    }
}
