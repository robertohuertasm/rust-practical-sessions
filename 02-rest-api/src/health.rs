use actix_web::{web, HttpResponse};

/// Health endpoint
pub fn endpoint(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(handler));
}

async fn handler() -> HttpResponse {
    HttpResponse::Ok().body("ok")
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_rt::test]
    async fn health_handler_works() {
        let res: HttpResponse = handler().await;
        assert!(res.status().is_success());
    }

    #[actix_rt::test]
    async fn health_handler_integration_works() {
        let svc = App::new().route("/health", web::get().to(handler));
        let mut app = test::init_service(svc).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());
    }
}
