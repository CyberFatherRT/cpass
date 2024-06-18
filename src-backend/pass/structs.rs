use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct AddPassword {
    pub password: String,
    pub name: String,
    pub website: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub master_password: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, ToSchema)]
pub struct AddPasswordResponse {
    pub password_id: Uuid,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Tags {
    pub tags: Vec<String>,
}
