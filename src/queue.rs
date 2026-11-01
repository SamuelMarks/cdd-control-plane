//! Asynchronous job queue abstraction and Redis-backed implementation.

use crate::config::AppConfig;
use crate::domain::{DocsPublishConfig, Registry, RepoTopologyMode, TargetLanguage};
use crate::error::Error;
use async_trait::async_trait;
use fred::prelude::*;
#[cfg(test)]
use mockall::automock;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Redis list key used to enqueue SDK release jobs for `cdd-publisher`.
pub const RELEASE_SDK_QUEUE_KEY: &str = "cdd_jobs:release_sdk";

/// Returns `"monorepo"` as the default topology mode for [`ReleaseSdkJob`] deserialization.
///
/// # Returns
///
/// [`RepoTopologyMode::Monorepo`].
const fn default_job_topology() -> RepoTopologyMode {
    RepoTopologyMode::Monorepo
}

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
    /// Upstream GitHub organization or user login (`${GITHUB_ORG_NAME}`).
    #[serde(default)]
    pub github_org_name: String,
    /// Resolved repository name (`${GITHUB_ORG_NAME}-sdk`, `${GITHUB_ORG_NAME}-sdk-${LANGUAGE}`, or custom).
    #[serde(default)]
    pub repo_name: String,
    /// Repository topology mode (`monorepo` or `polyrepo`).
    #[serde(default = "default_job_topology")]
    pub topology_mode: RepoTopologyMode,
    /// Selected target languages to build and publish for this release.
    #[serde(default)]
    pub target_languages: Vec<TargetLanguage>,
    /// Selected package registries to publish artifacts to (`crates_io`, `npm`, `pypi`, etc.).
    #[serde(default)]
    pub target_registries: Vec<Registry>,
    /// Whether artifacts should be cryptographically signed with the user's PGP key.
    #[serde(default)]
    pub enable_pgp_signing: bool,
    /// Documentation generation and hosting target configuration.
    #[serde(default)]
    pub docs_config: DocsPublishConfig,
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
    ///
    /// # Returns
    ///
    /// `Ok(())` when the job is pushed to the queue.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if serialization or Redis `LPUSH` fails.
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
    ///
    /// # Returns
    ///
    /// Connected [`QueueClient`] instance.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Redis`] if the URL is invalid or connection initialization fails.
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
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if JSON serialization or Redis `LPUSH` fails.
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
            github_org_name: "SamuelMarks".into(),
            repo_name: "SamuelMarks-sdk".into(),
            topology_mode: RepoTopologyMode::Monorepo,
            target_languages: vec![TargetLanguage::Rust, TargetLanguage::Python],
            target_registries: vec![Registry::CratesIo, Registry::PyPi],
            enable_pgp_signing: true,
            docs_config: DocsPublishConfig::default(),
        };
        client.enqueue_release_sdk(&job).await?;

        // Also test minimal serde defaults on ReleaseSdkJob
        let minimal_json = serde_json::json!({
            "release_id": 10,
            "org_id": 20,
            "repo_id": 30,
            "tag_name": "v0.1.0"
        });
        let parsed: ReleaseSdkJob = serde_json::from_value(minimal_json)?;
        assert_eq!(parsed.topology_mode, RepoTopologyMode::Monorepo);
        assert_eq!(parsed.target_languages, Vec::<TargetLanguage>::new());
        Ok(())
    }
}
