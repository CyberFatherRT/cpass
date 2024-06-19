use crate::proto::{
    pass_proto::{
        pass_server::Pass, AddPasswordRequest, DeletePasswordRequest, Password, Passwords,
        UpdatePasswordRequest,
    },
    types::{Empty, Uuid},
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
        todo!()
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
