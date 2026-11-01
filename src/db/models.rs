//! Database persistence and insertion models for `cdd-control-plane`.

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::db::schema::{
    audit_logs, organization_users, organizations, releases, repositories, user_tokens, users,
};
use crate::domain::AuthMethod;

/// Persistent user record representing an authenticated account or synced GitHub/Google identity.
#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
    Eq,
)]
#[diesel(table_name = users)]
pub struct User {
    /// Unique internal primary key for the user.
    pub id: i32,
    /// Optional upstream GitHub numeric user ID.
    pub github_id: Option<i64>,
    /// Unique login handle of the user.
    pub username: String,
    /// Unique email address of the user.
    pub email: String,
    /// Optional Argon2 PHC password hash string (omitted when serialized to JSON).
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    /// Optional upstream Google OAuth subject identifier (`sub`).
    #[serde(default)]
    pub google_id: Option<String>,
    /// Optional user avatar URL.
    #[serde(default)]
    pub avatar_url: Option<String>,
}

impl User {
    /// Returns the list of active [`AuthMethod`] credentials linked to this user account.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`User`] record.
    ///
    /// # Returns
    ///
    /// Vector of active [`AuthMethod`] variants (`Password`, `GitHub`, `Google`).
    #[must_use]
    pub fn connected_providers(&self) -> Vec<AuthMethod> {
        let mut methods = Vec::with_capacity(3);
        if self.password_hash.is_some() {
            methods.push(AuthMethod::Password);
        }
        if self.github_id.is_some() {
            methods.push(AuthMethod::GitHub);
        }
        if self.google_id.is_some() {
            methods.push(AuthMethod::Google);
        }
        methods
    }
}

/// Insertion payload for creating a new [`User`] row.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    /// Optional upstream GitHub numeric user ID.
    pub github_id: Option<i64>,
    /// Unique login handle of the user.
    pub username: &'a str,
    /// Unique email address of the user.
    pub email: &'a str,
    /// Optional Argon2 PHC password hash string.
    pub password_hash: Option<&'a str>,
    /// Optional upstream Google OAuth subject identifier.
    pub google_id: Option<&'a str>,
    /// Optional user avatar URL.
    pub avatar_url: Option<&'a str>,
}

/// Persistent organization record representing a multi-tenant workspace.
#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
    Eq,
)]
#[diesel(table_name = organizations)]
pub struct Organization {
    /// Unique internal primary key for the organization.
    pub id: i32,
    /// Optional upstream GitHub numeric organization ID.
    pub github_id: Option<i64>,
    /// Unique organization slug / login name.
    pub login: String,
    /// Optional human-readable description of the organization.
    pub description: Option<String>,
    /// Optional upstream `${GITHUB_ORG_NAME}` associated with the organization.
    #[serde(default)]
    pub github_org_name: Option<String>,
}

/// Insertion payload for creating a new [`Organization`] row.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = organizations)]
pub struct NewOrganization<'a> {
    /// Optional upstream GitHub numeric organization ID.
    pub github_id: Option<i64>,
    /// Unique organization slug / login name.
    pub login: &'a str,
    /// Optional human-readable description of the organization.
    pub description: Option<&'a str>,
    /// Optional upstream `${GITHUB_ORG_NAME}` associated with the organization.
    pub github_org_name: Option<&'a str>,
}

/// Persistent join record associating a [`User`] with an [`Organization`] and an [`crate::domain::OrgRole`].
#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
    Eq,
)]
#[diesel(table_name = organization_users)]
#[diesel(primary_key(organization_id, user_id))]
pub struct OrganizationUser {
    /// Foreign key referencing the [`Organization`] primary key.
    pub organization_id: i32,
    /// Foreign key referencing the [`User`] primary key.
    pub user_id: i32,
    /// Canonical lowercase role string (`owner`, `admin`, or `member`).
    pub role: String,
}

/// Insertion payload for linking a [`User`] to an [`Organization`].
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = organization_users)]
pub struct NewOrganizationUser<'a> {
    /// Foreign key referencing the [`Organization`] primary key.
    pub organization_id: i32,
    /// Foreign key referencing the [`User`] primary key.
    pub user_id: i32,
    /// Canonical lowercase role string (`owner`, `admin`, or `member`).
    pub role: &'a str,
}

/// Returns the default topology mode string (`"monorepo"`) for serde deserialization.
///
/// # Returns
///
/// `"monorepo"`.
fn default_topology_mode_str() -> String {
    "monorepo".to_string()
}

/// Returns an empty JSON object (`{}`) for serde deserialization defaults.
///
/// # Returns
///
/// `serde_json::json!({})`.
fn default_empty_json_obj() -> serde_json::Value {
    serde_json::json!({})
}

/// Returns an empty JSON array (`[]`) for serde deserialization defaults.
///
/// # Returns
///
/// `serde_json::json!([])`.
fn default_empty_json_arr() -> serde_json::Value {
    serde_json::json!([])
}

