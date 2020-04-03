use crate::{
    data::PostgresRepository,
    proto::{rpts_server::Rpts, HiRequest, HiResponse, User, UserRequest},
};
use tonic::{Request, Response, Status};

#[allow(clippy::module_name_repetitions)]
pub struct Rpts01Service {
    pub repository: PostgresRepository,
}

#[tonic::async_trait]
impl Rpts for Rpts01Service {
    async fn say_hi(&self, request: Request<HiRequest>) -> Result<Response<HiResponse>, Status> {
        let response = HiResponse {
            message: format!("Hello {}! How are you?", request.into_inner().hello),
        };
        Ok(Response::new(response))
    }

    async fn get_user(&self, request: Request<UserRequest>) -> Result<Response<User>, Status> {
        let name = request.into_inner().name;

        let user = self.repository.get_user(&name).await.map_err(|e| {
            Status::not_found(format!("No user with name {} exists. Error: {:?}", name, e))
        })?;

        Ok(Response::new(user))
    }
}
