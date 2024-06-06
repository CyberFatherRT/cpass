use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddPassword {
    pub password: String,
    pub name: String,
    pub website: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub master_password: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct AddTagsToPassword {
    pub tags: Vec<String>,
}
