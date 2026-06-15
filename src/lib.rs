#![deny(missing_docs)]

//! cdd-control-plane
//!
//! Control plane database and web server endpoints.

pub mod api;
pub mod config;
pub mod crypto;
pub mod error;
pub mod queue;

pub mod db {
    //! Database module
    pub mod models;
    pub mod repository;
    pub mod schema;
}

pub use error::Error;

/// Type alias for the application's Result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
