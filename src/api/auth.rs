//! Authentication endpoints for user registration, Argon2 password verification, and JWT issuance.

use crate::api::auth_middleware::Claims;
use crate::config::AppConfig;
use crate::db::repository::CddRepository;
use crate::domain::{Email, Password, Username};
use crate::error::Error;
use actix_web::{web, HttpResponse};
use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// Pre-computed dummy Argon2id PHC hash used to equalize timing when a username does not exist.
const DUMMY_PASSWORD_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHR2YWx1ZQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG";

/// Response payload containing a signed Bearer JWT.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    /// Signed JWT token string.
    pub token: String,
}

/// Request payload for authenticating with username and password.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct LoginPayload {
    /// Account username.
    pub username: String,
    /// Plaintext password.
    pub password: Option<String>,
}

/// Request payload for registering a new user account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RegisterPayload {
    /// Desired unique username (`1..=64` alphanumeric, `-`, `_`).
    pub username: String,
    /// Validated email address.
    pub email: String,
    /// Optional plaintext password (if provided, must be non-empty).
    pub password: Option<String>,
}

/// Generates a signed 24-hour JWT for the given user ID and username.
///
/// # Arguments
///
/// * `user_id` - Database primary key of the user.
/// * `username` - Canonical username of the user.
/// * `jwt_secret` - HMAC secret bytes from [`AppConfig`].
fn generate_token(user_id: i32, username: &str, jwt_secret: &[u8]) -> Result<String, Error> {
    let exp = usize::try_from((chrono::Utc::now() + chrono::Duration::hours(24)).timestamp())
        .map_err(|_| Error::InternalError)?;
    let claims = Claims {
        sub: user_id,
        exp,
        username: username.to_string(),
    };
    Ok(encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret),
    )?)
}

/// Hashes a plaintext password using Argon2id on a randomly generated salt.
///
/// # Arguments
///
/// * `password` - The plaintext password to hash.
fn hash_password(password: &str) -> Result<String, Error> {
    let hash: PasswordHash = Argon2::default().hash_password(password.as_bytes())?;
    Ok(hash.to_string())
}

/// Verifies a plaintext password against an Argon2 PHC hash string.
///
/// # Arguments
///
/// * `password` - The candidate plaintext password.
/// * `hash` - The stored Argon2 PHC string.
fn verify_password(password: &str, hash: &str) -> bool {
    if let Ok(parsed_hash) = PasswordHash::new(hash) {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    } else {
        false
    }
}

/// Registers a new user account and returns a signed JWT.
///
/// # Arguments
///
/// * `payload` - JSON [`RegisterPayload`] containing username, email, and optional password.
/// * `repo` - Database repository handle.
/// * `cfg` - Application configuration containing the JWT signing secret.
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterPayload,
    responses(
        (status = 201, description = "Successfully registered", body = AuthResponse),
        (status = 400, description = "Bad Request"),
        (status = 409, description = "Conflict")
    )
)]
pub async fn register(
    payload: web::Json<RegisterPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
    cfg: web::Data<AppConfig>,
) -> Result<HttpResponse, Error> {
    let username = Username::parse(&payload.username)?;
    let email = Email::parse(&payload.email)?;

    let hashed_pw = if let Some(pw) = payload.password.as_ref() {
        let validated_pw = Password::parse(pw)?;
        let pw_owned = validated_pw.expose_secret().to_string();
        Some(web::block(move || hash_password(&pw_owned)).await??)
    } else {
        None
    };

    let user = repo
        .create_user(
            None,
            username.as_str().to_string(),
            email.as_str().to_string(),
            hashed_pw,
        )
        .await?;

    let token = generate_token(user.id, &user.username, cfg.jwt_secret.as_bytes())?;

    Ok(HttpResponse::Created().json(AuthResponse { token }))
}

