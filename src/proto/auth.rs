use crate::{
    db::Db,
    error::CpassError,
    hashing::Argon,
    jwt::generate::{claims_from_metadata, create_token},
    proto::{
        auth_proto::{auth_server::Auth, CreateUserRequest, LoginRequest, UpdateUserRequest, User},
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

        let user = sqlx::query!(
            r#"
            SELECT * FROM users WHERE email = $1
            "#,
            email
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => CpassError::InvalidUsernameOrPassword,
            err => CpassError::DatabaseError(err),
        })?;

        match Argon::verify(password.as_bytes(), &user.password) {
            Ok(false) => return Err(CpassError::InvalidUsernameOrPassword.into()),
            Err(e) => return Err(e.into()),
            _ => {}
        }

        let token = create_token(&user.id)?;

        let user = User {
            token,
            email: user.email,
            username: user.username,
        };

        let response = Response::new(user);

        Ok(response)
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let mut conn = self.pool.conn().await?;
        let CreateUserRequest {
            email,
            username,
            password,
        } = request.get_ref().to_owned();

        let hash = Argon::hash_password(password.as_bytes())?;

        let res = sqlx::query!(
            r#"
            INSERT INTO users(email, username, password)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            email,
            username,
            hash
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(|err| match err {
            sqlx::Error::Database(err) if err.is_unique_violation() => {
                CpassError::UserAlreadyExists(email.to_string())
            }
            err => CpassError::DatabaseError(err),
        })?;

        let token = create_token(&res.id)?;

        Ok(Response::new(User {
            token,
            email,
            username,
        }))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<Empty>, Status> {
        let mut conn = self.pool.conn().await?;
        let UpdateUserRequest {
            email,
            username,
            password,
        } = request.get_ref().to_owned();

        let user_id = claims_from_metadata(request.metadata())?.sub;

        let password = password
            .map(|pass| Argon::hash_password(pass.as_bytes()))
            .transpose()?;

        let _ = sqlx::query!(
            r#"
            UPDATE users
            SET
                email = COALESCE($1, email),
                username = COALESCE($2, username),
                password = COALESCE($3, password)
            WHERE id = $4
            "#,
            email,
            username,
            password,
            user_id
        )
        .execute(&mut *conn)
        .await
        .map_err(|err| match err {
            sqlx::Error::Database(err) if err.is_unique_violation() => {
                CpassError::UserAlreadyExists(email.unwrap())
            }
            err => CpassError::DatabaseError(err),
        })?;

        Ok(Response::new(Empty {}))
    }

    async fn delete_user(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let mut conn = self.pool.conn().await?;

        let user_id = claims_from_metadata(request.metadata())?.sub;

        let _ = sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .execute(&mut *conn)
        .await
        .map_err(CpassError::DatabaseError);

        Ok(Response::new(Empty {}))
    }
}
