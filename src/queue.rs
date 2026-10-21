//! Asynchronous job queue abstraction and Redis-backed implementation.

use crate::config::AppConfig;
use crate::error::Error;
use async_trait::async_trait;
use fred::prelude::*;
#[cfg(test)]
use mockall::automock;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Redis list key used to enqueue SDK release jobs for `cdd-publisher`.
pub const RELEASE_SDK_QUEUE_KEY: &str = "cdd_jobs:release_sdk";

/// Job payload pushed onto the queue when an SDK release is triggered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ReleaseSdkJob {
    /// Database ID of the created [`crate::db::models::Release`].
    pub release_id: i32,
    /// Database ID of the owning [`crate::db::models::Organization`].
    pub org_id: i32,
    /// Database ID of the parent [`crate::db::models::Repository`].
    pub repo_id: i32,
    /// Git tag name of the release (e.g., `v1.0.0`).
    pub tag_name: String,
}

/// Abstract asynchronous queue interface for dispatching background worker jobs.
#[allow(missing_docs, clippy::missing_docs_in_private_items)]
#[cfg_attr(test, automock)]
#[async_trait]
pub trait JobQueue: Send + Sync {
    /// Enqueues an SDK release job for asynchronous processing by `cdd-publisher`.
    ///
    /// # Arguments
    ///
    /// * `&self` - The queue implementation.
    /// * `job` - The [`ReleaseSdkJob`] payload to serialize and enqueue.
    async fn enqueue_release_sdk(&self, job: &ReleaseSdkJob) -> Result<(), Error>;
}

/// Redis-backed implementation of [`JobQueue`].
#[derive(Clone)]
pub struct QueueClient {
    /// Underlying multiplexed Redis client.
    client: Client,
}

impl QueueClient {
    /// Initializes and connects a new [`QueueClient`] using the Redis URL in [`AppConfig`].
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration containing `redis_url`.
    pub async fn new(config: &AppConfig) -> Result<Self, Error> {
        let redis_config = Config::from_url(&config.redis_url)?;
        let client = Builder::from_config(redis_config).build()?;
        client.init().await?;
        Ok(Self { client })
    }
}

#[async_trait]
impl JobQueue for QueueClient {
    /// Serializes `job` as JSON and `LPUSH`es it onto [`RELEASE_SDK_QUEUE_KEY`].
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`QueueClient`] instance.
    /// * `job` - The [`ReleaseSdkJob`] payload to enqueue.
    async fn enqueue_release_sdk(&self, job: &ReleaseSdkJob) -> Result<(), Error> {
        let payload = serde_json::to_string(job)?;
        let _: () = self.client.lpush(RELEASE_SDK_QUEUE_KEY, payload).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! Unit and integration tests for `QueueClient` and `JobQueue`.

    use super::*;

    #[actix_web::test]
    async fn test_queue_config_invalid() -> Result<(), Error> {
        let mut config = AppConfig::load(None)?;
        config.redis_url = "invalid-url".into();
        let result = QueueClient::new(&config).await;
        assert!(matches!(result, Err(Error::Redis(_))));
        Ok(())
    }

    #[actix_web::test]
    async fn test_queue_config_valid_and_enqueue() -> Result<(), Error> {
        let mut config = AppConfig::load(None)?;
        config.redis_url = "redis://127.0.0.1:6379/0".into();

        let client = QueueClient::new(&config).await?;
        let job = ReleaseSdkJob {
            release_id: 1,
            org_id: 2,
            repo_id: 3,
            tag_name: "v1.0.0".into(),
        };
        client.enqueue_release_sdk(&job).await?;
        Ok(())
    }
}
