use serde::{ser::SerializeStruct, Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Uuid};

#[derive(FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password: String,
    pub password_hint: Option<String>,
}

#[derive(FromRow, Debug, Deserialize)]
pub struct Password {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub password: Vec<u8>,
    pub salt: Vec<u8>,
    pub name: String,
    pub website: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: Uuid,
    pub exp: u64,
}

impl Serialize for Password {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Password", 9)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("owner_id", &self.owner_id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("password", &hex::encode(&self.password))?;
        state.serialize_field("salt", &hex::encode(&self.salt))?;
        state.serialize_field("website", &self.website)?;
        state.serialize_field("username", &self.username)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("tags", &self.tags)?;
        state.end()
    }
}
