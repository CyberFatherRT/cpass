use crate::error::CpassError;

use super::Db;
use async_trait::async_trait;
use sqlx::{pool::PoolConnection, PgPool, Postgres};

#[async_trait]
impl Db for PgPool {
    type Conn = PoolConnection<Postgres>;

    async fn conn(&self) -> Result<Self::Conn, CpassError> {
        self.acquire().await.map_err(CpassError::DatabaseError)
    }
}
