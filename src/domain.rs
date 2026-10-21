//! Strongly-typed domain primitives, identifiers, finite-state enums, and validated value objects.

use crate::error::Error;
use derive_more::derive::{Display, From};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::ToSchema;

/// Strongly-typed identifier for a [`crate::db::models::User`].
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
    Display,
    From,
)]
pub struct UserId(pub i32);

impl UserId {
    /// Creates a new [`UserId`] from a raw database integer.
    ///
    /// # Arguments
    ///
    /// * `id` - The raw 32-bit integer primary key.
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// Returns the underlying 32-bit integer identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`UserId`] value.
    #[must_use]
    pub const fn into_inner(self) -> i32 {
        self.0
    }
}

/// Strongly-typed identifier for an [`crate::db::models::Organization`].
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
    Display,
    From,
)]
pub struct OrganizationId(pub i32);

impl OrganizationId {
    /// Creates a new [`OrganizationId`] from a raw database integer.
    ///
    /// # Arguments
    ///
    /// * `id` - The raw 32-bit integer primary key.
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// Returns the underlying 32-bit integer identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`OrganizationId`] value.
    #[must_use]
    pub const fn into_inner(self) -> i32 {
        self.0
    }
}

/// Strongly-typed identifier for a [`crate::db::models::Repository`].
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
    Display,
    From,
)]
pub struct RepositoryId(pub i32);

impl RepositoryId {
    /// Creates a new [`RepositoryId`] from a raw database integer.
    ///
    /// # Arguments
    ///
    /// * `id` - The raw 32-bit integer primary key.
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// Returns the underlying 32-bit integer identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`RepositoryId`] value.
    #[must_use]
    pub const fn into_inner(self) -> i32 {
        self.0
    }
}

/// Strongly-typed identifier for a [`crate::db::models::Release`].
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
    Display,
    From,
)]
pub struct ReleaseId(pub i32);

impl ReleaseId {
    /// Creates a new [`ReleaseId`] from a raw database integer.
    ///
    /// # Arguments
    ///
    /// * `id` - The raw 32-bit integer primary key.
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// Returns the underlying 32-bit integer identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`ReleaseId`] value.
    #[must_use]
    pub const fn into_inner(self) -> i32 {
        self.0
    }
}

/// Strongly-typed identifier for a [`crate::db::models::UserToken`].
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
    Display,
    From,
)]
pub struct UserTokenId(pub i32);

impl UserTokenId {
    /// Creates a new [`UserTokenId`] from a raw database integer.
    ///
    /// # Arguments
    ///
    /// * `id` - The raw 32-bit integer primary key.
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// Returns the underlying 32-bit integer identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`UserTokenId`] value.
    #[must_use]
    pub const fn into_inner(self) -> i32 {
        self.0
    }
}

/// Strongly-typed identifier for an [`crate::db::models::AuditLog`].
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
    Display,
    From,
)]
pub struct AuditLogId(pub i32);

impl AuditLogId {
    /// Creates a new [`AuditLogId`] from a raw database integer.
    ///
    /// # Arguments
    ///
    /// * `id` - The raw 32-bit integer primary key.
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// Returns the underlying 32-bit integer identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`AuditLogId`] value.
    #[must_use]
    pub const fn into_inner(self) -> i32 {
        self.0
    }
}

/// Strongly-typed upstream GitHub entity identifier.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
    Display,
    From,
)]
pub struct GitHubId(pub i64);

impl GitHubId {
    /// Creates a new [`GitHubId`] from a raw 64-bit integer.
    ///
    /// # Arguments
    ///
    /// * `id` - The raw 64-bit GitHub identifier.
    #[must_use]
    pub const fn new(id: i64) -> Self {
        Self(id)
    }

    /// Returns the underlying 64-bit GitHub identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`GitHubId`] value.
    #[must_use]
    pub const fn into_inner(self) -> i64 {
        self.0
    }
}

/// Role-Based Access Control (RBAC) membership role within an organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Display)]
#[serde(rename_all = "lowercase")]
pub enum OrgRole {
    /// Full administrative ownership of the organization.
    #[display("owner")]
    Owner,
    /// Administrative privileges to manage repositories and releases.
    #[display("admin")]
    Admin,
    /// Read-only member privileges within the organization.
    #[display("member")]
    Member,
}

