use crate::proto::{
    tag_proto::{tag_server::Tag, SetTagsRequest},
    types::Empty,
};
use sqlx::PgPool;
use tonic::{Request, Response, Status};

pub struct TagService {
    pool: PgPool,
}

impl TagService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl Tag for TagService {
    async fn set_tags(&self, request: Request<SetTagsRequest>) -> Result<Response<Empty>, Status> {
        todo!()
    }
}
