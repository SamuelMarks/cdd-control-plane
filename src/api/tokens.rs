//! API endpoints for managing user tokens (GitHub OAuth, NPM, PyPI, etc.).

use crate::api::auth_middleware::AuthenticatedUser;
use crate::config::AppConfig;
use crate::crypto::encrypt_local_secret;
use crate::db::repository::CddRepository;
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;

/// Payload for storing a user token.
#[derive(Deserialize)]
pub struct StoreTokenPayload {
    /// Provider (e.g., 'github', 'npm', 'pypi', 'cargo')
    pub provider: String,
    /// The token value to store
    pub token: String,
}

/// Store a token securely for the authenticated user.
pub async fn store_token(
    user: AuthenticatedUser,
    payload: web::Json<StoreTokenPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
    cfg: web::Data<AppConfig>,
) -> Result<HttpResponse, crate::error::Error> {
    let encrypted = encrypt_local_secret(&cfg.webhook_secret, &payload.token)?;
    repo.upsert_user_token(user.user_id, payload.provider.clone(), encrypted)
        .await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"status": "success"})))
}

/// Get a list of providers for which the user has stored tokens.
pub async fn list_tokens(
    user: AuthenticatedUser,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, crate::error::Error> {
    let tokens = repo.list_user_tokens(user.user_id).await?;
    let providers: Vec<String> = tokens.into_iter().map(|t| t.provider).collect();
    Ok(HttpResponse::Ok().json(providers))
}

/// Delete a stored token by provider.
pub async fn delete_token(
    user: AuthenticatedUser,
    path: web::Path<String>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, crate::error::Error> {
    let provider = path.into_inner();
    repo.delete_user_token(user.user_id, provider).await?;
    Ok(HttpResponse::Ok().finish())
}

/// Configure token routes.
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
    use super::*;
    use crate::api::auth_middleware::generate_test_token;
    use crate::db::models::UserToken;
    use crate::db::repository::MockCddRepository;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_store_token() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo.expect_upsert_user_token().returning(|_, _, _| {
            Ok(UserToken {
                id: 1,
                user_id: 1,
                provider: "npm".into(),
                encrypted_token: "encrypted".into(),
                created_at: chrono::Utc::now().naive_utc(),
                updated_at: chrono::Utc::now().naive_utc(),
            })
        });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(web::Data::new(AppConfig::load(None)?))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/tokens")
            .insert_header((
                "Authorization",
                format!("Bearer {}", generate_test_token()?),
            ))
            .set_json(serde_json::json!({ "provider": "npm", "token": "my_secret_token" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);
        Ok(())
    }

    #[actix_web::test]
    async fn test_list_tokens() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo.expect_list_user_tokens().returning(|_| {
            Ok(vec![UserToken {
                id: 1,
                user_id: 1,
                provider: "npm".into(),
                encrypted_token: "encrypted".into(),
                created_at: chrono::Utc::now().naive_utc(),
                updated_at: chrono::Utc::now().naive_utc(),
            }])
        });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/tokens")
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
    async fn test_delete_token() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_delete_user_token()
            .returning(|_, _| Ok(()));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri("/tokens/npm")
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
