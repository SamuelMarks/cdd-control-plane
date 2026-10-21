//! API endpoints and top-level `/api/v1` router configuration.

pub mod audit;
pub mod auth;
pub mod auth_middleware;
pub mod orgs;
pub mod repos;
pub mod tokens;

use actix_web::web;

/// Configures the `/api/v1` router scope and mounts all domain sub-routers.
///
/// # Arguments
///
/// * `cfg` - Actix service configuration builder.
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

/// Liveness and readiness health check endpoint (`GET /api/v1/health`).
async fn health_check() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "ok"
    }))
}

#[cfg(test)]
mod tests {
    //! Unit tests for the `/api/v1/health` endpoint.

    use super::*;
    use crate::error::Error;
    use actix_web::{body::to_bytes, http::StatusCode, test, App};

    #[actix_web::test]
    async fn test_health_check() -> Result<(), Error> {
        let app = test::init_service(App::new().configure(configure)).await;
        let req = test::TestRequest::get().uri("/api/v1/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let json: serde_json::Value = serde_json::from_slice(&body)?;
        assert_eq!(json["status"], "ok");
        Ok(())
    }
}
