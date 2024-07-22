use crate::{
    db::Db,
    error::CpassError,
    jwt::generate::claims_from_headers,
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
        let pass_id = uuid::Uuid::from_slice(uuid)
            .map_err(|_| Status::invalid_argument("Can not parse string as uuid."))?;

        let owner_id = claims_from_headers(request.metadata())?.sub;

        let row = sqlx::query!(
            r#"
            SELECT id, password, name, website, username, description
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

        Ok(Response::new(Password {
            uuid: row.id.into(),
            name: row.name,
            password: row.password,
            website: row.website,
            username: row.username,
            description: row.description,
        }))
    }

    async fn get_passwords(&self, request: Request<Empty>) -> Result<Response<Passwords>, Status> {
        let mut conn = self.pool.conn().await?;

        let owner_id = claims_from_headers(request.metadata())?.sub;

        let passwords = sqlx::query!(
            r#"
            SELECT id, password, name, website, username, description
            FROM passwords
            WHERE owner_id = $1
            "#,
            owner_id
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(CpassError::DatabaseError)?;

        let passwords = passwords
            .into_iter()
            .map(|x| Password {
                uuid: x.id.into(),
                name: x.name,
                password: x.password,
                website: x.website,
                username: x.username,
                description: x.description,
            })
            .collect::<Vec<Password>>();

        Ok(Response::new(Passwords { passwords }))
    }
    async fn add_password(
        &self,
        request: Request<AddPasswordRequest>,
    ) -> Result<Response<Uuid>, Status> {
        let mut conn = self.pool.conn().await?;
        let AddPasswordRequest {
            name,
            password,
            website,
            username,
            description,
        } = request.get_ref();

        let owner_id = claims_from_headers(request.metadata())?.sub;

        let password = hex::decode(password)
            .map_err(|_| Status::invalid_argument("Can not decode password from hex"))?;

        let row = sqlx::query!(
            r#"
            INSERT INTO passwords(owner_id, name, password, website, username, description)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            owner_id,
            name,
            password,
            website.as_ref(),
            username.as_ref(),
            description.as_ref(),
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(CpassError::DatabaseError)?;

        Ok(Response::new(Uuid {
            uuid: row.id.into(),
        }))
    }

    async fn update_password(
        &self,
        request: Request<UpdatePasswordRequest>,
    ) -> Result<Response<Empty>, Status> {
        let mut conn = self.pool.conn().await?;
        let UpdatePasswordRequest {
            uuid,
            name,
            password,
            website,
            username,
            description,
        } = request.get_ref().to_owned();

        let owner_id = claims_from_headers(request.metadata())?.sub;

        let password = password
            .map(|data| {
                hex::decode(data)
                    .map_err(|_| Status::invalid_argument("Can not decode password from hex"))
            })
            .transpose()?;

        let pass_id = uuid::Uuid::from_slice(&uuid)
            .map_err(|_| Status::invalid_argument("Can not parse string as uuid."))?;

        let _ = sqlx::query!(
            r#"
            UPDATE passwords
            SET
                name = COALESCE($1, name),
                password = COALESCE($2, password),
                website = COALESCE($3, website),
                username = COALESCE($4, username),
                description = COALESCE($5, description)
            WHERE id = $6 AND owner_id = $7
            "#,
            name,
            password,
            website,
            username,
            description,
            pass_id,
            owner_id
        )
        .execute(&mut *conn)
        .await
        .map_err(CpassError::DatabaseError)?;

        Ok(Response::new(Empty {}))
    }

    async fn delete_password(
        &self,
        request: Request<DeletePasswordRequest>,
    ) -> Result<Response<Empty>, Status> {
        let mut conn = self.pool.conn().await?;
        let DeletePasswordRequest { uuid } = request.get_ref();

        let owner_id = claims_from_headers(request.metadata())?.sub;
        let pass_id = uuid::Uuid::from_slice(uuid)
            .map_err(|_| Status::invalid_argument("Can not parse string as uuid."))?;

        let _ = sqlx::query!(
            r#"
            DELETE FROM passwords
            WHERE id = $1 AND owner_id = $2
            "#,
            pass_id,
            owner_id
        )
        .execute(&mut *conn)
        .await
        .map_err(CpassError::DatabaseError);

        Ok(Response::new(Empty {}))
    }
}
