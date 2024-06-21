use sqlx::FromRow;

#[derive(FromRow)]
pub struct Password {
    pub uuid: String,
    pub name: String,
    pub encrypted_password: Vec<u8>,
    pub salt: Option<Vec<u8>>,
    pub website: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>
}