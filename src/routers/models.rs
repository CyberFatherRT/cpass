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
    pub name: String,
    pub password: Vec<u8>,
    pub salt: Option<Vec<u8>>,
    pub website: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(ToSchema)]
pub struct UpdatePasswordRequest {
    pub name: Option<String>,
    pub password: Option<Vec<u8>>,
    pub salt: Option<Vec<u8>>,
    pub website: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(ToSchema)]
pub struct Password {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub password: Vec<u8>,
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
        let binding = Vec::new();
        let salt = match &self.salt {
            Some(data) => data,
            None => &binding,
        };
        state.serialize_field("id", &self.uuid)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("password", &hex::encode(&self.password))?;
        state.serialize_field("salt", &hex::encode(salt))?;
        state.serialize_field("website", &self.website)?;
        state.serialize_field("username", &self.username)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("tags", &self.tags)?;
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
        let mut salt = None;
        let mut website = None;
        let mut username = None;
        let mut description = None;
        let mut tags = None;

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
                "salt" => {
                    if salt.is_some() {
                        return Err(de::Error::duplicate_field("salt"));
                    }
                    salt = Some(map.next_value()?)
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
                "tags" => {
                    if tags.is_some() {
                        return Err(de::Error::duplicate_field("tags"));
                    }
                    tags = Some(map.next_value::<Vec<String>>()?)
                }
                _ => {
                    let _: de::IgnoredAny = map.next_value()?;
                }
            }
        }

        Ok(UpdatePasswordRequest {
            name,
            password: match password {
                Some(data) => hex::decode(data).ok(),
                None => None,
            },
            salt,
            website,
            username,
            description,
            tags,
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
                "salt",
                "website",
                "username",
                "description",
                "tags",
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
        let mut salt = None;
        let mut website = None;
        let mut username = None;
        let mut description = None;
        let mut tags = None;

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
                "salt" => {
                    if salt.is_some() {
                        return Err(de::Error::duplicate_field("salt"));
                    }
                    salt = Some(map.next_value()?)
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
                "tags" => {
                    if tags.is_some() {
                        return Err(de::Error::duplicate_field("tags"));
                    }
                    tags = Some(map.next_value::<Vec<String>>()?)
                }
                _ => {
                    let _: de::IgnoredAny = map.next_value()?;
                }
            }
        }

        let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
        let password = password.ok_or_else(|| de::Error::missing_field("password"))?;

        Ok(AddPasswordRequest {
            name,
            password: hex::decode(password).map_err(|e| de::Error::custom(e.to_string()))?,
            salt,
            website,
            username,
            description,
            tags: tags.unwrap_or_default(),
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
            &[
                "name",
                "password",
                "salt",
                "website",
                "username",
                "description",
                "tags",
            ],
            AddPasswordRequestVisitor,
        )
    }
}
