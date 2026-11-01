//! JWT authentication middleware and Actix request extractor.

use crate::config::AppConfig;
use crate::error::Error;
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use utoipa::ToSchema;

/// Claims stored inside a signed JWT issued by `cdd-control-plane`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    /// Subject — the numeric database user ID.
    pub sub: i32,
    /// Expiration time (Unix timestamp in seconds).
    pub exp: usize,
    /// The authenticated user's login username.
    pub username: String,
}

/// Actix-web extractor that validates a Bearer JWT against [`AppConfig::jwt_secret`]
/// and yields the caller's verified identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedUser {
    /// The authenticated user's database primary key ID.
    pub user_id: i32,
    /// The authenticated user's username.
    pub username: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    /// Extracts and validates the `Authorization: Bearer <token>` header from the incoming HTTP request.
    ///
    /// # Arguments
    ///
    /// * `req` - The incoming [`HttpRequest`].
    /// * `payload` - The request body stream [`Payload`].
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let Some(cfg) = req.app_data::<web::Data<AppConfig>>() else {
            return ready(Err(Error::InternalError.into()));
        };

        let secret = cfg.jwt_secret.as_bytes();
        let Ok(bearer) = BearerAuth::from_request(req, payload).into_inner() else {
            return ready(Err(Error::Unauthorized.into()));
        };

        match decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(secret),
            &Validation::default(),
        ) {
            Ok(token_data) => ready(Ok(AuthenticatedUser {
                user_id: token_data.claims.sub,
                username: token_data.claims.username,
            })),
            Err(_) => ready(Err(Error::Unauthorized.into())),
        }
    }
}

#[cfg(test)]
/// Constructs a default [`web::Data<AppConfig>`] instance for API unit tests.
pub fn test_app_config() -> Result<web::Data<AppConfig>, Error> {
    Ok(web::Data::new(AppConfig::load(None)?))
}

#[cfg(test)]
/// Generates a test JWT token signed with the default [`AppConfig`] secret key.
pub fn generate_test_token() -> Result<String, Error> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let cfg = AppConfig::load(None)?;
    let ts = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();
    let exp = usize::try_from(ts).unwrap_or(usize::MAX);
    let claims = Claims {
        sub: 1,
        exp,
        username: "testuser".to_string(),
    };
    let key = EncodingKey::from_secret(cfg.jwt_secret.as_bytes());
    Ok(encode(&Header::default(), &claims, &key)?)
}

#[cfg(test)]
mod tests {
    //! Unit tests for the `AuthenticatedUser` request extractor.

    use super::*;
    use actix_web::{body::to_bytes, http::StatusCode, test, App, HttpResponse, Responder};

    /// Test handler that echoes the authenticated user's ID and username.
    async fn dummy_handler(user: AuthenticatedUser) -> impl Responder {
        HttpResponse::Ok().json(serde_json::json!({
            "user_id": user.user_id,
            "username": user.username,
        }))
    }

    #[actix_web::test]
    async fn test_auth_middleware_missing_config_returns_internal_error() -> Result<(), Error> {
        let app = test::init_service(App::new().route("/", web::get().to(dummy_handler))).await;

        let token = generate_test_token()?;
        let req = test::TestRequest::get()
            .uri("/")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }

    #[actix_web::test]
    async fn test_auth_middleware_invalid_token() -> Result<(), Error> {
        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .route("/", web::get().to(dummy_handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/")
            .insert_header(("Authorization", "Bearer invalid.token.value"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let body = to_bytes(resp.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let json: serde_json::Value = serde_json::from_slice(&body)?;
        assert_eq!(json["error"], "Unauthorized");
        Ok(())
    }

    #[actix_web::test]
    async fn test_auth_middleware_no_token() -> Result<(), Error> {
        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .route("/", web::get().to(dummy_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        Ok(())
    }

    #[actix_web::test]
    async fn test_auth_middleware_with_config_validates_claims() -> Result<(), Error> {
        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .route("/", web::get().to(dummy_handler)),
        )
        .await;

        let token = generate_test_token()?;
        let req = test::TestRequest::get()
            .uri("/")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let json: serde_json::Value = serde_json::from_slice(&body)?;
        assert_eq!(json["user_id"], 1);
        assert_eq!(json["username"], "testuser");
        Ok(())
    }
}
