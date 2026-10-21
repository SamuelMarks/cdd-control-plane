//! Database persistence and insertion models for `cdd-control-plane`.

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::db::schema::{
    audit_logs, organization_users, organizations, releases, repositories, user_tokens, users,
};

/// Persistent user record representing an authenticated account or synced GitHub identity.
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
    /// Name of the repository within the organization.
    pub name: String,
    /// Optional human-readable description of the repository.
    pub description: Option<String>,
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
    /// Canonical lowercase provider identifier (`github`, `npm`, `pypi`, `cargo`).
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
