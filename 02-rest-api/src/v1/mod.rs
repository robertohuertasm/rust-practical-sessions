mod handlers;
pub mod repository;
pub mod service;

use actix_web::web;
use handlers::users;

/// Configures the API
pub fn api<S: service::Service + 'static>(cfg: &mut web::ServiceConfig) {
    configure_users::<S>(cfg);
}

/// Configures the user endpoints
fn configure_users<S: service::Service + 'static>(cfg: &mut web::ServiceConfig) {
    let path_user_id = "/{id}";
    cfg.service(
        web::scope(users::PATH)
            // GET
            .route(path_user_id, web::get().to(users::get::<S>))
            // POST
            .route("/", web::post().to(users::post::<S>))
            // PATCH
            .route(&path_user_id, web::patch().to(users::patch::<S>))
            // DELETE
            .route(&path_user_id, web::delete().to(users::delete::<S>)),
    );
}
