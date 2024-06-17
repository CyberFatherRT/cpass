use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateUser {
    pub email: String,
    pub username: String,
    pub password: String,
    pub password_hint: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub password: Option<String>,
    pub username: Option<String>,
    pub password_hint: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct DeleteUser {
    pub email: String,
}

#[allow(dead_code)]
#[derive(Deserialize, ToSchema)]
pub struct LoginUnauthorized {
    pub password_hint: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, ToSchema)]
pub struct AuthUserResponse {
    pub email: String,
    pub username: String,
    pub token: String,
}
