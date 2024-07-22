use base64::Engine;
use serde::{
    de::{self, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};
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
pub struct AddPasswordRequest {
    pub name: Vec<u8>,
    pub password: Vec<u8>,
    pub website: Option<Vec<u8>>,
    pub username: Option<Vec<u8>>,
    pub description: Option<Vec<u8>>,
}

#[derive(ToSchema)]
pub struct UpdatePasswordRequest {
    pub name: Option<Vec<u8>>,
    pub password: Option<Vec<u8>>,
    pub website: Option<Vec<u8>>,
    pub username: Option<Vec<u8>>,
    pub description: Option<Vec<u8>>,
}

#[derive(ToSchema)]
pub struct Password {
    pub uuid: uuid::Uuid,
    pub name: Vec<u8>,
    pub password: Vec<u8>,
    pub website: Option<Vec<u8>>,
    pub username: Option<Vec<u8>>,
    pub description: Option<Vec<u8>>,
}

fn to_base64(data: &[u8]) -> String {
    base64::prelude::BASE64_STANDARD.encode(data)
}

fn from_base64(data: &str) -> Option<Vec<u8>> {
    base64::prelude::BASE64_STANDARD
        .decode(data.as_bytes())
        .ok()
}

impl Serialize for Password {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Password", 6)?;
        state.serialize_field("id", &self.uuid)?;
        state.serialize_field("name", &to_base64(&self.name))?;
        state.serialize_field("password", &to_base64(&self.password))?;
        state.serialize_field("website", &self.website.as_ref().map(|x| to_base64(x)))?;
        state.serialize_field("username", &self.username.as_ref().map(|x| to_base64(x)))?;
        state.serialize_field(
            "description",
            &self.description.as_ref().map(|x| to_base64(x)),
        )?;
        state.end()
    }
}

struct UpdatePasswordRequestVisitor;

impl<'de> Visitor<'de> for UpdatePasswordRequestVisitor {
    type Value = UpdatePasswordRequest;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a struct representing UpdatePasswordRequest")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut name = None;
        let mut password = None;
        let mut website = None;
        let mut username = None;
        let mut description = None;

        while let Some(key) = map.next_key()? {
            match key {
                "name" => {
                    if name.is_some() {
                        return Err(de::Error::duplicate_field("name"));
                    }
                    name = Some(map.next_value()?)
                }
                "password" => {
                    if password.is_some() {
                        return Err(de::Error::duplicate_field("password"));
                    }

                    password = Some(map.next_value()?)
                }
                "website" => {
                    if website.is_some() {
                        return Err(de::Error::duplicate_field("website"));
                    }
                    website = Some(map.next_value()?)
                }
                "username" => {
                    if username.is_some() {
                        return Err(de::Error::duplicate_field("username"));
                    }
                    username = Some(map.next_value()?)
                }
                "description" => {
                    if description.is_some() {
                        return Err(de::Error::duplicate_field("description"));
                    }
                    description = Some(map.next_value()?)
                }
                _ => {
                    let _: de::IgnoredAny = map.next_value()?;
                }
            }
        }

        let to_result = || de::Error::custom("Can not decode `password` from base64");

        Ok(UpdatePasswordRequest {
            name: name.map(from_base64).ok_or_else(to_result)?,
            password: password.map(from_base64).ok_or_else(to_result)?,
            website: website.map(from_base64).ok_or_else(to_result)?,
            username: username.map(from_base64).ok_or_else(to_result)?,
            description: description.map(from_base64).ok_or_else(to_result)?,
        })
    }
}

impl<'de> Deserialize<'de> for UpdatePasswordRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "UpdatePasswordRequest",
            &[
                "uuid",
                "name",
                "password",
                "website",
                "username",
                "description",
            ],
            UpdatePasswordRequestVisitor,
        )
    }
}

struct AddPasswordRequestVisitor;

impl<'de> Visitor<'de> for AddPasswordRequestVisitor {
    type Value = AddPasswordRequest;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a struct representing AddPasswordRequest")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut name = None;
        let mut password = None;
        let mut website = None;
        let mut username = None;
        let mut description = None;

        while let Some(key) = map.next_key()? {
            match key {
                "name" => {
                    if name.is_some() {
                        return Err(de::Error::duplicate_field("name"));
                    }
                    name = Some(map.next_value()?)
                }
                "password" => {
                    if password.is_some() {
                        return Err(de::Error::duplicate_field("password"));
                    }

                    password = Some(map.next_value::<String>()?)
                }
                "website" => {
                    if website.is_some() {
                        return Err(de::Error::duplicate_field("website"));
                    }
                    website = Some(map.next_value()?)
                }
                "username" => {
                    if username.is_some() {
                        return Err(de::Error::duplicate_field("username"));
                    }
                    username = Some(map.next_value()?)
                }
                "description" => {
                    if description.is_some() {
                        return Err(de::Error::duplicate_field("description"));
                    }
                    description = Some(map.next_value()?)
                }
                _ => {
                    let _: de::IgnoredAny = map.next_value()?;
                }
            }
        }

        let to_result = || de::Error::custom("Can not decode `password` from base64");

        let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
        let password = password.ok_or_else(|| de::Error::missing_field("password"))?;

        Ok(AddPasswordRequest {
            name,
            password: from_base64(&password).ok_or_else(to_result)?,
            website: website.map(from_base64).ok_or_else(to_result)?,
            username: username.map(from_base64).ok_or_else(to_result)?,
            description: description.map(from_base64).ok_or_else(to_result)?,
        })
    }
}

impl<'de> Deserialize<'de> for AddPasswordRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "AddPasswordRequest",
            &["name", "password", "website", "username", "description"],
            AddPasswordRequestVisitor,
        )
    }
}
