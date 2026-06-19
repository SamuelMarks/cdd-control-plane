//! Configuration module.

use serde::{Deserialize, Serialize};

/// Application configuration loaded from a file and/or environment variables.
///
/// All fields can be overridden via environment variables prefixed with `CDD__`
/// (double underscore as separator), e.g. `CDD__JWT_SECRET=mysecret`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    /// PostgreSQL connection URL (env: `CDD__DATABASE_URL`).
    pub database_url: String,
    /// Address and port the HTTP server binds to (env: `CDD__SERVER_BIND`).
    pub server_bind: String,
    /// Secret used to sign and verify JWT tokens (env: `CDD__JWT_SECRET`).
    ///
    /// Defaults to `"super-secret-key"` — **must** be overridden in production.
    pub jwt_secret: String,
    /// Secret used to verify GitHub webhook HMAC-SHA256 signatures
    /// (env: `CDD__WEBHOOK_SECRET`).
    ///
    /// Defaults to `"my_webhook_secret"` — **must** be overridden in production.
    pub webhook_secret: String,
    /// Optional GitHub personal access token used as a system-level fallback
    /// when no per-user token is available (env: `CDD__GITHUB_TOKEN`).
    pub github_token: Option<String>,
    /// Redis connection URL (env: `CDD__REDIS_URL`).
    pub redis_url: String,
}

impl AppConfig {
    /// Load configuration from an optional file path and environment variables.
    ///
    /// Precedence (highest → lowest):
    /// 1. Environment variables (`CDD__*`)
    /// 2. Config file (if `config_path` is `Some`)
    /// 3. Built-in defaults
    pub fn load(config_path: Option<&str>) -> Result<Self, crate::error::Error> {
        let mut builder = config::Config::builder()
            .set_default("database_url", "postgres://postgres:password@localhost/cdd")
            .map_err(|_| crate::error::Error::InternalError)?
            .set_default("server_bind", "0.0.0.0:8081")
            .map_err(|_| crate::error::Error::InternalError)?
            .set_default("jwt_secret", "super-secret-key")
            .map_err(|_| crate::error::Error::InternalError)?
            .set_default("webhook_secret", "my_webhook_secret")
            .map_err(|_| crate::error::Error::InternalError)?
            .set_default("redis_url", "redis://127.0.0.1:6379/")
            .map_err(|_| crate::error::Error::InternalError)?;

        if let Some(path) = config_path {
            builder = builder.add_source(config::File::with_name(path).required(false));
        }

        builder
            .add_source(config::Environment::with_prefix("CDD").separator("__"))
            .build()
            .map_err(|_| crate::error::Error::InternalError)?
            .try_deserialize()
            .map_err(|_| crate::error::Error::InternalError)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_load_defaults() {
        let config = AppConfig::load(None).expect("failed to load config");
        assert_eq!(config.jwt_secret, "super-secret-key");
    }
}

#[test]
fn test_app_config_load_with_file() {
    let config = AppConfig::load(Some("non_existent_file.toml")).expect("failed to load config");
    assert_eq!(config.jwt_secret, "super-secret-key");
}
