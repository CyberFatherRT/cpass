use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Uuid};

#[derive(FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password: String,
    pub password_hint: Option<String>,
}

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct Password {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub password: String,
    pub name: String,
    pub website: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
}

// #[derive(FromRow)]
// pub struct Tags {
//     pub id: u64,
//     pub password_id: Uuid,
//     pub content: String,
// }

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub exp: u64,
}
