//! Repositories API endpoints.

use crate::api::auth_middleware::AuthenticatedUser;
use crate::db::repository::CddRepository;
use actix_web::{web, HttpResponse};
use std::sync::Arc;

/// Payload to create a repository
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateRepoPayload {
    /// Repository name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
}

/// Create a repository
pub async fn create_repo(
    user: AuthenticatedUser,
    path: web::Path<i32>,
    payload: web::Json<CreateRepoPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, crate::error::Error> {
    let org_id = path.into_inner();
    let role = repo.get_user_role(org_id, user.user_id).await?;
    if role.is_none() {
        return Err(crate::error::Error::Unauthorized);
    }

    let repository = repo
        .create_repository(
            org_id,
            None,
            payload.name.clone(),
            payload.description.clone(),
        )
        .await?;
    Ok(HttpResponse::Created().json(repository))
}

/// Get a repository
pub async fn get_repo(
    user: AuthenticatedUser,
    path: web::Path<(i32, i32)>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, crate::error::Error> {
    let (org_id, repo_id) = path.into_inner();
    let role = repo.get_user_role(org_id, user.user_id).await?;
    if role.is_none() {
        return Err(crate::error::Error::Unauthorized);
    }

    match repo.get_repository(repo_id).await? {
        Some(repository) => {
            if repository.organization_id != org_id {
                return Err(crate::error::Error::NotFound("Repository not found".into()));
            }
            Ok(HttpResponse::Ok().json(repository))
        }
        None => Err(crate::error::Error::NotFound("Repository not found".into())),
    }
}

/// Configure repository routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/orgs/{org_id}/repos")
            .route("", web::post().to(create_repo))
            .route("/{repo_id}", web::get().to(get_repo)),
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::auth_middleware::generate_test_token;
    use crate::db::models::Repository;
    use crate::db::repository::MockCddRepository;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_create_repo_unauthorized() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo.expect_get_user_role().returning(|_, _| Ok(None));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/orgs/1/repos")
            .insert_header(("Authorization", format!("Bearer {}", generate_test_token())))
            .set_json(CreateRepoPayload {
                name: "test".into(),
                description: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
        Ok(())
    }

    #[actix_web::test]
    async fn test_create_repo_authorized() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_get_user_role()
            .returning(|_, _| Ok(Some("member".into())));
        mock_repo
            .expect_create_repository()
            .returning(|_, _, _, _| {
                Ok(Repository {
                    id: 1,
                    organization_id: 1,
                    github_id: None,
                    name: "test".into(),
                    description: None,
                })
            });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/orgs/1/repos")
            .insert_header(("Authorization", format!("Bearer {}", generate_test_token())))
            .set_json(CreateRepoPayload {
                name: "test".into(),
                description: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);
        Ok(())
    }

    #[actix_web::test]
    async fn test_get_repo_unauthorized() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo.expect_get_user_role().returning(|_, _| Ok(None));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/orgs/1/repos/1")
            .insert_header(("Authorization", format!("Bearer {}", generate_test_token())))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
        Ok(())
    }

    #[actix_web::test]
    async fn test_get_repo_authorized() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_get_user_role()
            .returning(|_, _| Ok(Some("member".into())));
        mock_repo.expect_get_repository().returning(|_| {
            Ok(Some(Repository {
                id: 1,
                organization_id: 1,
                github_id: None,
                name: "test".into(),
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
            .uri("/orgs/1/repos/1")
            .insert_header(("Authorization", format!("Bearer {}", generate_test_token())))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
        Ok(())
    }

    #[actix_web::test]
    async fn test_get_repo_mismatch_org() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_get_user_role()
            .returning(|_, _| Ok(Some("member".into())));
        mock_repo.expect_get_repository().returning(|_| {
            Ok(Some(Repository {
                id: 1,
                organization_id: 2,
                github_id: None,
                name: "test".into(),
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
            .uri("/orgs/1/repos/1")
            .insert_header(("Authorization", format!("Bearer {}", generate_test_token())))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
        Ok(())
    }

    #[actix_web::test]
    async fn test_get_repo_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_get_user_role()
            .returning(|_, _| Ok(Some("member".into())));
        mock_repo.expect_get_repository().returning(|_| Ok(None));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/orgs/1/repos/1")
            .insert_header(("Authorization", format!("Bearer {}", generate_test_token())))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
        Ok(())
    }
}
