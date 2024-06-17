use crate::{
    auth::{routers::*, structs::*},
    pass::{password_routers::*, structs::*, tag_routers::*},
    structs::Password,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        login, create_user, update_user, delete_user,
        add_password, get_all_passwords, get_password, delete_password, update_password,
        add_tags, delete_tags, set_tags
    ),
    components(
        schemas(
            Password,
            AddPassword,
            AddPasswordResponse,
            Tags,
            CreateUser,
            UpdateUser,
            DeleteUser,
            LoginUser,
            LoginUnauthorized,
            AuthUserResponse
        ),
    ),
    tags(
        (name = "Auth", description = "Authentication and user management"),
        (name = "Password", description = "Password management"),
        (name = "Tag", description = "Tag management")
    ),
)]
pub struct ApiDoc;
