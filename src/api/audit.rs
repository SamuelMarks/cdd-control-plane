//! API endpoints for audit logs.

use crate::api::auth_middleware::AuthenticatedUser;
use crate::db::repository::CddRepository;
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;

/// Query parameters for listing audit logs.
#[derive(Deserialize)]
pub struct AuditLogQuery {
    /// Limit the number of results
    pub limit: Option<i64>,
    /// Offset the results
    pub offset: Option<i64>,
}

/// Get a list of audit logs for an organization.
pub async fn list_audit_logs(
    user: AuthenticatedUser,
    path: web::Path<i32>,
    query: web::Query<AuditLogQuery>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, crate::error::Error> {
    let org_id = path.into_inner();
    let role = repo.get_user_role(org_id, user.user_id).await?;

    // Only organization members can view audit logs
    if role.is_none() {
        return Err(crate::error::Error::Unauthorized);
    }

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let logs = repo.list_audit_logs(org_id, limit, offset).await?;
    Ok(HttpResponse::Ok().json(logs))
}

/// Configure audit routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/orgs/{org_id}/audit").route("", web::get().to(list_audit_logs)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::auth_middleware::generate_test_token;
    use crate::db::repository::MockCddRepository;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_list_audit_logs_unauthorized() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo.expect_get_user_role().returning(|_, _| Ok(None));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/orgs/1/audit")
            .insert_header((
                "Authorization",
                format!("Bearer {}", generate_test_token()?),
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
        Ok(())
    }

    #[actix_web::test]
    async fn test_list_audit_logs_authorized() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_get_user_role()
            .returning(|_, _| Ok(Some("member".to_string())));
        mock_repo
            .expect_list_audit_logs()
            .returning(|_, _, _| Ok(vec![]));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/orgs/1/audit?limit=10&offset=5")
            .insert_header((
                "Authorization",
                format!("Bearer {}", generate_test_token()?),
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
        Ok(())
    }
}