/// Returns the default documentation configuration JSON (`{"enabled":true,"target":"official"}`).
///
/// # Returns
///
/// Default documentation configuration JSON object.
fn default_docs_config_json() -> serde_json::Value {
    serde_json::json!({
        "enabled": true,
        "target": "official"
    })
}

/// Persistent repository record belonging to an [`Organization`].
#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
    Eq,
)]
#[diesel(table_name = repositories)]
pub struct Repository {
    /// Unique internal primary key for the repository.
    pub id: i32,
    /// Foreign key referencing the owning [`Organization`].
    pub organization_id: i32,
    /// Optional upstream GitHub numeric repository ID.
    pub github_id: Option<i64>,
    /// Name of the repository within the organization (`${GITHUB_ORG_NAME}-sdk`, `${GITHUB_ORG_NAME}-sdk-${LANGUAGE}`, or custom).
    pub name: String,
    /// Optional human-readable description of the repository.
    pub description: Option<String>,
    /// Upstream GitHub organization or user login (`${GITHUB_ORG_NAME}`).
    #[serde(default)]
    pub github_org_name: String,
    /// Repository topology mode (`"monorepo"` or `"polyrepo"`).
    #[serde(default = "default_topology_mode_str")]
    pub topology_mode: String,
    /// Optional custom monorepo name override (defaults to `${GITHUB_ORG_NAME}-sdk`).
    #[serde(default)]
    pub monorepo_name: Option<String>,
    /// JSON map of per-language polyrepo name overrides (`{ "<lang>": "<repo_name>" }`).
    #[serde(default = "default_empty_json_obj")]
    pub polyrepo_names_json: serde_json::Value,
    /// JSON array of selected [`crate::domain::TargetLanguage`] slugs.
    #[serde(default = "default_empty_json_arr")]
    pub target_languages_json: serde_json::Value,
    /// JSON map of [`crate::domain::LanguageGenerationOptions`] keyed by language slug.
    #[serde(default = "default_empty_json_obj")]
    pub language_options_json: serde_json::Value,
    /// JSON array of target [`crate::domain::Registry`] identifiers for package publishing.
    #[serde(default = "default_empty_json_arr")]
    pub release_registries_json: serde_json::Value,
    /// Whether PGP artifact signing is enabled for releases of this repository.
    #[serde(default)]
    pub enable_pgp_signing: bool,
    /// JSON representation of [`crate::domain::DocsPublishConfig`].
    #[serde(default = "default_docs_config_json")]
    pub docs_config_json: serde_json::Value,
}

/// Insertion payload for creating a new [`Repository`] row.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = repositories)]
pub struct NewRepository<'a> {
    /// Foreign key referencing the owning [`Organization`].
    pub organization_id: i32,
    /// Optional upstream GitHub numeric repository ID.
    pub github_id: Option<i64>,
    /// Name of the repository within the organization.
    pub name: &'a str,
    /// Optional human-readable description of the repository.
    pub description: Option<&'a str>,
    /// Upstream GitHub organization or user login (`${GITHUB_ORG_NAME}`).
    pub github_org_name: &'a str,
    /// Repository topology mode (`"monorepo"` or `"polyrepo"`).
    pub topology_mode: &'a str,
    /// Optional custom monorepo name override.
    pub monorepo_name: Option<&'a str>,
    /// JSON map of per-language polyrepo name overrides.
    pub polyrepo_names_json: serde_json::Value,
    /// JSON array of selected target language slugs.
    pub target_languages_json: serde_json::Value,
    /// JSON map of per-language generation options.
    pub language_options_json: serde_json::Value,
    /// JSON array of release registries.
    pub release_registries_json: serde_json::Value,
    /// Whether PGP artifact signing is enabled.
    pub enable_pgp_signing: bool,
    /// JSON documentation hosting configuration.
    pub docs_config_json: serde_json::Value,
}

/// Update changeset for modifying a [`Repository`]'s topology, languages, options, registries, PGP, and docs settings.
#[derive(Debug, Clone, Default, PartialEq, Eq, AsChangeset, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = repositories)]
pub struct UpdateRepositoryConfig {
    /// Optional updated repository name.
    pub name: Option<String>,
    /// Optional updated repository description.
    pub description: Option<String>,
    /// Optional updated `${GITHUB_ORG_NAME}`.
    pub github_org_name: Option<String>,
    /// Optional updated topology mode (`"monorepo"` or `"polyrepo"`).
    pub topology_mode: Option<String>,
    /// Optional updated custom monorepo name.
    pub monorepo_name: Option<String>,
    /// Optional updated polyrepo per-language custom names JSON map.
    pub polyrepo_names_json: Option<serde_json::Value>,
    /// Optional updated target languages JSON array.
    pub target_languages_json: Option<serde_json::Value>,
    /// Optional updated per-language generator options JSON map.
    pub language_options_json: Option<serde_json::Value>,
    /// Optional updated release registries JSON array.
    pub release_registries_json: Option<serde_json::Value>,
    /// Optional updated PGP signing toggle.
    pub enable_pgp_signing: Option<bool>,
    /// Optional updated documentation hosting configuration JSON.
    pub docs_config_json: Option<serde_json::Value>,
}

