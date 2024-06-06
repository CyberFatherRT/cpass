use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub username: String,
    pub password: String,
    pub password_hint: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangeUser {
    pub email: Option<String>,
    pub password: Option<String>,
    pub username: Option<String>,
}

#[derive(Deserialize)]
pub struct DeleteUser {
    pub email: String,
}
