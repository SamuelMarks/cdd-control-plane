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
            Error::Database(_) | Error::Pool(_) | Error::InternalError => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error"
                }))
            }
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