/// Persistent SDK release record belonging to a [`Repository`].
#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
    Eq,
)]
#[diesel(table_name = releases)]
pub struct Release {
    /// Unique internal primary key for the release.
    pub id: i32,
    /// Foreign key referencing the parent [`Repository`].
    pub repository_id: i32,
    /// Optional upstream GitHub numeric release ID.
    pub github_id: Option<i64>,
    /// Git tag name associated with the release (e.g., `v1.0.0`).
    pub tag_name: String,
    /// Optional release title.
    pub name: Option<String>,
    /// Optional release notes / changelog markdown body.
    pub body: Option<String>,
}

/// Insertion payload for creating a new [`Release`] row.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = releases)]
pub struct NewRelease<'a> {
    /// Foreign key referencing the parent [`Repository`].
    pub repository_id: i32,
    /// Optional upstream GitHub numeric release ID.
    pub github_id: Option<i64>,
    /// Git tag name associated with the release.
    pub tag_name: &'a str,
    /// Optional release title.
    pub name: Option<&'a str>,
    /// Optional release notes / changelog markdown body.
    pub body: Option<&'a str>,
}

/// Persistent encrypted credential record stored on behalf of a [`User`].
#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
    Eq,
)]
#[diesel(table_name = user_tokens)]
pub struct UserToken {
    /// Unique internal primary key for the stored token.
    pub id: i32,
    /// Foreign key referencing the owning [`User`].
    pub user_id: i32,
    /// Canonical lowercase provider identifier (see [`crate::domain::TokenProvider`]).
    pub provider: String,
    /// Base64-encoded `XSalsa20Poly1305` nonce and ciphertext (omitted when serialized).
    #[serde(skip_serializing)]
    pub encrypted_token: String,
    /// UTC timestamp when the token was initially created.
    pub created_at: chrono::NaiveDateTime,
    /// UTC timestamp when the token was last rotated or updated.
    pub updated_at: chrono::NaiveDateTime,
}

/// Insertion payload for storing or upserting a [`UserToken`] row.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_tokens)]
pub struct NewUserToken<'a> {
    /// Foreign key referencing the owning [`User`].
    pub user_id: i32,
    /// Canonical lowercase provider identifier.
    pub provider: &'a str,
    /// Base64-encoded `XSalsa20Poly1305` nonce and ciphertext.
    pub encrypted_token: &'a str,
}

/// Persistent audit trail event recorded for security-sensitive actions within an [`Organization`].
#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
    Eq,
)]
#[diesel(table_name = audit_logs)]
pub struct AuditLog {
    /// Unique internal primary key for the audit event.
    pub id: i32,
    /// Foreign key referencing the [`Organization`] where the event occurred.
    pub org_id: i32,
    /// Optional foreign key referencing the affected [`Repository`].
    pub repo_id: Option<i32>,
    /// Foreign key referencing the acting [`User`].
    pub user_id: i32,
    /// Canonical `snake_case` action identifier (see [`crate::domain::AuditAction`]).
    pub action: String,
    /// Optional structured JSON metadata describing the event details.
    pub metadata_json: Option<serde_json::Value>,
    /// UTC timestamp when the audit event was recorded.
    pub timestamp: chrono::NaiveDateTime,
}

/// Insertion payload for recording a new [`AuditLog`] row.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = audit_logs)]
pub struct NewAuditLog<'a> {
    /// Foreign key referencing the [`Organization`] where the event occurred.
    pub org_id: i32,
    /// Optional foreign key referencing the affected [`Repository`].
    pub repo_id: Option<i32>,
    /// Foreign key referencing the acting [`User`].
    pub user_id: i32,
    /// Canonical `snake_case` action identifier.
    pub action: &'a str,
    /// Optional structured JSON metadata describing the event details.
    pub metadata_json: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    //! Unit tests for database models and serde defaults.

    use super::*;

    #[test]
    fn test_user_connected_providers_and_repository_serde_defaults() -> Result<(), serde_json::Error>
    {
        let user = User {
            id: 1,
            github_id: Some(42),
            username: "alice".into(),
            email: "alice@example.com".into(),
            password_hash: Some("hash".into()),
            google_id: Some("google-1".into()),
            avatar_url: None,
        };
        assert_eq!(
            user.connected_providers(),
            vec![AuthMethod::Password, AuthMethod::GitHub, AuthMethod::Google]
        );

        let repo_json = serde_json::json!({
            "id": 10,
            "organization_id": 2,
            "github_id": null,
            "name": "acme-sdk",
            "description": null
        });
        let repo: Repository = serde_json::from_value(repo_json)?;
        assert_eq!(repo.topology_mode, "monorepo");
        assert_eq!(repo.polyrepo_names_json, serde_json::json!({}));
        assert_eq!(repo.target_languages_json, serde_json::json!([]));
        assert_eq!(
            repo.docs_config_json,
            serde_json::json!({"enabled": true, "target": "official"})
        );
        Ok(())
    }
}
