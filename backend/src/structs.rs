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

#[derive(FromRow, Debug)]
pub struct Password {
    pub id: Uuid,
    #[allow(dead_code)]
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

impl Serialize for Password {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Password", 6)?;
        state.serialize_field("id", &self.id.to_string())?;
        state.serialize_field("password", &self.password)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("website", &self.website)?;
        state.serialize_field("username", &self.username)?;
        state.serialize_field("description", &self.description)?;
        state.end()
    }
}