impl OrgRole {
    /// Returns the canonical lowercase string representation of the role.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`OrgRole`] instance.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Member => "member",
        }
    }

    /// Returns whether this role is authorized to manage organization settings and members.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`OrgRole`] instance.
    #[must_use]
    pub const fn can_manage_org(&self) -> bool {
        matches!(self, Self::Owner)
    }

    /// Returns whether this role is authorized to create repositories and releases.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`OrgRole`] instance.
    #[must_use]
    pub const fn can_write_repos(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    /// Returns whether this role is authorized to read organization resources and audit logs.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`OrgRole`] instance.
    #[must_use]
    pub const fn can_read(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::Member)
    }
}

impl FromStr for OrgRole {
    type Err = Error;

    /// Parses an [`OrgRole`] from its lowercase string representation.
    ///
    /// # Arguments
    ///
    /// * `s` - The input role string slice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "member" => Ok(Self::Member),
            other => Err(Error::BadRequest(format!(
                "Invalid organization role: {other}"
            ))),
        }
    }
}

/// Supported external registry/VCS providers for user tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Display)]
#[serde(rename_all = "lowercase")]
pub enum TokenProvider {
    /// GitHub OAuth or Personal Access Token.
    #[display("github")]
    GitHub,
    /// NPM registry authentication token.
    #[display("npm")]
    Npm,
    /// `PyPI` registry authentication token.
    #[display("pypi")]
    PyPi,
    /// Cargo (crates.io) registry authentication token.
    #[display("cargo")]
    Cargo,
}

impl TokenProvider {
    /// Returns the canonical lowercase string representation of the provider.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`TokenProvider`] instance.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::Npm => "npm",
            Self::PyPi => "pypi",
            Self::Cargo => "cargo",
        }
    }
}

impl FromStr for TokenProvider {
    type Err = Error;

    /// Parses a [`TokenProvider`] from its string representation.
    ///
    /// # Arguments
    ///
    /// * `s` - The provider name string slice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "github" => Ok(Self::GitHub),
            "npm" => Ok(Self::Npm),
            "pypi" => Ok(Self::PyPi),
            "cargo" => Ok(Self::Cargo),
            other => Err(Error::BadRequest(format!(
                "Unsupported token provider: {other}"
            ))),
        }
    }
}

/// Auditable control-plane events recorded in the `audit_logs` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Display)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    /// An organization was created.
    #[display("org_created")]
    OrgCreated,
    /// A member was added or updated in an organization.
    #[display("member_added")]
    MemberAdded,
    /// A repository was created within an organization.
    #[display("repo_created")]
    RepoCreated,
    /// An SDK release was created and queued for publishing.
    #[display("release_created")]
    ReleaseCreated,
    /// A user token was stored or rotated.
    #[display("token_stored")]
    TokenStored,
    /// A user token was deleted.
    #[display("token_deleted")]
    TokenDeleted,
}

impl AuditAction {
    /// Returns the canonical `snake_case` string representation of the audit action.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`AuditAction`] instance.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::OrgCreated => "org_created",
            Self::MemberAdded => "member_added",
            Self::RepoCreated => "repo_created",
            Self::ReleaseCreated => "release_created",
            Self::TokenStored => "token_stored",
            Self::TokenDeleted => "token_deleted",
        }
    }
}

impl FromStr for AuditAction {
    type Err = Error;

    /// Parses an [`AuditAction`] from its `snake_case` string representation.
    ///
    /// # Arguments
    ///
    /// * `s` - The action string slice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "org_created" => Ok(Self::OrgCreated),
            "member_added" => Ok(Self::MemberAdded),
            "repo_created" => Ok(Self::RepoCreated),
            "release_created" => Ok(Self::ReleaseCreated),
            "token_stored" => Ok(Self::TokenStored),
            "token_deleted" => Ok(Self::TokenDeleted),
            other => Err(Error::BadRequest(format!("Unknown audit action: {other}"))),
        }
    }
}

/// A secret string wrapper that redacts its contents in [`fmt::Debug`] and [`fmt::Display`].
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, From)]
#[serde(transparent)]
pub struct SecretString(String);

