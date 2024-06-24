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
        let pass_id = uuid
            .parse::<uuid::Uuid>()
            .map_err(|_| Status::invalid_argument("Can not parse string as uuid."))?;

        let owner_id = claims_from_headers(request.metadata())?.sub;

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
        let mut conn = self.pool.conn().await?;

        let owner_id = claims_from_headers(request.metadata())?.sub;

        let passwords = sqlx::query!(
            r#"
            SELECT id, password, name, salt, website, username, description, tags
            FROM passwords
            WHERE owner_id = $1
            "#,
            owner_id
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => Status::not_found("Password with that id not found"),
            _ => CpassError::DatabaseError(err).into(),
        })?;

        let passwords = passwords
            .into_iter()
            .map(|x| {
                let uuid = x.id.to_string();
                let name = x.name;
                let encrypted_password = hex::encode(x.password);
                let salt = x.salt.map(hex::encode);
                let website = x.website;
                let username = x.username;
                let description = x.description;
                let tags = x.tags;

                Password {
                    uuid,
                    name,
                    encrypted_password,
                    salt,
                    website,
                    username,
                    description,
                    tags,
                }
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
            salt,
            website,
            username,
            description,
            tags,
        } = request.get_ref().to_owned();

        let owner_id = claims_from_headers(request.metadata())?.sub;

        let password = hex::decode(password)
            .map_err(|_| Status::invalid_argument("Can not decode password from hex"))?;

        let salt = salt
            .map(|data| {
                hex::decode(data)
                    .map_err(|_| Status::invalid_argument("Can not decode salt from hex"))
            })
            .transpose()?;

        let row = sqlx::query!(
            r#"
            INSERT INTO passwords(owner_id, name, password, salt, website, username, description, tags)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
            owner_id,
            name,
            password,
            salt,
            website,
            username,
            description,
            &tags
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
            salt,
            website,
            username,
            description,
            tags,
        } = request.get_ref().to_owned();

        let owner_id = claims_from_headers(request.metadata())?.sub;

        let password = password
            .map(|data| {
                hex::decode(data)
                    .map_err(|_| Status::invalid_argument("Can not decode password from hex"))
            })
            .transpose()?;

        let salt = salt
            .map(|data| {
                hex::decode(data)
                    .map_err(|_| Status::invalid_argument("Can not decode salt from hex"))
            })
            .transpose()?;

        let pass_id: uuid::Uuid = uuid
            .parse()
            .map_err(|_| Status::invalid_argument("Can not parse uuid field as UUID"))?;

        let _ = sqlx::query!(
            r#"
            UPDATE passwords
            SET
                name = COALESCE($1, name),
                password = COALESCE($2, password),
                salt = COALESCE($3, salt),
                website = COALESCE($4, website),
                username = COALESCE($5, username),
                description = COALESCE($6, description),
                tags = COALESCE($7, tags)
            WHERE id = $8 AND owner_id = $9
            "#,
            name,
            password,
            salt,
            website,
            username,
            description,
            &tags,
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
        let pass_id: uuid::Uuid = uuid
            .parse()
            .map_err(|_| Status::invalid_argument("Can not parse uuid field as UUID"))?;

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
