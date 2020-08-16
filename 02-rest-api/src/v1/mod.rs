mod handlers;
pub mod repository;
pub mod service;

use actix_web::web;
use handlers::users;

/// Configures the API
pub fn api(cfg: &mut web::ServiceConfig) {
    configure_users(cfg);
}

/// Configures the user endpoints
fn configure_users(cfg: &mut web::ServiceConfig) {
    let resource_path = format!("{}/{{id}}", users::PATH);
    cfg
        // GET
        .route(&resource_path, web::get().to(users::get))
        // POST
        .route(users::PATH, web::post().to(users::post))
        // PATCH
        .route(&resource_path, web::patch().to(users::patch))
        // DELETE
        .route(&resource_path, web::delete().to(users::delete));
}
