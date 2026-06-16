//! API endpoints and router configuration.

pub mod audit;
pub mod auth;
pub mod auth_middleware;
pub mod orgs;
pub mod repos;
pub mod tokens;

use actix_web::web;

/// Configures the API router.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health_check))
            .configure(auth::configure)
            .configure(orgs::configure)
            .configure(repos::configure)
            .configure(tokens::configure)
            .configure(audit::configure),
    );
}

/// Health check endpoint.
async fn health_check() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "ok"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(App::new().configure(configure)).await;
        let req = test::TestRequest::get().uri("/api/v1/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }
}