/// Authenticates a user with username and password and returns a signed JWT.
///
/// Performs constant-path Argon2 verification even when the user does not exist to
/// mitigate timing-based username enumeration.
///
/// # Arguments
///
/// * `payload` - JSON [`LoginPayload`] containing username and password.
/// * `repo` - Database repository handle.
/// * `cfg` - Application configuration containing the JWT signing secret.
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginPayload,
    responses(
        (status = 200, description = "Successfully authenticated", body = AuthResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn login_password(
    payload: web::Json<LoginPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
    cfg: web::Data<AppConfig>,
) -> Result<HttpResponse, Error> {
    let Some(pw) = payload.password.clone() else {
        return Err(Error::Unauthorized);
    };

    let user_opt = repo.find_user_by_username(payload.username.clone()).await?;

    if let Some(user) = user_opt {
        let Some(stored_hash) = user.password_hash.clone() else {
            let _ = web::block(move || verify_password(&pw, DUMMY_PASSWORD_HASH)).await?;
            return Err(Error::Unauthorized);
        };
        let is_valid = web::block(move || verify_password(&pw, &stored_hash)).await?;
        if is_valid {
            let token = generate_token(user.id, &user.username, cfg.jwt_secret.as_bytes())?;
            Ok(HttpResponse::Ok().json(AuthResponse { token }))
        } else {
            Err(Error::Unauthorized)
        }
    } else {
        let _ = web::block(move || verify_password(&pw, DUMMY_PASSWORD_HASH)).await?;
        Err(Error::Unauthorized)
    }
}

/// Configures the `/auth` routes on the provided [`web::ServiceConfig`].
///
/// # Arguments
///
/// * `cfg` - Actix service configuration builder.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login_password)),
    );
}

#[cfg(test)]
mod tests {
    //! Unit tests for registration, password hashing, and login handlers.

    use super::*;
    use crate::api::auth_middleware::test_app_config;
    use crate::db::models::User;
    use crate::db::repository::MockCddRepository;
    use actix_web::{body::to_bytes, http::StatusCode, test, App};
    use jsonwebtoken::{decode, DecodingKey, Validation};

    #[actix_web::test]
    async fn test_verify_password_invalid_hash() {
        assert!(!verify_password("pwd", "invalid_hash_string"));
    }

    #[actix_web::test]
    async fn test_register_with_password_verifies_jwt_and_args() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_create_user()
            .withf(|gh_id, username, email, pw_hash| {
                gh_id.is_none()
                    && username == "test_user"
                    && email == "test@example.com"
                    && pw_hash
                        .as_deref()
                        .is_some_and(|h| h != "StrongPass123" && h.starts_with("$argon2"))
            })
            .times(1)
            .returning(|_, username, email, password_hash| {
                Ok(User {
                    id: 42,
                    github_id: None,
                    username,
                    email,
                    password_hash,
                })
            });

        let cfg = test_app_config()?;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(cfg.clone())
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(RegisterPayload {
                username: "test_user".into(),
                email: "test@example.com".into(),
                password: Some("StrongPass123".into()),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = to_bytes(resp.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let auth_resp: AuthResponse = serde_json::from_slice(&body)?;
        let decoded = decode::<Claims>(
            &auth_resp.token,
            &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
            &Validation::default(),
        )?;
        assert_eq!(decoded.claims.sub, 42);
        assert_eq!(decoded.claims.username, "test_user");
        Ok(())
    }

    #[actix_web::test]
    async fn test_register_no_password_and_validation_errors() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_create_user()
            .withf(|_, username, email, pw_hash| {
                username == "oauth_user" && email == "oauth@example.com" && pw_hash.is_none()
            })
            .times(1)
            .returning(|_, username, email, _| {
                Ok(User {
                    id: 7,
                    github_id: None,
                    username,
                    email,
                    password_hash: None,
                })
            });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(test_app_config()?)
                .configure(configure),
        )
        .await;

        let req_ok = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(RegisterPayload {
                username: "oauth_user".into(),
                email: "oauth@example.com".into(),
                password: None,
            })
            .to_request();
        let resp_ok = test::call_service(&app, req_ok).await;
        assert_eq!(resp_ok.status(), StatusCode::CREATED);

        let req_empty_pw = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(RegisterPayload {
                username: "oauth_user".into(),
                email: "oauth@example.com".into(),
                password: Some("   ".into()),
            })
            .to_request();
        let resp_empty_pw = test::call_service(&app, req_empty_pw).await;
        assert_eq!(resp_empty_pw.status(), StatusCode::BAD_REQUEST);

        let req_bad_email = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(RegisterPayload {
                username: "oauth_user".into(),
                email: "invalid-email".into(),
                password: None,
            })
            .to_request();
        let resp_bad_email = test::call_service(&app, req_bad_email).await;
        assert_eq!(resp_bad_email.status(), StatusCode::BAD_REQUEST);
        Ok(())
    }

