use super::{auth::*, models::*};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        login, create_user, update_user, delete_user,
    ),
    components(
        schemas(
            LoginRequest,
            CreateUserRequest,
            UpdateUserRequest,
            User
        ),
    ),
    tags(
        (name = "Auth", description = "Authentication and user management"),
    ),
)]
pub struct ApiDoc;