impl SecretString {
    /// Creates a new [`SecretString`] from any string-like value.
    ///
    /// # Arguments
    ///
    /// * `value` - The sensitive string value to wrap.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Explicitly exposes the underlying secret string slice for cryptographic operations.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`SecretString`] wrapper.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    /// Formats the secret as `SecretString([REDACTED])` to prevent credential leaks in logs.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString([REDACTED])")
    }
}

impl fmt::Display for SecretString {
    /// Formats the secret as `[REDACTED]` to prevent accidental display leaks.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

/// Validated non-empty password that redacts its contents in [`fmt::Debug`] and [`fmt::Display`].
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "String", into = "String")]
pub struct Password(String);

impl Password {
    /// Validates and constructs a [`Password`], rejecting empty or whitespace-only strings.
    ///
    /// # Arguments
    ///
    /// * `raw` - The candidate password string slice.
    pub fn parse(raw: &str) -> Result<Self, Error> {
        if raw.trim().is_empty() {
            return Err(Error::BadRequest("Password cannot be empty".into()));
        }
        Ok(Self(raw.to_string()))
    }

    /// Exposes the underlying password string slice for Argon2 hashing or verification.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`Password`] instance.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Password {
    /// Formats the password as `Password([REDACTED])` to prevent leaks in logs.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Password([REDACTED])")
    }
}

impl fmt::Display for Password {
    /// Formats the password as `[REDACTED]` to prevent accidental display leaks.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl TryFrom<String> for Password {
    type Error = Error;

    /// Validates and converts an owned [`String`] into a [`Password`].
    ///
    /// # Arguments
    ///
    /// * `value` - The raw password string.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<Password> for String {
    /// Extracts the underlying [`String`] from a [`Password`].
    ///
    /// # Arguments
    ///
    /// * `value` - The [`Password`] instance.
    fn from(value: Password) -> Self {
        value.0
    }
}

/// Validated non-empty username or organization login handle.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Display)]
#[serde(try_from = "String", into = "String")]
pub struct Username(String);

