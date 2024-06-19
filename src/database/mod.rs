use crate::error::CpassError;

mod postgres;

#[async_trait::async_trait]
pub trait Db {
    type Conn;

    async fn conn(&self) -> Result<Self::Conn, CpassError>;
}