    #[actix_web::test]
    async fn test_register_db_error() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_create_user()
            .times(1)
            .returning(|_, _, _, _| Err(Error::InternalError));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(test_app_config()?)
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(RegisterPayload {
                username: "test".into(),
                email: "test@test.com".into(),
                password: Some("pwd12345".into()),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }

    #[actix_web::test]
    async fn test_login_success_and_failure_branches() -> Result<(), Error> {
        let hashed_pw = hash_password("correct_pw")?;
        let mut mock_repo = MockCddRepository::new();

        let hash_clone = hashed_pw.clone();
        mock_repo
            .expect_find_user_by_username()
            .withf(|u| u == "alice")
            .times(2)
            .returning(move |_| {
                Ok(Some(User {
                    id: 11,
                    github_id: None,
                    username: "alice".into(),
                    email: "alice@example.com".into(),
                    password_hash: Some(hash_clone.clone()),
                }))
            });

        mock_repo
            .expect_find_user_by_username()
            .withf(|u| u == "oauth_only")
            .times(1)
            .returning(|_| {
                Ok(Some(User {
                    id: 12,
                    github_id: Some(99),
                    username: "oauth_only".into(),
                    email: "oauth@example.com".into(),
                    password_hash: None,
                }))
            });

        mock_repo
            .expect_find_user_by_username()
            .withf(|u| u == "missing_user")
            .times(1)
            .returning(|_| Ok(None));

        mock_repo
            .expect_find_user_by_username()
            .withf(|u| u == "db_fail")
            .times(1)
            .returning(|_| Err(Error::InternalError));

        let cfg = test_app_config()?;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(cfg.clone())
                .configure(configure),
        )
        .await;

        // 1. Valid password -> 200 OK + decodable JWT
        let req_ok = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "alice".into(),
                password: Some("correct_pw".into()),
            })
            .to_request();
        let resp_ok = test::call_service(&app, req_ok).await;
        assert_eq!(resp_ok.status(), StatusCode::OK);
        let body = to_bytes(resp_ok.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let auth_resp: AuthResponse = serde_json::from_slice(&body)?;
        let decoded = decode::<Claims>(
            &auth_resp.token,
            &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
            &Validation::default(),
        )?;
        assert_eq!(decoded.claims.sub, 11);
        assert_eq!(decoded.claims.username, "alice");

        // 2. Wrong password -> 401
        let req_wrong = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "alice".into(),
                password: Some("wrong_pw".into()),
            })
            .to_request();
        let resp_wrong = test::call_service(&app, req_wrong).await;
        assert_eq!(resp_wrong.status(), StatusCode::UNAUTHORIZED);

        // 3. No password in payload -> 401
        let req_no_pw = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "alice".into(),
                password: None,
            })
            .to_request();
        let resp_no_pw = test::call_service(&app, req_no_pw).await;
        assert_eq!(resp_no_pw.status(), StatusCode::UNAUTHORIZED);

        // 4. User without password hash -> 401
        let req_no_hash = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "oauth_only".into(),
                password: Some("any_pw".into()),
            })
            .to_request();
        let resp_no_hash = test::call_service(&app, req_no_hash).await;
        assert_eq!(resp_no_hash.status(), StatusCode::UNAUTHORIZED);

        // 5. Non-existent user -> 401
        let req_missing = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "missing_user".into(),
                password: Some("any_pw".into()),
            })
            .to_request();
        let resp_missing = test::call_service(&app, req_missing).await;
        assert_eq!(resp_missing.status(), StatusCode::UNAUTHORIZED);

        // 6. Database failure -> 500 Internal Server Error (not masked as 401)
        let req_db_err = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "db_fail".into(),
                password: Some("any_pw".into()),
            })
            .to_request();
        let resp_db_err = test::call_service(&app, req_db_err).await;
        assert_eq!(resp_db_err.status(), StatusCode::INTERNAL_SERVER_ERROR);

        Ok(())
    }
}
