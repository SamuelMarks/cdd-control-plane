//! Repository and SDK Release management API endpoints.

use crate::api::auth_middleware::AuthenticatedUser;
use crate::db::models::{Release, Repository};
use crate::db::repository::CddRepository;
use crate::domain::{AuditAction, Username};
use crate::error::Error;
use crate::queue::{JobQueue, ReleaseSdkJob};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// Request payload for creating a new [`Repository`] inside an organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreateRepoPayload {
    /// Repository name (`1..=64` alphanumeric, `-`, `_`).
    pub name: String,
    /// Optional human-readable description.
    pub description: Option<String>,
}

/// Request payload for creating a new SDK [`Release`] and enqueuing a publish job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreateReleasePayload {
    /// Non-empty Git tag name (e.g., `v1.0.0`).
    pub tag_name: String,
    /// Optional release title.
    pub name: Option<String>,
    /// Optional release notes / changelog markdown body.
    pub body: Option<String>,
}

/// Creates a new [`Repository`] inside `org_id` if the caller holds `owner` or `admin` privileges.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Organization primary key ID.
/// * `payload` - JSON [`CreateRepoPayload`].
/// * `repo` - Database repository handle.
#[utoipa::path(
    post,
    path = "/orgs/{org_id}/repos",
    params(
        ("org_id" = i32, Path, description = "Organization ID")
    ),
    request_body = CreateRepoPayload,
    responses(
        (status = 201, description = "Repository created", body = Repository),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn create_repo(
    user: AuthenticatedUser,
    path: web::Path<i32>,
    payload: web::Json<CreateRepoPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let org_id = path.into_inner();
    let repo_name = Username::parse(&payload.name)?;

    let Some(role) = repo.get_user_role(org_id, user.user_id).await? else {
        return Err(Error::Forbidden(
            "User is not a member of this organization".into(),
        ));
    };
    if !role.can_write_repos() {
        return Err(Error::Forbidden(
            "Insufficient permissions to create a repository".into(),
        ));
    }

    let repository = repo
        .create_repository(
            org_id,
            None,
            repo_name.as_str().to_string(),
            payload.description.clone(),
        )
        .await?;

    repo.create_audit_log(
        org_id,
        Some(repository.id),
        user.user_id,
        AuditAction::RepoCreated,
        Some(serde_json::json!({ "name": repository.name })),
    )
    .await?;

    Ok(HttpResponse::Created().json(repository))
}

/// Lists all repositories in `org_id` if the caller is a member of the organization.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Organization primary key ID.
/// * `repo` - Database repository handle.
#[utoipa::path(
    get,
    path = "/orgs/{org_id}/repos",
    params(
        ("org_id" = i32, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "List of repositories", body = [Repository]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn list_repos(
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

    let repos = repo.list_repositories_for_org(org_id).await?;
    Ok(HttpResponse::Ok().json(repos))
}

/// Retrieves a single [`Repository`] by `(org_id, repo_id)` if the caller is an organization member.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Tuple of `(org_id, repo_id)`.
/// * `repo` - Database repository handle.
#[utoipa::path(
    get,
    path = "/orgs/{org_id}/repos/{repo_id}",
    params(
        ("org_id" = i32, Path, description = "Organization ID"),
        ("repo_id" = i32, Path, description = "Repository ID")
    ),
    responses(
        (status = 200, description = "Repository details", body = Repository),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found")
    )
)]
pub async fn get_repo(
    user: AuthenticatedUser,
    path: web::Path<(i32, i32)>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let (org_id, repo_id) = path.into_inner();
    let Some(role) = repo.get_user_role(org_id, user.user_id).await? else {
        return Err(Error::Forbidden(
            "User is not a member of this organization".into(),
        ));
    };
    debug_assert!(role.can_read());

    match repo.get_repository(repo_id).await? {
        Some(repository) if repository.organization_id == org_id => {
            Ok(HttpResponse::Ok().json(repository))
        }
        _ => Err(Error::NotFound("Repository not found".into())),
    }
}

/// Creates a new SDK [`Release`], records an audit log, and enqueues a [`ReleaseSdkJob`] if a [`JobQueue`] is configured.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Tuple of `(org_id, repo_id)`.
/// * `payload` - JSON [`CreateReleasePayload`].
/// * `repo` - Database repository handle.
/// * `queue` - Optional background job queue handle.
#[utoipa::path(
    post,
    path = "/orgs/{org_id}/repos/{repo_id}/releases",
    params(
        ("org_id" = i32, Path, description = "Organization ID"),
        ("repo_id" = i32, Path, description = "Repository ID")
    ),
    request_body = CreateReleasePayload,
    responses(
        (status = 201, description = "Release created and queued", body = Release),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found")
    )
)]
pub async fn create_release(
    user: AuthenticatedUser,
    path: web::Path<(i32, i32)>,
    payload: web::Json<CreateReleasePayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
    queue: Option<web::Data<Arc<dyn JobQueue>>>,
) -> Result<HttpResponse, Error> {
    let (org_id, repo_id) = path.into_inner();
    let tag_trimmed = payload.tag_name.trim();
    if tag_trimmed.is_empty() {
        return Err(Error::BadRequest("Release tag_name cannot be empty".into()));
    }

    let Some(role) = repo.get_user_role(org_id, user.user_id).await? else {
        return Err(Error::Forbidden(
            "User is not a member of this organization".into(),
        ));
    };
    if !role.can_write_repos() {
        return Err(Error::Forbidden(
            "Insufficient permissions to create a release".into(),
        ));
    }

    let Some(repository) = repo.get_repository(repo_id).await? else {
        return Err(Error::NotFound("Repository not found".into()));
    };
    if repository.organization_id != org_id {
        return Err(Error::NotFound("Repository not found".into()));
    }

    let release = repo
        .create_release(
            repo_id,
            None,
            tag_trimmed.to_string(),
            payload.name.clone(),
            payload.body.clone(),
        )
        .await?;

    repo.create_audit_log(
        org_id,
        Some(repo_id),
        user.user_id,
        AuditAction::ReleaseCreated,
        Some(serde_json::json!({ "tag_name": release.tag_name })),
    )
    .await?;

    if let Some(q) = queue {
        q.enqueue_release_sdk(&ReleaseSdkJob {
            release_id: release.id,
            org_id,
            repo_id,
            tag_name: release.tag_name.clone(),
        })
        .await?;
    }

    Ok(HttpResponse::Created().json(release))
}

