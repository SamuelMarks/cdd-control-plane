//! Organization management API endpoints.

use crate::api::auth_middleware::AuthenticatedUser;
use crate::db::models::Organization;
use crate::db::repository::CddRepository;
use crate::domain::{AuditAction, Username};
use crate::error::Error;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// Request payload for creating a new [`Organization`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreateOrgPayload {
    /// Unique organization login handle (`1..=64` alphanumeric, `-`, `_`).
    pub login: String,
    /// Optional human-readable description.
    pub description: Option<String>,
}

/// Atomically creates an [`Organization`], assigns the caller as `owner`, and records an audit event.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity extracted from Bearer JWT.
/// * `payload` - JSON [`CreateOrgPayload`] with login and description.
/// * `repo` - Database repository handle.
#[utoipa::path(
    post,
    path = "/orgs",
    request_body = CreateOrgPayload,
    responses(
        (status = 201, description = "Organization created", body = Organization),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Conflict")
    )
)]
pub async fn create_org(
    user: AuthenticatedUser,
    payload: web::Json<CreateOrgPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let login = Username::parse(&payload.login)?;
    let org = repo
        .create_organization_with_owner(
            user.user_id,
            login.as_str().to_string(),
            payload.description.clone(),
        )
        .await?;

    repo.create_audit_log(
        org.id,
        None,
        user.user_id,
        AuditAction::OrgCreated,
        Some(serde_json::json!({ "login": org.login })),
    )
    .await?;

    Ok(HttpResponse::Created().json(org))
}

/// Lists all organizations where the authenticated user is a member.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity extracted from Bearer JWT.
/// * `repo` - Database repository handle.
#[utoipa::path(
    get,
    path = "/orgs",
    responses(
        (status = 200, description = "List of organizations", body = [Organization]),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_orgs(
    user: AuthenticatedUser,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let orgs = repo.list_organizations_for_user(user.user_id).await?;
    Ok(HttpResponse::Ok().json(orgs))
}

/// Retrieves an [`Organization`] by ID if the caller holds a membership role.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity extracted from Bearer JWT.
/// * `path` - Path parameter containing the organization primary key ID.
/// * `repo` - Database repository handle.
#[utoipa::path(
    get,
    path = "/orgs/{id}",
    params(
        ("id" = i32, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "Organization details", body = Organization),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found")
    )
)]
pub async fn get_org(
    user: AuthenticatedUser,
    path: web::Path<i32>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let org_id = path.into_inner();
    let Some(role) = repo.get_user_role(org_id, user.user_id).await? else {
        return Err(Error::Forbidden(
            "User is not a member of this organization".into(),
        ));
    };
    debug_assert!(role.can_read());

    match repo.get_organization(org_id).await? {
        Some(org) => Ok(HttpResponse::Ok().json(org)),
        None => Err(Error::NotFound("Organization not found".into())),
    }
}

/// Configures the `/orgs` routes on the provided [`web::ServiceConfig`].
///
/// # Arguments
///
/// * `cfg` - Actix service configuration builder.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/orgs")
            .route("", web::post().to(create_org))
            .route("", web::get().to(list_orgs))
            .route("/{id}", web::get().to(get_org)),
    );
}

#[cfg(test)]
mod tests {
    //! Unit tests for organization creation, listing, and RBAC retrieval.

    use super::*;
    use crate::api::auth_middleware::{generate_test_token, test_app_config};
    use crate::db::models::AuditLog;
    use crate::db::repository::MockCddRepository;
    use crate::domain::OrgRole;
    use actix_web::{body::to_bytes, http::StatusCode, test, App};

    #[actix_web::test]
    async fn test_create_and_list_orgs() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_create_organization_with_owner()
            .withf(|uid, login, desc| {
                *uid == 1 && login == "acme_corp" && desc.as_deref() == Some("Acme")
            })
            .times(1)
            .returning(|_, login, description| {
                Ok(Organization {
                    id: 10,
                    github_id: None,
                    login,
                    description,
                })
            });

        mock_repo
            .expect_create_audit_log()
            .withf(|org_id, repo_id, user_id, action, _| {
                *org_id == 10
                    && repo_id.is_none()
                    && *user_id == 1
                    && *action == AuditAction::OrgCreated
            })
            .times(1)
            .returning(|org_id, repo_id, user_id, action, metadata_json| {
                Ok(AuditLog {
                    id: 1,
                    org_id,
                    repo_id,
                    user_id,
                    action: action.as_str().to_string(),
                    metadata_json,
                    timestamp: chrono::Utc::now().naive_utc(),
                })
            });

