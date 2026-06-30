//! Error handling types.

use actix_web::{HttpResponse, ResponseError};
use derive_more::derive::{Display, From};
use diesel::r2d2::PoolError;

/// The main error type for the application.
#[derive(Debug, Display, From)]
pub enum Error {
    /// A database error occurred.
    #[display("Database error")]
    Database(diesel::result::Error),

    /// A pool error occurred.
    #[display("Database pool error")]
    Pool(PoolError),

    /// A configuration error occurred.
    #[display("Configuration error")]
    Config(config::ConfigError),

    /// A Redis error occurred.
    #[display("Redis error")]
    Redis(fred::error::Error),

    /// A crypto AEAD error occurred.
    #[display("Crypto AEAD error")]
    CryptoAead(crypto_secretbox::aead::Error),

    /// A base64 decoding error occurred.
    #[display("Base64 decode error")]
    Base64Decode(base64::DecodeError),

    /// A UTF-8 decoding error occurred.
    #[display("UTF-8 decode error")]
    Utf8(std::string::FromUtf8Error),

    /// An array conversion error occurred.
    #[display("Array conversion error")]
    TryFromSlice(std::array::TryFromSliceError),

    /// A JWT encoding/decoding error.
    #[display("JWT error")]
    Jwt(jsonwebtoken::errors::Error),

    /// A password hashing error.
    #[display("Password hashing error")]
    PasswordHash(argon2::password_hash::Error),

    /// A JSON serialization error occurred.
    #[display("JSON error")]
    Json(serde_json::Error),

    /// A bad request error.
    #[display("Bad request: {}", _0)]
    #[from(ignore)]
    BadRequest(String),

    /// An unauthorized error.
    #[display("Unauthorized")]
    Unauthorized,

    /// A not found error.
    #[display("Not found: {}", _0)]
    #[from(ignore)]
    NotFound(String),

    /// An internal server error.
    #[display("Internal server error")]
    InternalError,
}

impl std::error::Error for Error {}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::Database(_)
            | Error::Pool(_)
            | Error::Config(_)
            | Error::Redis(_)
            | Error::CryptoAead(_)
            | Error::Base64Decode(_)
            | Error::Utf8(_)
            | Error::TryFromSlice(_)
            | Error::Jwt(_)
            | Error::PasswordHash(_)
            | Error::Json(_)
            | Error::InternalError => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })),
            Error::BadRequest(msg) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": msg
            })),
            Error::Unauthorized => HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized"
            })),
            Error::NotFound(msg) => HttpResponse::NotFound().json(serde_json::json!({
                "error": msg
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_error_display() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(Error::InternalError.to_string(), "Internal server error");
        Ok(())
    }
    #[test]
    fn test_error_response() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            Error::InternalError.error_response().status(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            Error::Unauthorized.error_response().status(),
            actix_web::http::StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            Error::BadRequest("bad".into()).error_response().status(),
            actix_web::http::StatusCode::BAD_REQUEST
        );
        assert_eq!(
            Error::NotFound("no".into()).error_response().status(),
            actix_web::http::StatusCode::NOT_FOUND
        );
        assert_eq!(
            Error::Database(diesel::result::Error::NotFound)
                .error_response()
                .status(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        Ok(())
    }

    #[test]
    fn test_pool_error() -> Result<(), Box<dyn std::error::Error>> {
        let manager =
            diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new("postgres://invalid");
        let pool_res = diesel::r2d2::Pool::builder()
            .connection_timeout(std::time::Duration::from_millis(1))
            .build(manager);
        match pool_res {
            Ok(_) => panic!("Expected error"),
            Err(pool_err) => {
                assert_eq!(
                    crate::error::Error::Pool(pool_err)
                        .error_response()
                        .status(),
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
                );
            }
        }
        Ok(())
    }
}