/// Lists all SDK releases for a repository.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Tuple of `(org_id, repo_id)`.
/// * `repo` - Database repository handle.
#[utoipa::path(
    get,
    path = "/orgs/{org_id}/repos/{repo_id}/releases",
    params(
        ("org_id" = i32, Path, description = "Organization ID"),
        ("repo_id" = i32, Path, description = "Repository ID")
    ),
    responses(
        (status = 200, description = "List of releases", body = [Release]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found")
    )
)]
pub async fn list_releases(
    user: AuthenticatedUser,
    path: web::Path<(i32, i32)>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let (org_id, repo_id) = path.into_inner();
    let Some(role) = repo.get_user_role(org_id, user.user_id).await? else {
        return Err(Error::Forbidden(
            "User is not a member of this organization".into(),
        ));
    };
    debug_assert!(role.can_read());

    let Some(repository) = repo.get_repository(repo_id).await? else {
        return Err(Error::NotFound("Repository not found".into()));
    };
    if repository.organization_id != org_id {
        return Err(Error::NotFound("Repository not found".into()));
    }

    let releases = repo.list_releases_for_repo(repo_id).await?;
    Ok(HttpResponse::Ok().json(releases))
}

/// Configures the `/orgs/{org_id}/repos` routes on the provided [`web::ServiceConfig`].
///
/// # Arguments
///
/// * `cfg` - Actix service configuration builder.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/orgs/{org_id}/repos")
            .route("", web::post().to(create_repo))
            .route("", web::get().to(list_repos))
            .route("/{repo_id}", web::get().to(get_repo))
            .route("/{repo_id}/releases", web::post().to(create_release))
            .route("/{repo_id}/releases", web::get().to(list_releases)),
    );
}