        mock_repo
            .expect_list_organizations_for_user()
            .withf(|uid| *uid == 1)
            .times(1)
            .returning(|_| {
                Ok(vec![Organization {
                    id: 10,
                    github_id: None,
                    login: "acme_corp".into(),
                    description: Some("Acme".into()),
                }])
            });

        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let token = generate_test_token()?;
        let req_create = test::TestRequest::post()
            .uri("/orgs")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateOrgPayload {
                login: "acme_corp".into(),
                description: Some("Acme".into()),
            })
            .to_request();
        let resp_create = test::call_service(&app, req_create).await;
        assert_eq!(resp_create.status(), StatusCode::CREATED);
        let created_bytes = to_bytes(resp_create.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let created_org: Organization = serde_json::from_slice(&created_bytes)?;
        assert_eq!(created_org.id, 10);
        assert_eq!(created_org.login, "acme_corp");

        let req_list = test::TestRequest::get()
            .uri("/orgs")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_list = test::call_service(&app, req_list).await;
        assert_eq!(resp_list.status(), StatusCode::OK);
        let list_bytes = to_bytes(resp_list.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let orgs: Vec<Organization> = serde_json::from_slice(&list_bytes)?;
        assert_eq!(orgs.len(), 1);
        assert_eq!(orgs[0].login, "acme_corp");

        // Invalid login rejected with 400 Bad Request
        let req_invalid = test::TestRequest::post()
            .uri("/orgs")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateOrgPayload {
                login: "invalid org name!".into(),
                description: None,
            })
            .to_request();
        let resp_invalid = test::call_service(&app, req_invalid).await;
        assert_eq!(resp_invalid.status(), StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[actix_web::test]
    async fn test_get_org_forbidden_authorized_and_not_found() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, uid| *org_id == 1 && *uid == 1)
            .times(1)
            .returning(|_, _| Ok(None));

        mock_repo
            .expect_get_user_role()
            .withf(|org_id, uid| *org_id == 2 && *uid == 1)
            .times(1)
            .returning(|_, _| Ok(Some(OrgRole::Member)));
        mock_repo
            .expect_get_organization()
            .withf(|org_id| *org_id == 2)
            .times(1)
            .returning(|_| {
                Ok(Some(Organization {
                    id: 2,
                    github_id: None,
                    login: "org_two".into(),
                    description: None,
                }))
            });

        mock_repo
            .expect_get_user_role()
            .withf(|org_id, uid| *org_id == 3 && *uid == 1)
            .times(1)
            .returning(|_, _| Ok(Some(OrgRole::Owner)));
        mock_repo
            .expect_get_organization()
            .withf(|org_id| *org_id == 3)
            .times(1)
            .returning(|_| Ok(None));

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
            .uri("/orgs/1")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_forbidden = test::call_service(&app, req_forbidden).await;
        assert_eq!(resp_forbidden.status(), StatusCode::FORBIDDEN);

        // Member -> 200 OK
        let req_ok = test::TestRequest::get()
            .uri("/orgs/2")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_ok = test::call_service(&app, req_ok).await;
        assert_eq!(resp_ok.status(), StatusCode::OK);
        let body = to_bytes(resp_ok.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let org: Organization = serde_json::from_slice(&body)?;
        assert_eq!(org.login, "org_two");

        // Missing org -> 404 Not Found
        let req_nf = test::TestRequest::get()
            .uri("/orgs/3")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_nf = test::call_service(&app, req_nf).await;
        assert_eq!(resp_nf.status(), StatusCode::NOT_FOUND);

        Ok(())
    }

    #[actix_web::test]
    async fn test_org_db_errors() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_create_organization_with_owner()
            .times(1)
            .returning(|_, _, _| Err(Error::InternalError));
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
        let req_post = test::TestRequest::post()
            .uri("/orgs")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateOrgPayload {
                login: "test_org".into(),
                description: None,
            })
            .to_request();
        let resp_post = test::call_service(&app, req_post).await;
        assert_eq!(resp_post.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let req_get = test::TestRequest::get()
            .uri("/orgs/1")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_get = test::call_service(&app, req_get).await;
        assert_eq!(resp_get.status(), StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }
}
