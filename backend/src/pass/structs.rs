use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddPassword {
    pub password: String,
    pub name: String,
    pub website: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub master_password: String,
}

#[derive(Deserialize)]
pub struct AddTagsToPassword {
    pub id: String,
    pub tags: Vec<String>,
}
