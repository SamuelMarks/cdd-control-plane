//! API endpoints for querying organization audit logs.

use crate::api::auth_middleware::AuthenticatedUser;
use crate::db::models::AuditLog;
use crate::db::repository::CddRepository;
use crate::domain::Pagination;
use crate::error::Error;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

/// Query parameters for paginating organization audit logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, IntoParams, ToSchema)]
pub struct AuditLogQuery {
    /// Maximum number of audit log entries to return (`1..=100`, default `50`).
    pub limit: Option<i64>,
    /// Number of audit log entries to skip (`>= 0`, default `0`).
    pub offset: Option<i64>,
}

/// Lists paginated [`AuditLog`] entries for an organization if the caller is a member.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Organization primary key ID.
/// * `query` - Validated [`AuditLogQuery`] pagination parameters.
/// * `repo` - Database repository handle.
#[utoipa::path(
    get,
    path = "/orgs/{org_id}/audit",
    params(
        ("org_id" = i32, Path, description = "Organization ID"),
        AuditLogQuery
    ),
    responses(
        (status = 200, description = "Paginated list of audit logs", body = [AuditLog]),
        (status = 400, description = "Invalid pagination parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn list_audit_logs(
    user: AuthenticatedUser,
    path: web::Path<i32>,
    query: web::Query<AuditLogQuery>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let pagination = Pagination::new(query.limit, query.offset)?;
    let org_id = path.into_inner();
    let Some(role) = repo.get_user_role(org_id, user.user_id).await? else {
        return Err(Error::Forbidden(
            "User is not a member of this organization".into(),
        ));
    };
    debug_assert!(role.can_read());

    let logs = repo
        .list_audit_logs(org_id, pagination.limit, pagination.offset)
        .await?;
    Ok(HttpResponse::Ok().json(logs))
}

/// Configures the `/orgs/{org_id}/audit` route on the provided [`web::ServiceConfig`].
///
/// # Arguments
///
/// * `cfg` - Actix service configuration builder.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/orgs/{org_id}/audit").route("", web::get().to(list_audit_logs)));
}

#[cfg(test)]
mod tests {
    //! Unit tests for audit log pagination and RBAC enforcement.

    use super::*;
    use crate::api::auth_middleware::{generate_test_token, test_app_config};
    use crate::db::repository::MockCddRepository;
    use crate::domain::OrgRole;
    use actix_web::{body::to_bytes, http::StatusCode, test, App};

    #[actix_web::test]
    async fn test_list_audit_logs_rbac_and_pagination() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();

        mock_repo
            .expect_get_user_role()
            .withf(|org_id, uid| *org_id == 1 && *uid == 1)
            .times(1)
            .returning(|_, _| Ok(None));

        mock_repo
            .expect_get_user_role()
            .withf(|org_id, uid| *org_id == 2 && *uid == 1)
            .times(2)
            .returning(|_, _| Ok(Some(OrgRole::Member)));

        mock_repo
            .expect_list_audit_logs()
            .withf(|org_id, limit, offset| *org_id == 2 && *limit == 10 && *offset == 5)
            .times(1)
            .returning(|org_id, _, _| {
                Ok(vec![AuditLog {
                    id: 77,
                    org_id,
                    repo_id: None,
                    user_id: 1,
                    action: "org_created".into(),
                    metadata_json: None,
                    timestamp: chrono::Utc::now().naive_utc(),
                }])
            });

        mock_repo
            .expect_list_audit_logs()
            .withf(|org_id, limit, offset| *org_id == 2 && *limit == 50 && *offset == 0)
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let token = generate_test_token()?;

        // Non-member -> 403 Forbidden
        let req_forbidden = test::TestRequest::get()
            .uri("/orgs/1/audit")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_forbidden).await.status(),
            StatusCode::FORBIDDEN
        );

        // Custom limit & offset -> 200 OK with expected log
        let req_custom = test::TestRequest::get()
            .uri("/orgs/2/audit?limit=10&offset=5")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_custom = test::call_service(&app, req_custom).await;
        assert_eq!(resp_custom.status(), StatusCode::OK);
        let bytes = to_bytes(resp_custom.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let logs: Vec<AuditLog> = serde_json::from_slice(&bytes)?;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].id, 77);

        // Default limit & offset -> 200 OK
        let req_default = test::TestRequest::get()
            .uri("/orgs/2/audit")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_default).await.status(),
            StatusCode::OK
        );

        // Invalid pagination (limit > 100 or negative offset) -> 400 Bad Request
        let req_bad_limit = test::TestRequest::get()
            .uri("/orgs/2/audit?limit=500")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_bad_limit).await.status(),
            StatusCode::BAD_REQUEST
        );

        Ok(())
    }

    #[actix_web::test]
    async fn test_list_audit_logs_db_error() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_get_user_role()
            .times(1)
            .returning(|_, _| Err(Error::InternalError));

        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let token = generate_test_token()?;
        let req = test::TestRequest::get()
            .uri("/orgs/1/audit")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }
}
