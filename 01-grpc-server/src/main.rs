mod data;
mod proto;
mod service;

use proto::rpts_server::RptsServer;
use service::Rpts01Service;
use std::env;
use tonic::{metadata::MetadataValue, transport::Server, Request, Status};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let address = "0.0.0.0:50051";
    let addr = address.parse()?;
    let conn_str = &env::var("DATABASE_URL")?;

    let repository = data::PostgresRepository::build(conn_str).await?;
    let rpts01_service = Rpts01Service { repository };

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
