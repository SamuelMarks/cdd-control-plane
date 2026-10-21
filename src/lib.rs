//! `cdd-control-plane`
//!
//! Central control-plane API, authentication, RBAC, cryptographic secret vault,
//! asynchronous job queue, and PostgreSQL persistence layer for the CDD ecosystem.

pub mod api;
pub mod config;
pub mod crypto;
pub mod domain;
pub mod error;
pub mod queue;

pub mod db {
    //! Database persistence models, repository traits, and schema definitions.
    pub mod models;
    pub mod repository;
    pub mod schema;
}

pub use error::Error;

/// Crate-wide `Result` alias defaulting to [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;
