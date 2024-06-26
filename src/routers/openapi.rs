use super::{auth::*, models::*, pass::*};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        login, create_user, update_user, delete_user,
        get_password, add_password, get_passwords, update_password, delete_password
    ),
    components(
        schemas(
            LoginRequest,
            CreateUserRequest,
            UpdateUserRequest,
            User,
            Password,
            AddPasswordRequest,
            UpdatePasswordRequest,
        ),
    ),
    tags(
        (name = "Auth", description = "Authentication and user management"),
    ),
)]
pub struct ApiDoc;
