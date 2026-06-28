//! Organizations API endpoints.

use crate::api::auth_middleware::AuthenticatedUser;
use crate::db::repository::CddRepository;
use actix_web::{web, HttpResponse};
use std::sync::Arc;

/// Payload to create an organization
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateOrgPayload {
    /// Organization login name
    pub login: String,
    /// Optional description
    pub description: Option<String>,
}

/// Create an organization
pub async fn create_org(
    user: AuthenticatedUser,
    payload: web::Json<CreateOrgPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, crate::error::Error> {
    let org = repo
        .create_organization(None, payload.login.clone(), payload.description.clone())
        .await?;
    repo.add_user_to_organization(org.id, user.user_id, "owner".to_string())
        .await?;
    Ok(HttpResponse::Created().json(org))
}

/// Get an organization
pub async fn get_org(
    user: AuthenticatedUser,
    path: web::Path<i32>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, crate::error::Error> {
    let org_id = path.into_inner();
    let role = repo.get_user_role(org_id, user.user_id).await?;
    if role.is_none() {
        return Err(crate::error::Error::Unauthorized);
    }

    match repo.get_organization(org_id).await? {
        Some(org) => Ok(HttpResponse::Ok().json(org)),
        None => Err(crate::error::Error::NotFound(
            "Organization not found".into(),
        )),
    }
}

/// Configure organization routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/orgs")
            .route("", web::post().to(create_org))
            .route("/{id}", web::get().to(get_org)),
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::auth_middleware::generate_test_token;
    use crate::db::models::Organization;
    use crate::db::repository::MockCddRepository;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_create_org() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo.expect_create_organization().returning(|_, _, _| {
            Ok(Organization {
                id: 1,
                github_id: None,
                login: "test".into(),
                description: None,
            })
        });
        mock_repo
            .expect_add_user_to_organization()
            .returning(|_, _, _| {
                Ok(crate::db::models::OrganizationUser {
                    organization_id: 1,
                    user_id: 1,
                    role: "owner".into(),
                })
            });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/orgs")
            .insert_header((
                "Authorization",
                format!("Bearer {}", generate_test_token()?),
            ))
            .set_json(CreateOrgPayload {
                login: "test".into(),
                description: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);
        Ok(())
    }

    #[actix_web::test]
    async fn test_get_org_unauthorized() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo.expect_get_user_role().returning(|_, _| Ok(None));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/orgs/1")
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
    async fn test_get_org_authorized() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_get_user_role()
            .returning(|_, _| Ok(Some("member".into())));
        mock_repo.expect_get_organization().returning(|_| {
            Ok(Some(Organization {
                id: 1,
                github_id: None,
                login: "test".into(),
                description: None,
            }))
        });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/orgs/1")
            .insert_header((
                "Authorization",
                format!("Bearer {}", generate_test_token()?),
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
        Ok(())
    }

    #[actix_web::test]
    async fn test_get_org_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_get_user_role()
            .returning(|_, _| Ok(Some("member".into())));
        mock_repo.expect_get_organization().returning(|_| Ok(None));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/orgs/1")
            .insert_header((
                "Authorization",
                format!("Bearer {}", generate_test_token()?),
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
        Ok(())
    }

    #[actix_web::test]
    async fn test_create_org_db_error() -> Result<(), Box<dyn std::error::Error>> {
        use diesel::result::Error;
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_create_organization()
            .returning(|_, _, _| Err(Error::NotFound));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/orgs")
            .insert_header((
                "Authorization",
                format!("Bearer {}", generate_test_token()?),
            ))
            .set_json(CreateOrgPayload {
                login: "test".into(),
                description: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        Ok(())
    }

    #[actix_web::test]
    async fn test_get_org_db_error() -> Result<(), Box<dyn std::error::Error>> {
        use diesel::result::Error;
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_get_user_role()
            .returning(|_, _| Err(Error::NotFound));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/orgs/1")
            .insert_header((
                "Authorization",
                format!("Bearer {}", generate_test_token()?),
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        Ok(())
    }
}
