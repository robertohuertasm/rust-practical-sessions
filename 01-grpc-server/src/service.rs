use crate::{
    data::Repository,
    proto::{rpts_server::Rpts, HiRequest, HiResponse, User, UserRequest},
};
use tonic::{Request, Response, Status};

#[allow(clippy::module_name_repetitions)]
pub struct Rpts01Service<T: Repository> {
    pub repository: T,
}

#[tonic::async_trait]
impl<T: Repository + Send + Sync + 'static> Rpts for Rpts01Service<T> {
    async fn say_hi(&self, request: Request<HiRequest>) -> Result<Response<HiResponse>, Status> {
        let response = HiResponse {
            message: format!("Hello {}! How are you?", request.into_inner().hello),
        };
        Ok(Response::new(response))
    }

    async fn get_user(&self, request: Request<UserRequest>) -> Result<Response<User>, Status> {
        let name = request.into_inner().name;

        self.repository
            .get_user(&name)
            .await
            .map(Response::new)
            .map_err(|e| {
                Status::not_found(format!("No user with name {} exists. Error: {:?}", name, e))
            })
    }
}