#[cfg(test)]
mod tests {
    //! Unit tests for repository and SDK release endpoints.

    use super::*;
    use crate::api::auth_middleware::{generate_test_token, test_app_config};
    use crate::db::models::AuditLog;
    use crate::db::repository::MockCddRepository;
    use crate::domain::OrgRole;
    use crate::queue::MockJobQueue;
    use actix_web::{body::to_bytes, http::StatusCode, test, App};

    #[actix_web::test]
    async fn test_create_and_list_repos_rbac() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();

        // Org 1: non-member -> 403
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 1)
            .times(2)
            .returning(|_, _| Ok(None));

        // Org 2: member (read-only) -> 403 on POST, 200 on GET
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 2)
            .times(2)
            .returning(|_, _| Ok(Some(OrgRole::Member)));
        mock_repo
            .expect_list_repositories_for_org()
            .withf(|org_id| *org_id == 2)
            .times(1)
            .returning(|_| {
                Ok(vec![Repository {
                    id: 5,
                    organization_id: 2,
                    github_id: None,
                    name: "existing_repo".into(),
                    description: None,
                }])
            });

        // Org 3: admin -> 201 on POST + audit log
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 3)
            .times(1)
            .returning(|_, _| Ok(Some(OrgRole::Admin)));
        mock_repo
            .expect_create_repository()
            .withf(|org_id, gh_id, name, _| *org_id == 3 && gh_id.is_none() && name == "sdk_repo")
            .times(1)
            .returning(|org_id, github_id, name, description| {
                Ok(Repository {
                    id: 9,
                    organization_id: org_id,
                    github_id,
                    name,
                    description,
                })
            });
        mock_repo
            .expect_create_audit_log()
            .withf(|org_id, repo_id, uid, action, _| {
                *org_id == 3
                    && *repo_id == Some(9)
                    && *uid == 1
                    && *action == AuditAction::RepoCreated
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

        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let token = generate_test_token()?;

        let req_non_member = test::TestRequest::post()
            .uri("/orgs/1/repos")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateRepoPayload {
                name: "sdk_repo".into(),
                description: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_non_member).await.status(),
            StatusCode::FORBIDDEN
        );

        let req_list_non_member = test::TestRequest::get()
            .uri("/orgs/1/repos")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_list_non_member).await.status(),
            StatusCode::FORBIDDEN
        );

        let req_member_post = test::TestRequest::post()
            .uri("/orgs/2/repos")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateRepoPayload {
                name: "sdk_repo".into(),
                description: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_member_post).await.status(),
            StatusCode::FORBIDDEN
        );

        let req_member_list = test::TestRequest::get()
            .uri("/orgs/2/repos")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_list = test::call_service(&app, req_member_list).await;
        assert_eq!(resp_list.status(), StatusCode::OK);
        let list_bytes = to_bytes(resp_list.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let repos: Vec<Repository> = serde_json::from_slice(&list_bytes)?;
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "existing_repo");

        let req_admin_post = test::TestRequest::post()
            .uri("/orgs/3/repos")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateRepoPayload {
                name: "sdk_repo".into(),
                description: Some("SDK".into()),
            })
            .to_request();
        let resp_created = test::call_service(&app, req_admin_post).await;
        assert_eq!(resp_created.status(), StatusCode::CREATED);
        let created_bytes = to_bytes(resp_created.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let created_repo: Repository = serde_json::from_slice(&created_bytes)?;
        assert_eq!(created_repo.id, 9);
        assert_eq!(created_repo.name, "sdk_repo");

        Ok(())
    }

    #[actix_web::test]
    async fn test_get_repo_branches() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 1)
            .times(1)
            .returning(|_, _| Ok(None));

        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 2)
            .times(3)
            .returning(|_, _| Ok(Some(OrgRole::Member)));

        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 10)
            .times(1)
            .returning(|_| {
                Ok(Some(Repository {
                    id: 10,
                    organization_id: 2,
                    github_id: None,
                    name: "valid_repo".into(),
                    description: None,
                }))
            });

        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 11)
            .times(1)
            .returning(|_| {
                Ok(Some(Repository {
                    id: 11,
                    organization_id: 99,
                    github_id: None,
                    name: "other_org_repo".into(),
                    description: None,
                }))
            });

        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 12)
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

        let req_unauth = test::TestRequest::get()
            .uri("/orgs/1/repos/10")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_unauth).await.status(),
            StatusCode::FORBIDDEN
        );

        let req_ok = test::TestRequest::get()
            .uri("/orgs/2/repos/10")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_ok = test::call_service(&app, req_ok).await;
        assert_eq!(resp_ok.status(), StatusCode::OK);
        let body = to_bytes(resp_ok.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let repo_item: Repository = serde_json::from_slice(&body)?;
        assert_eq!(repo_item.name, "valid_repo");

        let req_mismatch = test::TestRequest::get()
            .uri("/orgs/2/repos/11")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_mismatch).await.status(),
            StatusCode::NOT_FOUND
        );

        let req_nf = test::TestRequest::get()
            .uri("/orgs/2/repos/12")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_nf).await.status(),
            StatusCode::NOT_FOUND
        );

        Ok(())
    }

    #[actix_web::test]
    async fn test_create_and_list_releases_with_job_queue() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        let mut mock_queue = MockJobQueue::new();

        // Org 1: non-member -> 403
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 1)
            .times(2)
            .returning(|_, _| Ok(None));

        // Org 2: member -> 403 on POST release, 200 on GET releases
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 2)
            .times(4)
            .returning(|_, _| Ok(Some(OrgRole::Member)));

        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 50)
            .times(1)
            .returning(|_| {
                Ok(Some(Repository {
                    id: 50,
                    organization_id: 2,
                    github_id: None,
                    name: "r50".into(),
                    description: None,
                }))
            });
        mock_repo
            .expect_list_releases_for_repo()
            .withf(|rid| *rid == 50)
            .times(1)
            .returning(|_| {
                Ok(vec![Release {
                    id: 100,
                    repository_id: 50,
                    github_id: None,
                    tag_name: "v1.0.0".into(),
                    name: Some("Initial".into()),
                    body: None,
                }])
            });

        // Repo mismatch & missing on list_releases
        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 51)
            .times(1)
            .returning(|_| {
                Ok(Some(Repository {
                    id: 51,
                    organization_id: 999,
                    github_id: None,
                    name: "r51".into(),
                    description: None,
                }))
            });
        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 52)
            .times(1)
            .returning(|_| Ok(None));

        // Org 3: owner -> 201 on POST release + queue job
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 3)
            .times(3)
            .returning(|_, _| Ok(Some(OrgRole::Owner)));

        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 60)
            .times(1)
            .returning(|_| {
                Ok(Some(Repository {
                    id: 60,
                    organization_id: 3,
                    github_id: None,
                    name: "r60".into(),
                    description: None,
                }))
            });
        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 61)
            .times(1)
            .returning(|_| {
                Ok(Some(Repository {
                    id: 61,
                    organization_id: 999,
                    github_id: None,
                    name: "r61".into(),
                    description: None,
                }))
            });
        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 62)
            .times(1)
            .returning(|_| Ok(None));

        mock_repo
            .expect_create_release()
            .withf(|rid, _, tag, _, _| *rid == 60 && tag == "v2.0.0")
            .times(1)
            .returning(|repository_id, github_id, tag_name, name, body| {
                Ok(Release {
                    id: 200,
                    repository_id,
                    github_id,
                    tag_name,
                    name,
                    body,
                })
            });

        mock_repo
            .expect_create_audit_log()
            .withf(|org_id, repo_id, _, action, _| {
                *org_id == 3 && *repo_id == Some(60) && *action == AuditAction::ReleaseCreated
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

        mock_queue
            .expect_enqueue_release_sdk()
            .withf(|job| {
                job.release_id == 200
                    && job.org_id == 3
                    && job.repo_id == 60
                    && job.tag_name == "v2.0.0"
            })
            .times(1)
            .returning(|_| Ok(()));

        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(web::Data::new(Arc::new(mock_queue) as Arc<dyn JobQueue>))
                .configure(configure),
        )
        .await;

        let token = generate_test_token()?;

        // Empty tag_name -> 400 Bad Request
        let req_empty_tag = test::TestRequest::post()
            .uri("/orgs/3/repos/60/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "   ".into(),
                name: None,
                body: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_empty_tag).await.status(),
            StatusCode::BAD_REQUEST
        );

        // Non-member POST & GET -> 403
        let req_nm_post = test::TestRequest::post()
            .uri("/orgs/1/repos/60/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "v1.0.0".into(),
                name: None,
                body: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_nm_post).await.status(),
            StatusCode::FORBIDDEN
        );

        let req_nm_get = test::TestRequest::get()
            .uri("/orgs/1/repos/60/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_nm_get).await.status(),
            StatusCode::FORBIDDEN
        );

        // Read-only member POST -> 403
        let req_mem_post = test::TestRequest::post()
            .uri("/orgs/2/repos/50/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "v1.0.0".into(),
                name: None,
                body: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_mem_post).await.status(),
            StatusCode::FORBIDDEN
        );

        // Member GET releases -> 200 OK
        let req_mem_get = test::TestRequest::get()
            .uri("/orgs/2/repos/50/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_mem_get = test::call_service(&app, req_mem_get).await;
        assert_eq!(resp_mem_get.status(), StatusCode::OK);
        let rel_bytes = to_bytes(resp_mem_get.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let releases: Vec<Release> = serde_json::from_slice(&rel_bytes)?;
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].tag_name, "v1.0.0");

        // Member GET releases mismatch & missing -> 404
        let req_get_mismatch = test::TestRequest::get()
            .uri("/orgs/2/repos/51/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_get_mismatch).await.status(),
            StatusCode::NOT_FOUND
        );
        let req_get_missing = test::TestRequest::get()
            .uri("/orgs/2/repos/52/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_get_missing).await.status(),
            StatusCode::NOT_FOUND
        );

        // Owner POST release mismatch & missing -> 404
        let req_post_mismatch = test::TestRequest::post()
            .uri("/orgs/3/repos/61/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "v2.0.0".into(),
                name: None,
                body: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_post_mismatch).await.status(),
            StatusCode::NOT_FOUND
        );
        let req_post_missing = test::TestRequest::post()
            .uri("/orgs/3/repos/62/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "v2.0.0".into(),
                name: None,
                body: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_post_missing).await.status(),
            StatusCode::NOT_FOUND
        );

        // Owner POST release valid -> 201 Created + queued
        let req_owner_post = test::TestRequest::post()
            .uri("/orgs/3/repos/60/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "v2.0.0".into(),
                name: Some("Major".into()),
                body: Some("Notes".into()),
            })
            .to_request();
        let resp_owner = test::call_service(&app, req_owner_post).await;
        assert_eq!(resp_owner.status(), StatusCode::CREATED);
        let created_bytes = to_bytes(resp_owner.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let created_rel: Release = serde_json::from_slice(&created_bytes)?;
        assert_eq!(created_rel.id, 200);
        assert_eq!(created_rel.tag_name, "v2.0.0");

        Ok(())
    }
}
