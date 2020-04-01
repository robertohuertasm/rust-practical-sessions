use proto::{
    rpts_server::{Rpts, RptsServer},
    HiRequest, HiResponse,
};
use tonic::{transport::Server, Request, Response, Status};

pub mod proto {
    tonic::include_proto!("rpts01");
}

pub struct Rpts01Service {}

#[tonic::async_trait]
impl Rpts for Rpts01Service {
    async fn say_hi(&self, request: Request<HiRequest>) -> Result<Response<HiResponse>, Status> {
        let response = HiResponse {
            message: format!("Hello {}! How are you?", request.into_inner().hello),
        };
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "0.0.0.0:50051";
    let addr = address.parse()?;
    let rpts01_service = Rpts01Service {};

    Server::builder()
        .add_service(RptsServer::new(rpts01_service))
        .serve(addr)
        .await?;

    Ok(())

    // use the grpcurl call below to test it
    // grpcurl -plaintext -import-path ./proto -proto rpts01.proto -d '{"hello": "Rob"}' localhost:50051 rpts01.Rpts/SayHi
}
