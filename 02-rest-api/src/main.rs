#[macro_use]
mod macros;
mod health;
mod models;
mod v1;

use actix_cors::Cors;
use actix_web::{
    middleware::{self, Logger},
    web, App, HttpServer,
};
use actix_web_middleware_cognito::{Cognito, CognitoValidator};
use actix_web_prom::PrometheusMetrics;
use middleware::Compress;
use std::sync::Arc;
use tracing as log;
use v1::repository::PostgresRepository;
use v1::service::{Rpts02Service, ServiceInjector};

const PORT: &str = "3000";

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // set up
    dotenv::dotenv().ok();
    // structured logging and distributed tracing
    tracing_subscriber::fmt()
        .with_ansi(true)
        .json()
        .flatten_event(true)
        .with_target(true)
        .with_span_list(true)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc3339())
        .init();
    // cognito validator initialized here to avoid building it too many times
    // this reads from env variables to be built, so you'll have to restart the server
    // for changes to take effect.
    let cognito_validator =
        Arc::new(CognitoValidator::create().expect("Error generating Cognito Validator"));
    // metrics for Prometheus
    let prometheus = PrometheusMetrics::new("rpts02_api", Some("/metrics"), None);
    // instantiate a database connection pool
    let repository = PostgresRepository::build_from_env()
        .await
        .expect("Error initializing Database connection pool");
    // creating the service layer
    let svc = Rpts02Service::new(repository);
    let svc = ServiceInjector::new(svc);
    let svc = Arc::new(svc);

    // starting the server
    HttpServer::new(move || {
        log::trace!("🚀 Server thread started at port {}!", PORT);
        // cognito middleware
        let cognito = Cognito::new(cognito_validator.clone());
        // cors middleware
        let cors = Cors::default().allowed_methods(vec!["GET", "POST", "PUT", "DELETE"]);

        // set up the app
        App::new()
            .wrap(prometheus.clone())
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(cors)
            .service(
                web::scope("/v1")
                    .wrap(cognito)
                    .data(svc.clone())
                    .configure(v1::api),
            )
            .configure(health::endpoint)
    })
    .bind(format!("0.0.0.0:{}", PORT))
    .unwrap_or_else(|_| panic!("🔥 Couldn't start the server at port {}", PORT))
    .run()
    .await
}
