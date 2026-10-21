//! Error handling types for `cdd-control-plane`.

use actix_web::{HttpResponse, ResponseError};
use derive_more::derive::{Display, From};
use diesel::r2d2::PoolError;
use diesel::result::DatabaseErrorKind;

/// The unified error type for the control plane application.
#[derive(Debug, Display, From)]
pub enum Error {
    /// A relational database error occurred.
    #[display("Database error")]
    Database(diesel::result::Error),

    /// A database connection pool error occurred.
    #[display("Database pool error")]
    Pool(PoolError),

    /// An Actix blocking thread-pool execution error occurred.
    #[display("Blocking thread pool error")]
    Blocking(actix_web::error::BlockingError),

    /// An application configuration error occurred.
    #[display("Configuration error")]
    Config(config::ConfigError),

    /// A Redis client or queue error occurred.
    #[display("Redis error")]
    Redis(fred::error::Error),

    /// A cryptographic AEAD encryption or decryption error occurred.
    #[display("Crypto AEAD error")]
    CryptoAead(crypto_secretbox::aead::Error),

    /// A Base64 decoding error occurred.
    #[display("Base64 decode error")]
    Base64Decode(base64::DecodeError),

    /// A UTF-8 string decoding error occurred.
    #[display("UTF-8 decode error")]
    Utf8(std::string::FromUtf8Error),

    /// A fixed-size byte slice conversion error occurred.
    #[display("Array conversion error")]
    TryFromSlice(std::array::TryFromSliceError),

    /// A JSON Web Token encoding or decoding error occurred.
    #[display("JWT error")]
    Jwt(jsonwebtoken::errors::Error),

    /// An Argon2 password hashing or verification error occurred.
    #[display("Password hashing error")]
    PasswordHash(argon2::password_hash::Error),

    /// A JSON serialization or deserialization error occurred.
    #[display("JSON error")]
    Json(serde_json::Error),

    /// A client validation or malformed input error (HTTP 400).
    #[display("Bad request: {_0}")]
    #[from(ignore)]
    BadRequest(String),

    /// An authentication failure error (HTTP 401).
    #[display("Unauthorized")]
    Unauthorized,

    /// An authorization / insufficient permissions error (HTTP 403).
    #[display("Forbidden: {_0}")]
    #[from(ignore)]
    Forbidden(String),

    /// A requested resource was not found (HTTP 404).
    #[display("Not found: {_0}")]
    #[from(ignore)]
    NotFound(String),

    /// A resource state conflict or uniqueness violation occurred (HTTP 409).
    #[display("Conflict: {_0}")]
    #[from(ignore)]
    Conflict(String),

    /// An unexpected internal server error occurred (HTTP 500).
    #[display("Internal server error")]
    InternalError,
}

impl std::error::Error for Error {}

impl ResponseError for Error {
    /// Converts the domain [`Error`] into an [`HttpResponse`] with a JSON error payload.
    ///
    /// # Arguments
    ///
    /// * `&self` - The error instance to convert into an HTTP response.
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::Database(diesel::result::Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _,
            )) => HttpResponse::Conflict().json(serde_json::json!({
                "error": "Resource already exists"
            })),
            Error::Database(_)
            | Error::Pool(_)
            | Error::Blocking(_)
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
            Error::Forbidden(msg) => HttpResponse::Forbidden().json(serde_json::json!({
                "error": msg
            })),
            Error::NotFound(msg) => HttpResponse::NotFound().json(serde_json::json!({
                "error": msg
            })),
            Error::Conflict(msg) => HttpResponse::Conflict().json(serde_json::json!({
                "error": msg
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the unified error module.

    use super::*;
    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;

    /// Helper to extract the JSON error message from an `Error`'s HTTP response.
    async fn extract_error_body(err: &Error) -> Result<(StatusCode, String), Error> {
        let resp = err.error_response();
        let status = resp.status();
        let bytes = to_bytes(resp.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)?;
        let msg = value["error"].as_str().unwrap_or_default().to_string();
        Ok((status, msg))
    }

    #[actix_web::test]
    async fn test_error_display_and_responses() -> Result<(), Error> {
        assert_eq!(Error::InternalError.to_string(), "Internal server error");
        assert_eq!(Error::Unauthorized.to_string(), "Unauthorized");
        assert_eq!(
            Error::BadRequest("invalid input".into()).to_string(),
            "Bad request: invalid input"
        );
        assert_eq!(
            Error::Forbidden("insufficient role".into()).to_string(),
            "Forbidden: insufficient role"
        );
        assert_eq!(
            Error::NotFound("missing".into()).to_string(),
            "Not found: missing"
        );
        assert_eq!(
            Error::Conflict("duplicate".into()).to_string(),
            "Conflict: duplicate"
        );

        let (status, msg) = extract_error_body(&Error::InternalError).await?;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(msg, "Internal server error");

        let (status, msg) = extract_error_body(&Error::Unauthorized).await?;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(msg, "Unauthorized");

        let (status, msg) = extract_error_body(&Error::BadRequest("bad".into())).await?;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(msg, "bad");

        let (status, msg) = extract_error_body(&Error::Forbidden("denied".into())).await?;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(msg, "denied");

        let (status, msg) = extract_error_body(&Error::NotFound("no".into())).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(msg, "no");

        let (status, msg) = extract_error_body(&Error::Conflict("exists".into())).await?;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(msg, "exists");

        let (status, msg) =
            extract_error_body(&Error::Database(diesel::result::Error::NotFound)).await?;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(msg, "Internal server error");

        let unique_err = Error::Database(diesel::result::Error::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            Box::new("duplicate key".to_string()),
        ));
        let (status, msg) = extract_error_body(&unique_err).await?;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(msg, "Resource already exists");

        Ok(())
    }

    #[test]
    fn test_pool_error() {
        let manager =
            diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new("postgres://invalid");
        let pool_res = diesel::r2d2::Pool::builder()
            .connection_timeout(std::time::Duration::from_millis(1))
            .build(manager);
        assert!(pool_res.is_err());
        if let Err(pool_err) = pool_res {
            let err = Error::Pool(pool_err);
            assert_eq!(
                err.error_response().status(),
                StatusCode::INTERNAL_SERVER_ERROR
            );
        }
    }
}
