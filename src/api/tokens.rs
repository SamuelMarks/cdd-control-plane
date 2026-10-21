//! API endpoints for managing encrypted user registry and VCS tokens.

use crate::api::auth_middleware::AuthenticatedUser;
use crate::config::AppConfig;
use crate::crypto::encrypt_local_secret;
use crate::db::repository::CddRepository;
use crate::domain::TokenProvider;
use crate::error::Error;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use utoipa::ToSchema;

/// Request payload for storing or rotating a user token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct StoreTokenPayload {
    /// Target registry/VCS provider (`github`, `npm`, `pypi`, `cargo`).
    pub provider: String,
    /// Plaintext token value to encrypt and store at rest.
    pub token: String,
}

/// Encrypts a user token with [`AppConfig::encryption_key`] and upserts it for the caller.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `payload` - JSON [`StoreTokenPayload`].
/// * `repo` - Database repository handle.
/// * `cfg` - Application configuration containing the symmetric `encryption_key`.
#[utoipa::path(
    post,
    path = "/tokens",
    request_body = StoreTokenPayload,
    responses(
        (status = 201, description = "Token encrypted and stored"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn store_token(
    user: AuthenticatedUser,
    payload: web::Json<StoreTokenPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
    cfg: web::Data<AppConfig>,
) -> Result<HttpResponse, Error> {
    let provider = TokenProvider::from_str(&payload.provider)?;
    if payload.token.trim().is_empty() {
        return Err(Error::BadRequest("Token value cannot be empty".into()));
    }

    let encrypted = encrypt_local_secret(&cfg.encryption_key, &payload.token)?;
    repo.upsert_user_token(user.user_id, provider, encrypted)
        .await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"status": "success"})))
}

/// Lists the provider names for which the authenticated user has stored tokens.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `repo` - Database repository handle.
#[utoipa::path(
    get,
    path = "/tokens",
    responses(
        (status = 200, description = "List of configured token providers", body = [String]),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_tokens(
    user: AuthenticatedUser,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let tokens = repo.list_user_tokens(user.user_id).await?;
    let providers: Vec<String> = tokens.into_iter().map(|t| t.provider).collect();
    Ok(HttpResponse::Ok().json(providers))
}

/// Deletes a stored token for the authenticated user and specified provider.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Provider name path parameter (`github`, `npm`, `pypi`, `cargo`).
/// * `repo` - Database repository handle.
#[utoipa::path(
    delete,
    path = "/tokens/{provider}",
    params(
        ("provider" = String, Path, description = "Token provider name")
    ),
    responses(
        (status = 200, description = "Token deleted"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn delete_token(
    user: AuthenticatedUser,
    path: web::Path<String>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let provider = TokenProvider::from_str(&path.into_inner())?;
    repo.delete_user_token(user.user_id, provider).await?;
    Ok(HttpResponse::Ok().finish())
}

/// Configures the `/tokens` routes on the provided [`web::ServiceConfig`].
///
/// # Arguments
///
/// * `cfg` - Actix service configuration builder.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tokens")
            .route("", web::post().to(store_token))
            .route("", web::get().to(list_tokens))
            .route("/{provider}", web::delete().to(delete_token)),
    );
}

#[cfg(test)]
mod tests {
    //! Unit tests for token encryption, storage, listing, and deletion.

    use super::*;
    use crate::api::auth_middleware::{generate_test_token, test_app_config};
    use crate::crypto::decrypt_local_secret;
    use crate::db::models::UserToken;
    use crate::db::repository::MockCddRepository;
    use actix_web::{body::to_bytes, http::StatusCode, test, App};

    #[actix_web::test]
    async fn test_store_list_and_delete_tokens() -> Result<(), Error> {
        let cfg = test_app_config()?;
        let expected_enc_key = cfg.encryption_key.clone();

        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_upsert_user_token()
            .withf(move |uid, provider, enc| {
                *uid == 1
                    && *provider == TokenProvider::Npm
                    && decrypt_local_secret(&expected_enc_key, enc)
                        .is_ok_and(|plain| plain == "npm_secret_token")
            })
            .times(1)
            .returning(|user_id, provider, encrypted_token| {
                Ok(UserToken {
                    id: 1,
                    user_id,
                    provider: provider.as_str().to_string(),
                    encrypted_token,
                    created_at: chrono::Utc::now().naive_utc(),
                    updated_at: chrono::Utc::now().naive_utc(),
                })
            });

        mock_repo
            .expect_list_user_tokens()
            .withf(|uid| *uid == 1)
            .times(1)
            .returning(|_| {
                Ok(vec![UserToken {
                    id: 1,
                    user_id: 1,
                    provider: "npm".into(),
                    encrypted_token: "encrypted".into(),
                    created_at: chrono::Utc::now().naive_utc(),
                    updated_at: chrono::Utc::now().naive_utc(),
                }])
            });

        mock_repo
            .expect_delete_user_token()
            .withf(|uid, provider| *uid == 1 && *provider == TokenProvider::Npm)
            .times(1)
            .returning(|_, _| Ok(()));

        let app = test::init_service(
            App::new()
                .app_data(cfg)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let token = generate_test_token()?;

        // Store valid token
        let req_store = test::TestRequest::post()
            .uri("/tokens")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(StoreTokenPayload {
                provider: "npm".into(),
                token: "npm_secret_token".into(),
            })
            .to_request();
        let resp_store = test::call_service(&app, req_store).await;
        assert_eq!(resp_store.status(), StatusCode::CREATED);

        // Reject invalid provider or empty token
        let req_bad_prov = test::TestRequest::post()
            .uri("/tokens")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(StoreTokenPayload {
                provider: "unknown".into(),
                token: "npm_secret_token".into(),
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_bad_prov).await.status(),
            StatusCode::BAD_REQUEST
        );

        let req_empty_tok = test::TestRequest::post()
            .uri("/tokens")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(StoreTokenPayload {
                provider: "npm".into(),
                token: "   ".into(),
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_empty_tok).await.status(),
            StatusCode::BAD_REQUEST
        );

        // List tokens and verify payload
        let req_list = test::TestRequest::get()
            .uri("/tokens")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_list = test::call_service(&app, req_list).await;
        assert_eq!(resp_list.status(), StatusCode::OK);
        let list_bytes = to_bytes(resp_list.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let providers: Vec<String> = serde_json::from_slice(&list_bytes)?;
        assert_eq!(providers, vec!["npm".to_string()]);

        // Delete token
        let req_del = test::TestRequest::delete()
            .uri("/tokens/npm")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_del).await.status(),
            StatusCode::OK
        );

        // Delete with invalid provider -> 400
        let req_del_bad = test::TestRequest::delete()
            .uri("/tokens/invalid_provider")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_del_bad).await.status(),
            StatusCode::BAD_REQUEST
        );

        Ok(())
    }

    #[actix_web::test]
    async fn test_store_token_db_error() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_upsert_user_token()
            .times(1)
            .returning(|_, _, _| Err(Error::InternalError));

        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let token = generate_test_token()?;
        let req = test::TestRequest::post()
            .uri("/tokens")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(StoreTokenPayload {
                provider: "npm".into(),
                token: "my_secret_token".into(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }
}
