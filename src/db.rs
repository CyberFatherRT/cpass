use crate::error::CpassError;

use async_trait::async_trait;
use sqlx::{pool::PoolConnection, PgPool, Postgres, Transaction};

#[async_trait]
pub trait Db {
    type Conn;
    type Thx;

    async fn conn(&self) -> Result<Self::Conn, CpassError>;
    async fn thx(&self) -> Result<Self::Thx, CpassError>;
}

#[async_trait]
impl Db for PgPool {
    type Conn = PoolConnection<Postgres>;
    type Thx = Transaction<'static, Postgres>;

    async fn conn(&self) -> Result<Self::Conn, CpassError> {
        self.acquire().await.map_err(CpassError::DatabaseError)
    }

    async fn thx(&self) -> Result<Self::Thx, CpassError> {
        self.begin().await.map_err(CpassError::DatabaseError)
    }
}
