use proto::{
    rpts_server::{Rpts, RptsServer},
    HiRequest, HiResponse, User, UserRequest,
};
use tonic::{metadata::MetadataValue, transport::Server, Request, Response, Status};

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

    async fn get_user(&self, _: Request<UserRequest>) -> Result<Response<User>, Status> {
        let response = User::default();
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "0.0.0.0:50051";
    let addr = address.parse()?;
    let rpts01_service = Rpts01Service {};

    Server::builder()
        .add_service(RptsServer::with_interceptor(rpts01_service, interceptor))
        .serve(addr)
        .await?;

    Ok(())

    // use the grpcurl calls below to test it
    // Unauthenticated
    // grpcurl -plaintext -import-path ./proto -proto rpts01.proto -d '{"hello": "Rob"}' localhost:50051 rpts01.Rpts/SayHi

    // Authenticated
    // grpcurl -plaintext -import-path ./proto -proto rpts01.proto -d '{"hello": "Rob"}' -H 'authorization: Bearer myjwttoken' localhost:50051 rpts01.Rpts/SayHi
    // grpcurl -plaintext -import-path ./proto -proto rpts01.proto -d '{"name": "Roberto"}' -H 'authorization: Bearer myjwttoken' localhost:50051 rpts01.Rpts/GetUser
}

fn interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    let token = MetadataValue::from_str("Bearer myjwttoken").unwrap();
    println!("Validating the request");
    match req.metadata().get("authorization") {
        Some(t) if t == token => Ok(req),
        _ => Err(Status::unauthenticated("The token is invalid")),
    }
}
