use std::ops::Deref;

use crate::{
    db::Db,
    error::CpassError,
    hashing::Argon,
    jwt::generate::create_token,
    proto::{
        auth_proto::{
            auth_server::Auth, CreateUserRequest, DeleteUserRequest, LoginRequest,
            UpdateUserRequest, User,
        },
        types::Empty,
    },
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
        let mut conn = self.pool.conn().await?;
        let LoginRequest { email, password } = request.get_ref();

        let res = sqlx::query!(
            r#"
            SELECT * FROM users WHERE email = $1
            "#,
            email
        )
        .fetch_one(&mut *conn)
        .await;

        let res = match res {
            Ok(ok) => ok,
            Err(_) => return Err(CpassError::InvalidUsernameOrPassword.into()),
        };

        match Argon::verify(password.deref().as_bytes(), &res.password) {
            Ok(false) => return Err(CpassError::InvalidUsernameOrPassword.into()),
            Err(e) => return Err(e.into()),
            _ => {}
        }

        let token = create_token(&res.id)?;

        let user = User {
            token,
            email: res.email,
            username: res.username,
        };

        let response = Response::new(user);

        Ok(response)
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
