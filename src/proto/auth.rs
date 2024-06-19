use crate::proto::{
    auth_proto::{
        auth_server::Auth, CreateUserRequest, DeleteUserRequest, LoginRequest, UpdateUserRequest,
        User,
    },
    types::Empty,
};
use sqlx::PgPool;
use tonic::{Request, Response, Status};

pub struct AuthService {
    pool: PgPool,
}

impl AuthService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl Auth for AuthService {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<User>, Status> {
        todo!()
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        todo!()
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<Empty>, Status> {
        todo!()
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<Empty>, Status> {
        todo!()
    }
}
