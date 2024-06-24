use serde::{ser::SerializeStruct, Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct User {
    pub email: String,
    pub token: String,
    pub username: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(ToSchema)]
pub struct Password {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub encrypted_password: Vec<u8>,
    pub salt: Option<Vec<u8>>,
    pub website: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

impl Serialize for Password {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Password", 8)?;
        let salt = match &self.salt {
            Some(data) => data,
            None => &Vec::new(),
        };
        state.serialize_field("id", &self.uuid)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("password", &hex::encode(&self.encrypted_password))?;
        state.serialize_field("salt", &hex::encode(salt))?;
        state.serialize_field("website", &self.website)?;
        state.serialize_field("username", &self.username)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("tags", &self.tags)?;
        state.end()
    }
}
