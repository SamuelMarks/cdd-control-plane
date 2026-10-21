//! Application configuration loading and secret redaction.

use crate::error::Error;
use serde::Deserialize;
use std::fmt;

/// Application configuration loaded from an optional TOML file and environment variables.
///
/// All fields can be overridden via environment variables prefixed with `CDD__`
/// (double underscore as separator), e.g. `CDD__JWT_SECRET=mysecret`.
#[derive(Deserialize, Clone)]
pub struct AppConfig {
    /// PostgreSQL connection URL (env: `CDD__DATABASE_URL`).
    pub database_url: String,
    /// Address and port the HTTP server binds to (env: `CDD__SERVER_BIND`).
    pub server_bind: String,
    /// Secret used to sign and verify JWT tokens (env: `CDD__JWT_SECRET`).
    pub jwt_secret: String,
    /// Secret used to verify GitHub webhook HMAC-SHA256 signatures (env: `CDD__WEBHOOK_SECRET`).
    pub webhook_secret: String,
    /// Dedicated symmetric master key used to encrypt user tokens at rest (env: `CDD__ENCRYPTION_KEY`).
    pub encryption_key: String,
    /// Optional GitHub personal access token used as a system-level fallback (env: `CDD__GITHUB_TOKEN`).
    pub github_token: Option<String>,
    /// Redis connection URL (env: `CDD__REDIS_URL`).
    pub redis_url: String,
}

impl fmt::Debug for AppConfig {
    /// Formats [`AppConfig`] with all credential and secret fields redacted.
    ///
    /// # Arguments
    ///
    /// * `f` - The debug formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppConfig")
            .field("database_url", &"[REDACTED]")
            .field("server_bind", &self.server_bind)
            .field("jwt_secret", &"[REDACTED]")
            .field("webhook_secret", &"[REDACTED]")
            .field("encryption_key", &"[REDACTED]")
            .field(
                "github_token",
                &self.github_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("redis_url", &self.redis_url)
            .finish()
    }
}

impl AppConfig {
    /// Loads [`AppConfig`] from an optional TOML file path and `CDD__*` environment variables.
    ///
    /// Precedence (highest to lowest):
    /// 1. Environment variables (`CDD__*`)
    /// 2. Configuration file (when `config_path` is `Some`)
    /// 3. Built-in defaults
    ///
    /// # Arguments
    ///
    /// * `config_path` - Optional path to a TOML configuration file.
    pub fn load(config_path: Option<&str>) -> Result<Self, Error> {
        let mut builder = config::Config::builder()
            .set_default(
                "database_url",
                "postgres://postgres:password@localhost/cdd_test",
            )?
            .set_default("server_bind", "0.0.0.0:8081")?
            .set_default("jwt_secret", "super-secret-key")?
            .set_default("webhook_secret", "my_webhook_secret_key_for_hmac")?
            .set_default("encryption_key", "default-local-encryption-master-key-32b")?
            .set_default("redis_url", "redis://127.0.0.1:6379/")?;

        if let Some(path) = config_path {
            builder = builder.add_source(config::File::with_name(path).required(true));
        }

        Ok(builder
            .add_source(config::Environment::with_prefix("CDD").separator("__"))
            .build()?
            .try_deserialize()?)
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for configuration loading and secret redaction.

    use super::*;
    use std::io::Write;

    #[test]
    fn test_app_config_load_defaults_and_debug_redaction() -> Result<(), Error> {
        let cfg = AppConfig::load(None)?;
        assert_eq!(cfg.jwt_secret, "super-secret-key");
        assert_eq!(
            cfg.encryption_key,
            "default-local-encryption-master-key-32b"
        );

        let debug_without_gh = format!("{cfg:?}");
        assert!(debug_without_gh.contains("[REDACTED]"));
        assert!(!debug_without_gh.contains("super-secret-key"));
        assert!(!debug_without_gh.contains("default-local-encryption-master-key-32b"));

        let mut cfg_with_gh = cfg;
        cfg_with_gh.github_token = Some("ghp_sensitive_token".to_string());
        let debug_with_gh = format!("{cfg_with_gh:?}");
        assert!(!debug_with_gh.contains("ghp_sensitive_token"));
        Ok(())
    }

    #[test]
    fn test_app_config_load_from_real_toml_file() -> Result<(), Error> {
        let temp_path =
            std::env::temp_dir().join(format!("cdd_config_test_{}.toml", uuid::Uuid::new_v4()));
        let path_str = temp_path.to_string_lossy().to_string();

        let mut file = std::fs::File::create(&temp_path).map_err(|_| Error::InternalError)?;
        file.write_all(
            br#"server_bind = "127.0.0.1:9090"
jwt_secret = "custom-toml-jwt-secret"
encryption_key = "custom-toml-encryption-key-32bytes"
"#,
        )
        .map_err(|_| Error::InternalError)?;

        let cfg = AppConfig::load(Some(&path_str))?;
        let _ = std::fs::remove_file(&temp_path);

        assert_eq!(cfg.server_bind, "127.0.0.1:9090");
        assert_eq!(cfg.jwt_secret, "custom-toml-jwt-secret");
        assert_eq!(cfg.encryption_key, "custom-toml-encryption-key-32bytes");
        Ok(())
    }
}
