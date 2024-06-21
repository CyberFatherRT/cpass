use crate::{
    db::Db,
    error::CpassError,
    jwt::models::Claims,
    proto::{
        pass_proto::{
            pass_server::Pass, AddPasswordRequest, DeletePasswordRequest, Password, Passwords,
            UpdatePasswordRequest,
        },
        types::{Empty, Uuid},
    },
};
use sqlx::PgPool;
use tonic::{Request, Response, Status};

pub struct PassService {
    pool: PgPool,
}

impl PassService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl Pass for PassService {
    async fn get_password(&self, request: Request<Uuid>) -> Result<Response<Password>, Status> {
        let mut conn = self.pool.conn().await?;
        let Uuid { uuid } = request.get_ref();
        let pass_id = uuid
            .parse::<uuid::Uuid>()
            .map_err(|_| Status::invalid_argument("Can not parse string as uuid."))?;

        let headers = request.metadata().to_owned().into_headers();

        if !headers.contains_key("authorization") {
            return Err(Status::unauthenticated("No authorization token was found"));
        }

        let claims: Claims = headers
            .get("authorization")
            .unwrap()
            .to_str()
            .map_err(|_| Status::invalid_argument("Wrong authorization Bearer format"))?
            .parse()?;

        let owner_id = claims.sub;

        let password = sqlx::query!(
            r#"
            SELECT id, password, name, salt, website, username, description, tags
            FROM passwords
            WHERE id = $1 AND owner_id = $2
            "#,
            pass_id,
            owner_id
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => Status::not_found("Password with that id not found"),
            _ => CpassError::DatabaseError(err).into(),
        })?;

        let uuid = password.id.to_string();
        let name = password.name;
        let encrypted_password = hex::encode(password.password);
        let salt = password.salt.map(hex::encode);

        let website = password.website;
        let username = password.username;
        let description = password.description;
        let tags = password.tags;

        Ok(Response::new(Password {
            uuid,
            name,
            encrypted_password,
            salt,
            website,
            username,
            description,
            tags,
        }))
    }

    async fn get_passwords(&self, request: Request<Empty>) -> Result<Response<Passwords>, Status> {
        todo!()
    }

    async fn add_password(
        &self,
        request: Request<AddPasswordRequest>,
    ) -> Result<Response<Uuid>, Status> {
        todo!()
    }

    async fn update_password(
        &self,
        request: Request<UpdatePasswordRequest>,
    ) -> Result<Response<Empty>, Status> {
        todo!()
    }

    async fn delete_password(
        &self,
        request: Request<DeletePasswordRequest>,
    ) -> Result<Response<Empty>, Status> {
        todo!()
    }
}