impl Username {
    /// Validates and constructs a [`Username`].
    ///
    /// Must be between 1 and 64 characters and contain only ASCII alphanumeric characters,
    /// hyphens (`-`), or underscores (`_`).
    ///
    /// # Arguments
    ///
    /// * `raw` - The candidate username string.
    pub fn parse(raw: &str) -> Result<Self, Error> {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.len() > 64 {
            return Err(Error::BadRequest(
                "Username must be between 1 and 64 characters".into(),
            ));
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(Error::BadRequest(
                "Username may only contain alphanumeric characters, hyphens, or underscores".into(),
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the validated username string slice.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`Username`] instance.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Username {
    type Error = Error;

    /// Validates and converts an owned [`String`] into a [`Username`].
    ///
    /// # Arguments
    ///
    /// * `value` - The raw string to validate.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<Username> for String {
    /// Extracts the underlying [`String`] from a [`Username`].
    ///
    /// # Arguments
    ///
    /// * `value` - The [`Username`] instance.
    fn from(value: Username) -> Self {
        value.0
    }
}

/// Validated email address.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Display)]
#[serde(try_from = "String", into = "String")]
pub struct Email(String);

impl Email {
    /// Validates and constructs an [`Email`] address.
    ///
    /// # Arguments
    ///
    /// * `raw` - The candidate email address string.
    pub fn parse(raw: &str) -> Result<Self, Error> {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.len() > 254 {
            return Err(Error::BadRequest(
                "Email must be between 1 and 254 characters".into(),
            ));
        }
        let Some((local, domain)) = trimmed.split_once('@') else {
            return Err(Error::BadRequest("Email must contain an '@' symbol".into()));
        };
        if local.is_empty() || domain.is_empty() || !domain.contains('.') {
            return Err(Error::BadRequest("Email format is invalid".into()));
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the validated email string slice.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`Email`] instance.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Email {
    type Error = Error;

    /// Validates and converts an owned [`String`] into an [`Email`].
    ///
    /// # Arguments
    ///
    /// * `value` - The raw string to validate.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<Email> for String {
    /// Extracts the underlying [`String`] from an [`Email`].
    ///
    /// # Arguments
    ///
    /// * `value` - The [`Email`] instance.
    fn from(value: Email) -> Self {
        value.0
    }
}

/// Validated pagination parameters with safe bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Pagination {
    /// Maximum number of records to return (`1..=100`, default `50`).
    pub limit: i64,
    /// Number of records to skip (`>= 0`, default `0`).
    pub offset: i64,
}

impl Pagination {
    /// Default page size when no limit is provided.
    pub const DEFAULT_LIMIT: i64 = 50;
    /// Maximum allowed page size to prevent unbounded queries.
    pub const MAX_LIMIT: i64 = 100;

    /// Validates and constructs [`Pagination`] from optional query parameters.
    ///
    /// Rejects negative `limit`, zero `limit`, `limit > 100`, or negative `offset` with [`Error::BadRequest`].
    ///
    /// # Arguments
    ///
    /// * `limit` - Optional maximum number of rows.
    /// * `offset` - Optional row offset.
    pub fn new(limit: Option<i64>, offset: Option<i64>) -> Result<Self, Error> {
        let resolved_limit = limit.unwrap_or(Self::DEFAULT_LIMIT);
        if !(1..=Self::MAX_LIMIT).contains(&resolved_limit) {
            return Err(Error::BadRequest(format!(
                "Pagination limit must be between 1 and {}",
                Self::MAX_LIMIT
            )));
        }
        let resolved_offset = offset.unwrap_or(0);
        if resolved_offset < 0 {
            return Err(Error::BadRequest(
                "Pagination offset must be non-negative".into(),
            ));
        }
        Ok(Self {
            limit: resolved_limit,
            offset: resolved_offset,
        })
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for domain identifiers, enums, and value objects.

    use super::*;

    #[test]
    fn test_strongly_typed_ids() {
        let uid = UserId::new(10);
        assert_eq!(uid.into_inner(), 10);
        assert_eq!(uid.to_string(), "10");

        let oid = OrganizationId::new(20);
        assert_eq!(oid.into_inner(), 20);

        let rid = RepositoryId::new(30);
        assert_eq!(rid.into_inner(), 30);

        let rel_id = ReleaseId::new(40);
        assert_eq!(rel_id.into_inner(), 40);

        let tok_id = UserTokenId::new(50);
        assert_eq!(tok_id.into_inner(), 50);

        let aud_id = AuditLogId::new(60);
        assert_eq!(aud_id.into_inner(), 60);

        let gh_id = GitHubId::new(7000);
        assert_eq!(gh_id.into_inner(), 7000);
    }

    #[test]
    fn test_org_role_permissions_and_parsing() -> Result<(), Error> {
        assert_eq!(OrgRole::from_str("owner")?, OrgRole::Owner);
        assert_eq!(OrgRole::from_str("ADMIN")?, OrgRole::Admin);
        assert_eq!(OrgRole::from_str(" member ")?, OrgRole::Member);
        assert!(OrgRole::from_str("guest").is_err());

        assert_eq!(OrgRole::Owner.as_str(), "owner");
        assert_eq!(OrgRole::Admin.as_str(), "admin");
        assert_eq!(OrgRole::Member.as_str(), "member");

        assert!(OrgRole::Owner.can_manage_org());
        assert!(!OrgRole::Admin.can_manage_org());
        assert!(!OrgRole::Member.can_manage_org());

        assert!(OrgRole::Owner.can_write_repos());
        assert!(OrgRole::Admin.can_write_repos());
        assert!(!OrgRole::Member.can_write_repos());

        assert!(OrgRole::Owner.can_read());
        assert!(OrgRole::Admin.can_read());
        assert!(OrgRole::Member.can_read());
        Ok(())
    }

    #[test]
    fn test_token_provider_parsing() -> Result<(), Error> {
        assert_eq!(TokenProvider::from_str("github")?, TokenProvider::GitHub);
        assert_eq!(TokenProvider::from_str("NPM")?, TokenProvider::Npm);
        assert_eq!(TokenProvider::from_str("pypi")?, TokenProvider::PyPi);
        assert_eq!(TokenProvider::from_str("cargo")?, TokenProvider::Cargo);
        assert!(TokenProvider::from_str("maven").is_err());

        assert_eq!(TokenProvider::GitHub.as_str(), "github");
        assert_eq!(TokenProvider::Npm.as_str(), "npm");
        assert_eq!(TokenProvider::PyPi.as_str(), "pypi");
        assert_eq!(TokenProvider::Cargo.as_str(), "cargo");
        Ok(())
    }

    #[test]
    fn test_audit_action_parsing() -> Result<(), Error> {
        assert_eq!(
            AuditAction::from_str("org_created")?,
            AuditAction::OrgCreated
        );
        assert_eq!(
            AuditAction::from_str("member_added")?,
            AuditAction::MemberAdded
        );
        assert_eq!(
            AuditAction::from_str("repo_created")?,
            AuditAction::RepoCreated
        );
        assert_eq!(
            AuditAction::from_str("release_created")?,
            AuditAction::ReleaseCreated
        );
        assert_eq!(
            AuditAction::from_str("token_stored")?,
            AuditAction::TokenStored
        );
        assert_eq!(
            AuditAction::from_str("token_deleted")?,
            AuditAction::TokenDeleted
        );
        assert!(AuditAction::from_str("invalid_action").is_err());

        assert_eq!(AuditAction::OrgCreated.as_str(), "org_created");
        assert_eq!(AuditAction::MemberAdded.as_str(), "member_added");
        assert_eq!(AuditAction::RepoCreated.as_str(), "repo_created");
        assert_eq!(AuditAction::ReleaseCreated.as_str(), "release_created");
        assert_eq!(AuditAction::TokenStored.as_str(), "token_stored");
        assert_eq!(AuditAction::TokenDeleted.as_str(), "token_deleted");
        Ok(())
    }

    #[test]
    fn test_secret_string_redaction() {
        let secret = SecretString::new("top-secret-value");
        assert_eq!(secret.expose_secret(), "top-secret-value");
        assert_eq!(format!("{secret:?}"), "SecretString([REDACTED])");
        assert_eq!(format!("{secret}"), "[REDACTED]");
    }

    #[test]
    fn test_password_validation_and_redaction() -> Result<(), Error> {
        let pw = Password::try_from("SuperSecret123!".to_string())?;
        assert_eq!(pw.expose_secret(), "SuperSecret123!");
        assert_eq!(format!("{pw:?}"), "Password([REDACTED])");
        assert_eq!(format!("{pw}"), "[REDACTED]");
        let raw: String = pw.into();
        assert_eq!(raw, "SuperSecret123!");
        assert!(Password::parse("   ").is_err());
        Ok(())
    }

    #[test]
    fn test_username_validation() -> Result<(), Error> {
        let valid = Username::try_from("alice_01-dev".to_string())?;
        assert_eq!(valid.as_str(), "alice_01-dev");
        let into_str: String = valid.into();
        assert_eq!(into_str, "alice_01-dev");

        assert!(Username::parse("").is_err());
        assert!(Username::parse(&"a".repeat(65)).is_err());
        assert!(Username::parse("bad user!").is_err());
        Ok(())
    }

    #[test]
    fn test_email_validation() -> Result<(), Error> {
        let valid = Email::try_from("alice@example.com".to_string())?;
        assert_eq!(valid.as_str(), "alice@example.com");
        let into_str: String = valid.into();
        assert_eq!(into_str, "alice@example.com");

        assert!(Email::parse("").is_err());
        assert!(Email::parse(&format!("{}@example.com", "a".repeat(250))).is_err());
        assert!(Email::parse("no-at-symbol.com").is_err());
        assert!(Email::parse("@example.com").is_err());
        assert!(Email::parse("alice@").is_err());
        assert!(Email::parse("alice@localhost").is_err());
        Ok(())
    }

    #[test]
    fn test_pagination_validation() -> Result<(), Error> {
        let default_page = Pagination::new(None, None)?;
        assert_eq!(default_page.limit, 50);
        assert_eq!(default_page.offset, 0);

        let custom_page = Pagination::new(Some(25), Some(10))?;
        assert_eq!(custom_page.limit, 25);
        assert_eq!(custom_page.offset, 10);

        assert!(Pagination::new(Some(0), None).is_err());
        assert!(Pagination::new(Some(-1), None).is_err());
        assert!(Pagination::new(Some(101), None).is_err());
        assert!(Pagination::new(None, Some(-5)).is_err());
        Ok(())
    }
}
