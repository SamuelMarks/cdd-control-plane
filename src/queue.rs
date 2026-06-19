//! Asynchronous queue integration via Redis.

use crate::config::AppConfig;
use fred::prelude::*;
use serde::{Deserialize, Serialize};

/// The job payload for SDK release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseSdkJob {
    /// The ID of the release.
    pub release_id: i32,
    /// The organization ID.
    pub org_id: i32,
    /// The repository ID.
    pub repo_id: i32,
    /// The tag name.
    pub tag_name: String,
}

/// A client for enqueuing jobs to Redis.
#[derive(Clone)]
pub struct QueueClient {
    /// The underlying Redis client.
    client: Client,
}

impl QueueClient {
    /// Initialize a new Redis queue client.
    pub async fn new(config: &AppConfig) -> Result<Self, crate::error::Error> {
        let redis_config =
            Config::from_url(&config.redis_url).map_err(|_| crate::error::Error::InternalError)?;
        let client = Builder::from_config(redis_config)
            .build()
            .map_err(|_| crate::error::Error::InternalError)?;
        client
            .init()
            .await
            .map_err(|_| crate::error::Error::InternalError)?;
        Ok(Self { client })
    }

    /// Enqueue a "Release SDK" job.
    pub async fn enqueue_release_sdk(
        &self,
        job: &ReleaseSdkJob,
    ) -> Result<(), crate::error::Error> {
        let payload = serde_json::to_string(job).map_err(|_| crate::error::Error::InternalError)?;
        // We push to a Redis List acting as a queue.
        let _: () = self
            .client
            .lpush("cdd_jobs:release_sdk", payload)
            .await
            .map_err(|_| crate::error::Error::InternalError)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_queue_config_invalid() -> Result<(), Box<dyn std::error::Error>> {
        let config = AppConfig {
            database_url: "".into(),
            server_bind: "".into(),
            jwt_secret: "".into(),
            webhook_secret: "".into(),
            github_token: None,
            redis_url: "invalid-url".into(),
        };
        let result = QueueClient::new(&config).await;
        assert!(result.is_err());
        Ok(())
    }

    #[actix_web::test]
    async fn test_queue_config_valid() -> Result<(), Box<dyn std::error::Error>> {
        let config = AppConfig {
            database_url: "".into(),
            server_bind: "".into(),
            jwt_secret: "".into(),
            webhook_secret: "".into(),
            github_token: None,
            redis_url: "redis://127.0.0.1:6379/0".into(),
        };
        let client_result = QueueClient::new(&config).await;
        assert!(client_result.is_ok());

        let client = client_result?;
        let job = ReleaseSdkJob {
            release_id: 1,
            org_id: 2,
            repo_id: 3,
            tag_name: "v1.0.0".into(),
        };
        let enqueue_result = client.enqueue_release_sdk(&job).await;
        assert!(enqueue_result.is_ok());
        Ok(())
    }
}
