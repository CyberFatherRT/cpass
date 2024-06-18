use crate::proto::{
    tag_proto::{tag_server::Tag, SetTagsRequest,}, types::Empty,
};
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct TagService {
}

#[tonic::async_trait]
impl Tag for TagService {
    async fn set_tags(&self, request: Request<SetTagsRequest>) -> Result<Response<Empty>, Status> {
        todo!()
    }
}
