//! Authentication module for JWT and Argon2 logic.

use crate::config::AppConfig;
use crate::db::repository::CddRepository;
use actix_web::{web, HttpResponse};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// Response containing the JWT token
#[derive(Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    /// JWT token
    pub token: String,
}

/// Payload for logging in with username/password
#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginPayload {
    /// Username
    pub username: String,
    /// Password
    pub password: Option<String>,
}

/// Payload for registering a new user
#[derive(Serialize, Deserialize, ToSchema)]
pub struct RegisterPayload {
    /// Desired username
    pub username: String,
    /// Email address
    pub email: String,
    /// Password
    pub password: Option<String>,
}

/// Generate a signed JWT for the given user, using the secret from `AppConfig`.
fn generate_token(
    user_id: i32,
    username: &str,
    jwt_secret: &[u8],
) -> Result<String, crate::error::Error> {
    let claims = crate::api::auth_middleware::Claims {
        sub: user_id,
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        username: username.to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret),
    )
    .map_err(|_| crate::error::Error::InternalError)
}

fn hash_password(password: &str) -> Result<String, crate::error::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| crate::error::Error::InternalError)?;
    Ok(hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    if let Ok(parsed_hash) = PasswordHash::new(hash) {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    } else {
        false
    }
}

/// Register a new user
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterPayload,
    responses(
        (status = 201, description = "Successfully registered", body = AuthResponse),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn register(
    payload: web::Json<RegisterPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
    cfg: web::Data<AppConfig>,
) -> Result<HttpResponse, crate::error::Error> {
    let hashed_pw = if let Some(pw) = payload.password.as_ref() {
        Some(hash_password(pw)?)
    } else {
        None
    };

    let user = repo
        .create_user(
            None,
            payload.username.clone(),
            payload.email.clone(),
            hashed_pw,
        )
        .await
        .map_err(|_| crate::error::Error::InternalError)?;

    let token = generate_token(user.id, &user.username, cfg.jwt_secret.as_bytes())?;

    Ok(HttpResponse::Created().json(AuthResponse { token }))
}

/// Login with username/password
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
) -> Result<HttpResponse, crate::error::Error> {
    match repo.find_user_by_username(payload.username.clone()).await {
        Ok(Some(user)) => {
            if let Some(pw) = &payload.password {
                if let Some(h) = &user.password_hash {
                    if verify_password(pw, h) {
                        let token =
                            generate_token(user.id, &user.username, cfg.jwt_secret.as_bytes())?;
                        return Ok(HttpResponse::Ok().json(AuthResponse { token }));
                    }
                }
            }
            Err(crate::error::Error::Unauthorized)
        }
        _ => Err(crate::error::Error::Unauthorized),
    }
}

/// Configure auth routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login_password)),
    );
}
#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_register() {
        use crate::db::models::User;
        use crate::db::repository::MockCddRepository;
        use actix_web::App;
        let mut mock_repo = MockCddRepository::new();
        mock_repo.expect_create_user().returning(|_, _, _, _| {
            Ok(User {
                id: 1,
                github_id: None,
                username: "test".into(),
                email: "test@test.com".into(),
                password_hash: None,
            })
        });

        let app = actix_web::test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(web::Data::new(AppConfig::load(None).expect("config")))
                .configure(configure),
        )
        .await;

        let req = actix_web::test::TestRequest::post()
            .uri("/auth/register")
            .set_json(RegisterPayload {
                username: "test".into(),
                email: "test@test.com".into(),
                password: Some("pwd".into()),
            })
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn test_login_success() {
        use crate::db::models::User;
        use crate::db::repository::MockCddRepository;
        use actix_web::App;
        let mut mock_repo = MockCddRepository::new();
        let hashed_pw = hash_password("pwd").unwrap_or_else(|_| "".into());
        mock_repo
            .expect_find_user_by_username()
            .returning(move |_| {
                Ok(Some(User {
                    id: 1,
                    github_id: None,
                    username: "test".into(),
                    email: "test@test.com".into(),
                    password_hash: Some(hashed_pw.clone()),
                }))
            });

        let app = actix_web::test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(web::Data::new(AppConfig::load(None).expect("config")))
                .configure(configure),
        )
        .await;

        let req = actix_web::test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "test".into(),
                password: Some("pwd".into()),
            })
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_login_unauthorized() {
        use crate::db::repository::MockCddRepository;
        use actix_web::App;
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_find_user_by_username()
            .returning(|_| Ok(None));

        let app = actix_web::test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(web::Data::new(AppConfig::load(None).expect("config")))
                .configure(configure),
        )
        .await;

        let req = actix_web::test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "test".into(),
                password: Some("wrong".into()),
            })
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }
}

#[test]
fn test_verify_password_invalid_hash() {
    assert!(!verify_password("pwd", "invalid_hash_string"));
}

#[actix_web::test]
async fn test_register_no_password() {
    use crate::db::models::User;
    use crate::db::repository::MockCddRepository;
    use actix_web::App;
    let mut mock_repo = MockCddRepository::new();
    mock_repo.expect_create_user().returning(|_, _, _, _| {
        Ok(User {
            id: 1,
            github_id: None,
            username: "test".into(),
            email: "test@test.com".into(),
            password_hash: None,
        })
    });

    let app = actix_web::test::init_service(
        App::new()
            .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
            .app_data(web::Data::new(AppConfig::load(None).expect("config")))
            .configure(configure),
    )
    .await;

    let req = actix_web::test::TestRequest::post()
        .uri("/auth/register")
        .set_json(RegisterPayload {
            username: "test".into(),
            email: "test@test.com".into(),
            password: None,
        })
        .to_request();
    let resp = actix_web::test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);
}

#[actix_web::test]
async fn test_login_wrong_password() {
    use crate::db::models::User;
    use crate::db::repository::MockCddRepository;
    use actix_web::App;
    let mut mock_repo = MockCddRepository::new();
    let hashed_pw = hash_password("pwd").unwrap_or_else(|_| "".into());
    mock_repo
        .expect_find_user_by_username()
        .returning(move |_| {
            Ok(Some(User {
                id: 1,
                github_id: None,
                username: "test".into(),
                email: "test@test.com".into(),
                password_hash: Some(hashed_pw.clone()),
            }))
        });

    let app = actix_web::test::init_service(
        App::new()
            .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
            .app_data(web::Data::new(AppConfig::load(None).expect("config")))
            .configure(configure),
    )
    .await;

    let req = actix_web::test::TestRequest::post()
        .uri("/auth/login")
        .set_json(LoginPayload {
            username: "test".into(),
            password: Some("wrong".into()),
        })
        .to_request();
    let resp = actix_web::test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_login_no_password() {
    use crate::db::models::User;
    use crate::db::repository::MockCddRepository;
    use actix_web::App;
    let mut mock_repo = MockCddRepository::new();
    let hashed_pw = hash_password("pwd").unwrap_or_else(|_| "".into());
    mock_repo
        .expect_find_user_by_username()
        .returning(move |_| {
            Ok(Some(User {
                id: 1,
                github_id: None,
                username: "test".into(),
                email: "test@test.com".into(),
                password_hash: Some(hashed_pw.clone()),
            }))
        });

    let app = actix_web::test::init_service(
        App::new()
            .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
            .app_data(web::Data::new(AppConfig::load(None).expect("config")))
            .configure(configure),
    )
    .await;

    let req = actix_web::test::TestRequest::post()
        .uri("/auth/login")
        .set_json(LoginPayload {
            username: "test".into(),
            password: None,
        })
        .to_request();
    let resp = actix_web::test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}
