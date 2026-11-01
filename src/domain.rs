//! Strongly-typed domain primitives, identifiers, finite-state enums, and validated value objects.

use crate::error::Error;
use derive_more::derive::{Display, From};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
    ///
    /// # Returns
    ///
    /// The wrapped [`UserId`] instance.
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// Returns the underlying 32-bit integer identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`UserId`] value.
    ///
    /// # Returns
    ///
    /// The raw 32-bit integer primary key.
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
    ///
    /// # Returns
    ///
    /// The wrapped [`OrganizationId`] instance.
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// Returns the underlying 32-bit integer identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`OrganizationId`] value.
    ///
    /// # Returns
    ///
    /// The raw 32-bit integer primary key.
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
    ///
    /// # Returns
    ///
    /// The wrapped [`RepositoryId`] instance.
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// Returns the underlying 32-bit integer identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`RepositoryId`] value.
    ///
    /// # Returns
    ///
    /// The raw 32-bit integer primary key.
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
    ///
    /// # Returns
    ///
    /// The wrapped [`ReleaseId`] instance.
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// Returns the underlying 32-bit integer identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`ReleaseId`] value.
    ///
    /// # Returns
    ///
    /// The raw 32-bit integer primary key.
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
    ///
    /// # Returns
    ///
    /// The wrapped [`UserTokenId`] instance.
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// Returns the underlying 32-bit integer identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`UserTokenId`] value.
    ///
    /// # Returns
    ///
    /// The raw 32-bit integer primary key.
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
    ///
    /// # Returns
    ///
    /// The wrapped [`AuditLogId`] instance.
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// Returns the underlying 32-bit integer identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`AuditLogId`] value.
    ///
    /// # Returns
    ///
    /// The raw 32-bit integer primary key.
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
    ///
    /// # Returns
    ///
    /// The wrapped [`GitHubId`] instance.
    #[must_use]
    pub const fn new(id: i64) -> Self {
        Self(id)
    }

    /// Returns the underlying 64-bit GitHub identifier.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`GitHubId`] value.
    ///
    /// # Returns
    ///
    /// The raw 64-bit GitHub identifier.
    #[must_use]
    pub const fn into_inner(self) -> i64 {
        self.0
    }
}

/// Validated upstream Google subject identifier (`sub` claim).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema, Display,
)]
#[serde(try_from = "String", into = "String")]
pub struct GoogleId(String);

impl GoogleId {
    /// Validates and constructs a [`GoogleId`].
    ///
    /// Must be between 1 and 255 printable ASCII characters without control characters.
    ///
    /// # Arguments
    ///
    /// * `input` - Raw Google subject identifier string slice.
    ///
    /// # Returns
    ///
    /// The validated [`GoogleId`] value object.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the input is empty, longer than 255 characters,
    /// or contains non-printable ASCII characters.
    pub fn parse(input: &str) -> Result<Self, Error> {
        let trimmed = input.trim();
        if trimmed.is_empty() || trimmed.len() > 255 {
            return Err(Error::BadRequest(
                "GoogleId must be between 1 and 255 characters".into(),
            ));
        }
        if !trimmed.chars().all(|c| c.is_ascii_graphic()) {
            return Err(Error::BadRequest(
                "GoogleId must contain only printable ASCII characters".into(),
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the validated Google ID string slice.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`GoogleId`] instance.
    ///
    /// # Returns
    ///
    /// The underlying Google identifier string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for GoogleId {
    type Err = Error;

    /// Parses a [`GoogleId`] from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Input string slice.
    ///
    /// # Returns
    ///
    /// The validated [`GoogleId`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if validation fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<String> for GoogleId {
    type Error = Error;

    /// Validates and converts an owned [`String`] into a [`GoogleId`].
    ///
    /// # Arguments
    ///
    /// * `value` - Owned string value.
    ///
    /// # Returns
    ///
    /// The validated [`GoogleId`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if validation fails.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<GoogleId> for String {
    /// Extracts the underlying [`String`] from a [`GoogleId`].
    ///
    /// # Arguments
    ///
    /// * `value` - The [`GoogleId`] instance.
    ///
    /// # Returns
    ///
    /// The inner [`String`].
    fn from(value: GoogleId) -> Self {
        value.0
    }
}

/// Supported external OAuth identity providers.
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
)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    /// GitHub OAuth identity provider.
    #[display("github")]
    GitHub,
    /// Google `OpenID` Connect / `OAuth2` identity provider.
    #[display("google")]
    Google,
}

impl OAuthProvider {
    /// Returns the canonical lowercase string representation of the OAuth provider.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`OAuthProvider`] instance.
    ///
    /// # Returns
    ///
    /// Static lowercase provider identifier (`"github"` or `"google"`).
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::Google => "google",
        }
    }
}

impl FromStr for OAuthProvider {
    type Err = Error;

    /// Parses an [`OAuthProvider`] from its string representation.
    ///
    /// # Arguments
    ///
    /// * `s` - Input provider string slice.
    ///
    /// # Returns
    ///
    /// The parsed [`OAuthProvider`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the provider name is unknown.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "github" => Ok(Self::GitHub),
            "google" => Ok(Self::Google),
            other => Err(Error::BadRequest(format!(
                "Unsupported OAuth provider: {other}"
            ))),
        }
    }
}

/// Supported user authentication and account-linking methods.
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
)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    /// Argon2id password-based authentication.
    #[display("password")]
    Password,
    /// GitHub OAuth identity authentication.
    #[display("github")]
    GitHub,
    /// Google OAuth identity authentication.
    #[display("google")]
    Google,
}

impl AuthMethod {
    /// Returns the canonical lowercase string representation of the authentication method.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`AuthMethod`] instance.
    ///
    /// # Returns
    ///
    /// Static lowercase identifier (`"password"`, `"github"`, or `"google"`).
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Password => "password",
            Self::GitHub => "github",
            Self::Google => "google",
        }
    }
}

impl FromStr for AuthMethod {
    type Err = Error;

    /// Parses an [`AuthMethod`] from its string representation.
    ///
    /// # Arguments
    ///
    /// * `s` - Input method string slice.
    ///
    /// # Returns
    ///
    /// The parsed [`AuthMethod`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the authentication method is unknown.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "password" => Ok(Self::Password),
            "github" => Ok(Self::GitHub),
            "google" => Ok(Self::Google),
            other => Err(Error::BadRequest(format!(
                "Unsupported authentication method: {other}"
            ))),
        }
    }
}

/// Validated non-empty OAuth authorization code.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Display)]
#[serde(try_from = "String", into = "String")]
pub struct OAuthCode(String);

impl OAuthCode {
    /// Validates and constructs an [`OAuthCode`], rejecting empty or whitespace-only strings.
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw authorization code string slice.
    ///
    /// # Returns
    ///
    /// The validated [`OAuthCode`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the authorization code is empty or whitespace-only.
    pub fn parse(raw: &str) -> Result<Self, Error> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(Error::BadRequest(
                "OAuth authorization code cannot be empty".into(),
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the validated authorization code string slice.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`OAuthCode`] instance.
    ///
    /// # Returns
    ///
    /// The underlying authorization code string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for OAuthCode {
    type Err = Error;

    /// Parses an [`OAuthCode`] from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Input string slice.
    ///
    /// # Returns
    ///
    /// The validated [`OAuthCode`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the input is empty.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<String> for OAuthCode {
    type Error = Error;

    /// Validates and converts an owned [`String`] into an [`OAuthCode`].
    ///
    /// # Arguments
    ///
    /// * `value` - Owned authorization code string.
    ///
    /// # Returns
    ///
    /// The validated [`OAuthCode`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the input is empty.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<OAuthCode> for String {
    /// Extracts the underlying [`String`] from an [`OAuthCode`].
    ///
    /// # Arguments
    ///
    /// * `value` - The [`OAuthCode`] instance.
    ///
    /// # Returns
    ///
    /// The inner [`String`].
    fn from(value: OAuthCode) -> Self {
        value.0
    }
}

/// Validated GitHub organization or user login handle (`${GITHUB_ORG_NAME}`).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema, Display,
)]
#[serde(try_from = "String", into = "String")]
pub struct GitHubOrgName(String);

impl GitHubOrgName {
    /// Validates and constructs a [`GitHubOrgName`] according to GitHub handle rules:
    /// - Length `1..=39` characters.
    /// - Only ASCII alphanumeric characters or single hyphens (`-`).
    /// - Cannot start or end with a hyphen.
    ///
    /// # Arguments
    ///
    /// * `input` - Candidate GitHub organization name string slice.
    ///
    /// # Returns
    ///
    /// The validated [`GitHubOrgName`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the organization handle violates GitHub naming rules.
    pub fn parse(input: &str) -> Result<Self, Error> {
        let trimmed = input.trim();
        if trimmed.is_empty() || trimmed.len() > 39 {
            return Err(Error::BadRequest(
                "GitHub organization name must be between 1 and 39 characters".into(),
            ));
        }
        if trimmed.starts_with('-') || trimmed.ends_with('-') || trimmed.contains("--") {
            return Err(Error::BadRequest(
                "GitHub organization name cannot start/end with a hyphen or contain consecutive hyphens".into(),
            ));
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err(Error::BadRequest(
                "GitHub organization name may only contain alphanumeric characters or single hyphens".into(),
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the validated GitHub organization name string slice.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`GitHubOrgName`] instance.
    ///
    /// # Returns
    ///
    /// The underlying organization name string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for GitHubOrgName {
    type Err = Error;

    /// Parses a [`GitHubOrgName`] from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Input string slice.
    ///
    /// # Returns
    ///
    /// The validated [`GitHubOrgName`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if validation fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<String> for GitHubOrgName {
    type Error = Error;

    /// Validates and converts an owned [`String`] into a [`GitHubOrgName`].
    ///
    /// # Arguments
    ///
    /// * `value` - Owned string.
    ///
    /// # Returns
    ///
    /// The validated [`GitHubOrgName`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if validation fails.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<GitHubOrgName> for String {
    /// Extracts the underlying [`String`] from a [`GitHubOrgName`].
    ///
    /// # Arguments
    ///
    /// * `value` - The [`GitHubOrgName`] instance.
    ///
    /// # Returns
    ///
    /// The inner [`String`].
    fn from(value: GitHubOrgName) -> Self {
        value.0
    }
}

/// Validated GitHub repository name.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema, Display,
)]
#[serde(try_from = "String", into = "String")]
pub struct RepoName(String);

impl RepoName {
    /// Validates and constructs a [`RepoName`] according to GitHub repository naming rules:
    /// - Length `1..=100` characters.
    /// - Only ASCII alphanumeric characters, hyphens (`-`), underscores (`_`), or periods (`.`).
    /// - Cannot be `"."` or `".."`.
    ///
    /// # Arguments
    ///
    /// * `input` - Candidate repository name string slice.
    ///
    /// # Returns
    ///
    /// The validated [`RepoName`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the repository name violates GitHub naming rules.
    pub fn parse(input: &str) -> Result<Self, Error> {
        let trimmed = input.trim();
        if trimmed.is_empty() || trimmed.len() > 100 {
            return Err(Error::BadRequest(
                "Repository name must be between 1 and 100 characters".into(),
            ));
        }
        if trimmed == "." || trimmed == ".." {
            return Err(Error::BadRequest(
                "Repository name cannot be '.' or '..'".into(),
            ));
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(Error::BadRequest(
                "Repository name may only contain alphanumeric characters, '-', '_', or '.'".into(),
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the validated repository name string slice.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`RepoName`] instance.
    ///
    /// # Returns
    ///
    /// The underlying repository name string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for RepoName {
    type Err = Error;

    /// Parses a [`RepoName`] from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Input string slice.
    ///
    /// # Returns
    ///
    /// The validated [`RepoName`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if validation fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<String> for RepoName {
    type Error = Error;

    /// Validates and converts an owned [`String`] into a [`RepoName`].
    ///
    /// # Arguments
    ///
    /// * `value` - Owned string.
    ///
    /// # Returns
    ///
    /// The validated [`RepoName`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if validation fails.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<RepoName> for String {
    /// Extracts the underlying [`String`] from a [`RepoName`].
    ///
    /// # Arguments
    ///
    /// * `value` - The [`RepoName`] instance.
    ///
    /// # Returns
    ///
    /// The inner [`String`].
    fn from(value: RepoName) -> Self {
        value.0
    }
}

/// Repository layout mode for generated SDKs/servers.
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
)]
#[serde(rename_all = "snake_case")]
pub enum RepoTopologyMode {
    /// Single monorepo containing all target languages (default name: `${GITHUB_ORG_NAME}-sdk`).
    #[display("monorepo")]
    Monorepo,
    /// Separate repository per target language (default names: `${GITHUB_ORG_NAME}-sdk-${LANGUAGE}`).
    #[display("polyrepo")]
    Polyrepo,
}

impl RepoTopologyMode {
    /// Returns the canonical lowercase string representation of the topology mode.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`RepoTopologyMode`] instance.
    ///
    /// # Returns
    ///
    /// Static string slice (`"monorepo"` or `"polyrepo"`).
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Monorepo => "monorepo",
            Self::Polyrepo => "polyrepo",
        }
    }
}

impl FromStr for RepoTopologyMode {
    type Err = Error;

    /// Parses a [`RepoTopologyMode`] from its string representation.
    ///
    /// # Arguments
    ///
    /// * `s` - Input string slice.
    ///
    /// # Returns
    ///
    /// The parsed [`RepoTopologyMode`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the mode is neither `"monorepo"` nor `"polyrepo"`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "monorepo" => Ok(Self::Monorepo),
            "polyrepo" => Ok(Self::Polyrepo),
            other => Err(Error::BadRequest(format!(
                "Invalid repository topology mode: {other}"
            ))),
        }
    }
}

/// All 13 canonical target languages supported by the CDD compiler suite.
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
)]
#[serde(rename_all = "lowercase")]
pub enum TargetLanguage {
    /// C (`cdd-c`).
    #[display("c")]
    C,
    /// C++ (`cdd-cpp`).
    #[display("cpp")]
    Cpp,
    /// C# (`cdd-csharp`).
    #[display("csharp")]
    Csharp,
    /// Go (`cdd-go`).
    #[display("go")]
    Go,
    /// Java (`cdd-java`).
    #[display("java")]
    Java,
    /// Kotlin (`cdd-kotlin`).
    #[display("kotlin")]
    Kotlin,
    /// PHP (`cdd-php`).
    #[display("php")]
    Php,
    /// Python (`cdd-python-all`).
    #[display("python")]
    Python,
    /// Ruby (`cdd-ruby`).
    #[display("ruby")]
    Ruby,
    /// Rust (`cdd-rust`).
    #[display("rust")]
    Rust,
    /// Shell / POSIX sh (`cdd-sh`).
    #[display("sh")]
    Sh,
    /// Swift (`cdd-swift`).
    #[display("swift")]
    Swift,
    /// TypeScript (`cdd-ts`).
    #[display("ts")]
    Ts,
}

/// Static array of all 13 supported [`TargetLanguage`] variants.
const ALL_TARGET_LANGUAGES: [TargetLanguage; 13] = [
    TargetLanguage::C,
    TargetLanguage::Cpp,
    TargetLanguage::Csharp,
    TargetLanguage::Go,
    TargetLanguage::Java,
    TargetLanguage::Kotlin,
    TargetLanguage::Php,
    TargetLanguage::Python,
    TargetLanguage::Ruby,
    TargetLanguage::Rust,
    TargetLanguage::Sh,
    TargetLanguage::Swift,
    TargetLanguage::Ts,
];

impl TargetLanguage {
    /// Returns a reference to the static array of all 13 supported target languages.
    ///
    /// # Returns
    ///
    /// Reference to the 13-element [`TargetLanguage`] array.
    #[must_use]
    pub const fn all() -> &'static [TargetLanguage; 13] {
        &ALL_TARGET_LANGUAGES
    }

    /// Returns the short canonical language slug used in `${GITHUB_ORG_NAME}-sdk-${LANGUAGE}`.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`TargetLanguage`] variant.
    ///
    /// # Returns
    ///
    /// Static lowercase slug (e.g., `"python"`, `"rust"`, `"ts"`).
    #[must_use]
    pub const fn slug(&self) -> &'static str {
        match self {
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Csharp => "csharp",
            Self::Go => "go",
            Self::Java => "java",
            Self::Kotlin => "kotlin",
            Self::Php => "php",
            Self::Python => "python",
            Self::Ruby => "ruby",
            Self::Rust => "rust",
            Self::Sh => "sh",
            Self::Swift => "swift",
            Self::Ts => "ts",
        }
    }

    /// Returns the upstream CDD compiler repository name for this language.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`TargetLanguage`] variant.
    ///
    /// # Returns
    ///
    /// Static compiler repository name (e.g., `"cdd-python-all"`, `"cdd-rust"`, `"cdd-ts"`).
    #[must_use]
    pub const fn compiler_repo(&self) -> &'static str {
        match self {
            Self::C => "cdd-c",
            Self::Cpp => "cdd-cpp",
            Self::Csharp => "cdd-csharp",
            Self::Go => "cdd-go",
            Self::Java => "cdd-java",
            Self::Kotlin => "cdd-kotlin",
            Self::Php => "cdd-php",
            Self::Python => "cdd-python-all",
            Self::Ruby => "cdd-ruby",
            Self::Rust => "cdd-rust",
            Self::Sh => "cdd-sh",
            Self::Swift => "cdd-swift",
            Self::Ts => "cdd-ts",
        }
    }
}

impl FromStr for TargetLanguage {
    type Err = Error;

    /// Parses a [`TargetLanguage`] from either its short slug (`"python"`, `"ts"`)
    /// or its compiler repository name (`"cdd-python-all"`, `"cdd-ts"`).
    ///
    /// # Arguments
    ///
    /// * `s` - Input language string slice.
    ///
    /// # Returns
    ///
    /// The parsed [`TargetLanguage`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the language identifier is unrecognized.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "c" | "cdd-c" => Ok(Self::C),
            "cpp" | "c++" | "cdd-cpp" => Ok(Self::Cpp),
            "csharp" | "c#" | "cs" | "cdd-csharp" => Ok(Self::Csharp),
            "go" | "golang" | "cdd-go" => Ok(Self::Go),
            "java" | "cdd-java" => Ok(Self::Java),
            "kotlin" | "kt" | "cdd-kotlin" => Ok(Self::Kotlin),
            "php" | "cdd-php" => Ok(Self::Php),
            "python" | "py" | "cdd-python" | "cdd-python-all" => Ok(Self::Python),
            "ruby" | "rb" | "cdd-ruby" => Ok(Self::Ruby),
            "rust" | "rs" | "cdd-rust" => Ok(Self::Rust),
            "sh" | "shell" | "bash" | "cdd-sh" => Ok(Self::Sh),
            "swift" | "cdd-swift" => Ok(Self::Swift),
            "ts" | "typescript" | "cdd-ts" => Ok(Self::Ts),
            other => Err(Error::BadRequest(format!(
                "Unsupported target language: {other}"
            ))),
        }
    }
}

/// Resolved repository target produced by expanding a [`RepoTopology`] across selected languages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ResolvedRepoTarget {
    /// Owning GitHub organization or user login (`${GITHUB_ORG_NAME}`).
    pub github_org_name: GitHubOrgName,
    /// Resolved repository name (`${GITHUB_ORG_NAME}-sdk`, `${GITHUB_ORG_NAME}-sdk-${LANGUAGE}`, or custom).
    pub repo_name: RepoName,
    /// Topology mode (`monorepo` or `polyrepo`).
    pub topology_mode: RepoTopologyMode,
    /// Languages generated inside this repository.
    pub languages: Vec<TargetLanguage>,
    /// Optional subdirectory per language (`Some("sdk/<lang>")` in monorepo mode, `None` in polyrepo mode).
    pub sub_directory_by_language: BTreeMap<TargetLanguage, Option<String>>,
}

/// Topology configuration determining whether languages are emitted into a single monorepo
/// or many individual per-language repositories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum RepoTopology {
    /// Single monorepo containing all selected target languages.
    Monorepo {
        /// Optional custom repository name override (defaults to `${GITHUB_ORG_NAME}-sdk`).
        repo_name: Option<RepoName>,
    },
    /// Separate repository per target language.
    Polyrepo {
        /// Optional per-language repository name overrides (each defaults to `${GITHUB_ORG_NAME}-sdk-${LANGUAGE}`).
        repo_names: BTreeMap<TargetLanguage, RepoName>,
    },
}

impl RepoTopology {
    /// Computes the default monorepo name `${GITHUB_ORG_NAME}-sdk`.
    ///
    /// # Arguments
    ///
    /// * `org` - Validated [`GitHubOrgName`].
    ///
    /// # Returns
    ///
    /// Validated [`RepoName`] formatted as `${GITHUB_ORG_NAME}-sdk`.
    #[must_use]
    pub fn default_monorepo_name(org: &GitHubOrgName) -> RepoName {
        RepoName(format!("{}-sdk", org.as_str()))
    }

    /// Computes the default polyrepo name `${GITHUB_ORG_NAME}-sdk-${LANGUAGE}` for a given language.
    ///
    /// # Arguments
    ///
    /// * `org` - Validated [`GitHubOrgName`].
    /// * `lang` - Target [`TargetLanguage`].
    ///
    /// # Returns
    ///
    /// Validated [`RepoName`] formatted as `${GITHUB_ORG_NAME}-sdk-${LANGUAGE}`.
    #[must_use]
    pub fn default_polyrepo_name(org: &GitHubOrgName, lang: TargetLanguage) -> RepoName {
        RepoName(format!("{}-sdk-{}", org.as_str(), lang.slug()))
    }

    /// Resolves the effective monorepo name, using the custom override if configured in [`Self::Monorepo`]
    /// or falling back to [`Self::default_monorepo_name`].
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`RepoTopology`] instance.
    /// * `org` - Validated [`GitHubOrgName`].
    ///
    /// # Returns
    ///
    /// The resolved [`RepoName`].
    #[must_use]
    pub fn resolve_monorepo_name(&self, org: &GitHubOrgName) -> RepoName {
        if let Self::Monorepo {
            repo_name: Some(custom),
        } = self
        {
            custom.clone()
        } else {
            Self::default_monorepo_name(org)
        }
    }

    /// Resolves the effective polyrepo name for `lang`, using the custom override in [`Self::Polyrepo`]
    /// if present or falling back to [`Self::default_polyrepo_name`].
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`RepoTopology`] instance.
    /// * `org` - Validated [`GitHubOrgName`].
    /// * `lang` - Target [`TargetLanguage`].
    ///
    /// # Returns
    ///
    /// The resolved [`RepoName`].
    #[must_use]
    pub fn resolve_polyrepo_name(&self, org: &GitHubOrgName, lang: TargetLanguage) -> RepoName {
        if let Self::Polyrepo { repo_names } = self {
            if let Some(custom) = repo_names.get(&lang) {
                return custom.clone();
            }
        }
        Self::default_polyrepo_name(org, lang)
    }

    /// Expands this topology across the selected `languages` into one or more [`ResolvedRepoTarget`] specs.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`RepoTopology`] instance.
    /// * `org` - Validated [`GitHubOrgName`].
    /// * `languages` - Slice of selected [`TargetLanguage`] targets.
    ///
    /// # Returns
    ///
    /// Vector of [`ResolvedRepoTarget`] descriptors (1 entry for `Monorepo`, `languages.len()` entries for `Polyrepo`).
    #[must_use]
    pub fn expand_targets(
        &self,
        org: &GitHubOrgName,
        languages: &[TargetLanguage],
    ) -> Vec<ResolvedRepoTarget> {
        match self {
            Self::Monorepo { .. } => {
                let repo_name = self.resolve_monorepo_name(org);
                let mut sub_directory_by_language = BTreeMap::new();
                for &lang in languages {
                    sub_directory_by_language.insert(lang, Some(format!("sdk/{}", lang.slug())));
                }
                vec![ResolvedRepoTarget {
                    github_org_name: org.clone(),
                    repo_name,
                    topology_mode: RepoTopologyMode::Monorepo,
                    languages: languages.to_vec(),
                    sub_directory_by_language,
                }]
            }
            Self::Polyrepo { .. } => languages
                .iter()
                .map(|&lang| {
                    let repo_name = self.resolve_polyrepo_name(org, lang);
                    let mut sub_directory_by_language = BTreeMap::new();
                    sub_directory_by_language.insert(lang, None);
                    ResolvedRepoTarget {
                        github_org_name: org.clone(),
                        repo_name,
                        topology_mode: RepoTopologyMode::Polyrepo,
                        languages: vec![lang],
                        sub_directory_by_language,
                    }
                })
                .collect(),
        }
    }
}

/// Code generation subcommand target passed to the language compiler (`cdd-<lang>`).
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
)]
#[serde(rename_all = "snake_case")]
pub enum GeneratorTarget {
    /// Generate client SDK library (`to_sdk`).
    #[display("to_sdk")]
    ToSdk,
    /// Generate CLI wrapping the client SDK (`to_sdk_cli`).
    #[display("to_sdk_cli")]
    ToSdkCli,
    /// Generate backend server scaffolding (`to_server`).
    #[display("to_server")]
    ToServer,
}

impl GeneratorTarget {
    /// Returns the canonical `snake_case` string representation of the generator target.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`GeneratorTarget`] variant.
    ///
    /// # Returns
    ///
    /// Static target identifier (`"to_sdk"`, `"to_sdk_cli"`, or `"to_server"`).
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ToSdk => "to_sdk",
            Self::ToSdkCli => "to_sdk_cli",
            Self::ToServer => "to_server",
        }
    }
}

impl Default for GeneratorTarget {
    /// Returns [`GeneratorTarget::ToSdk`] as the default generation mode.
    ///
    /// # Returns
    ///
    /// [`GeneratorTarget::ToSdk`].
    fn default() -> Self {
        Self::ToSdk
    }
}

impl FromStr for GeneratorTarget {
    type Err = Error;

    /// Parses a [`GeneratorTarget`] from its `snake_case` string representation.
    ///
    /// # Arguments
    ///
    /// * `s` - Input string slice.
    ///
    /// # Returns
    ///
    /// The parsed [`GeneratorTarget`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the target string is invalid.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "to_sdk" => Ok(Self::ToSdk),
            "to_sdk_cli" => Ok(Self::ToSdkCli),
            "to_server" => Ok(Self::ToServer),
            other => Err(Error::BadRequest(format!(
                "Unsupported generator target: {other}"
            ))),
        }
    }
}

/// Per-language compiler options controlling framework, ORM, CLI, and MCP generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct LanguageGenerationOptions {
    /// Primary compilation target (`to_sdk`, `to_sdk_cli`, or `to_server`).
    pub target: GeneratorTarget,
    /// When `true` and `target == ToSdk`, emits CLI bindings (`to_sdk_cli`).
    pub cli: bool,
    /// When `true`, emits Model Context Protocol (`--mcp`) server/tools for this language.
    pub mcp: bool,
    /// Optional target framework (e.g., `"angular"`, `"react"`, `"vue"`, `"actix"`, `"axum"`, `"fastapi"`).
    pub framework: Option<String>,
    /// Optional target ORM/database layer (e.g., `"diesel"`, `"seaorm"`, `"sqlalchemy"`, `"prisma"`, `"typeorm"`).
    pub orm: Option<String>,
    /// When `true`, generates an administrative interface where supported.
    pub auto_admin: bool,
    /// When `true`, generates unit and contract tests alongside the SDK/server.
    pub tests: bool,
    /// When `true`, omits `.github/workflows` CI scaffolding.
    pub no_github_actions: bool,
    /// When `true`, omits package manifest (`Cargo.toml`, `package.json`, `pyproject.toml`, etc.).
    pub no_installable_package: bool,
}

impl Default for LanguageGenerationOptions {
    /// Returns default [`LanguageGenerationOptions`] (`target: ToSdk`, `tests: true`, others `false`/`None`).
    ///
    /// # Returns
    ///
    /// Default [`LanguageGenerationOptions`].
    fn default() -> Self {
        Self {
            target: GeneratorTarget::ToSdk,
            cli: false,
            mcp: false,
            framework: None,
            orm: None,
            auto_admin: false,
            tests: true,
            no_github_actions: false,
            no_installable_package: false,
        }
    }
}

impl LanguageGenerationOptions {
    /// Returns the effective [`GeneratorTarget`] after accounting for the `cli` flag.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`LanguageGenerationOptions`] instance.
    ///
    /// # Returns
    ///
    /// [`GeneratorTarget::ToSdkCli`] if `self.cli` is `true` and `self.target == GeneratorTarget::ToSdk`,
    /// otherwise `self.target`.
    #[must_use]
    pub const fn effective_target(&self) -> GeneratorTarget {
        if self.cli && matches!(self.target, GeneratorTarget::ToSdk) {
            GeneratorTarget::ToSdkCli
        } else {
            self.target
        }
    }

    /// Validates that `framework` and `orm` (if specified) are supported by `lang`.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`LanguageGenerationOptions`] instance.
    /// * `lang` - The [`TargetLanguage`] being configured.
    ///
    /// # Returns
    ///
    /// `Ok(())` if all options are valid for `lang`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if an incompatible or empty `framework` or `orm` is configured.
    pub fn validate_for(&self, lang: TargetLanguage) -> Result<(), Error> {
        if let Some(fw) = self.framework.as_deref() {
            let normalized = fw.trim().to_ascii_lowercase();
            if normalized.is_empty() {
                return Err(Error::BadRequest(
                    "Framework option cannot be empty when provided".into(),
                ));
            }
            let allowed_frameworks: &[&str] = match lang {
                TargetLanguage::Ts => &[
                    "fetch", "axios", "angular", "react", "vue", "express", "fastify", "nestjs",
                ],
                TargetLanguage::Rust => &["reqwest", "actix", "axum", "poem", "rocket", "warp"],
                TargetLanguage::Python => &[
                    "requests", "httpx", "aiohttp", "fastapi", "flask", "django", "bottle",
                ],
                TargetLanguage::Go => &["net_http", "gin", "echo", "fiber", "chi", "gorilla"],
                TargetLanguage::Java => &[
                    "okhttp",
                    "httpclient",
                    "spring",
                    "springboot",
                    "quarkus",
                    "micronau",
                    "micronaut",
                    "javalin",
                ],
                TargetLanguage::Kotlin => &["ktor", "okhttp", "spring", "http4k"],
                TargetLanguage::Csharp => &["httpclient", "aspnetcore", "minimalapi"],
                TargetLanguage::Cpp => &["libcurl", "cpr", "crow", "drogon", "oatpp", "pistache"],
                TargetLanguage::C => &["libcurl", "microhttpd", "mongoose"],
                TargetLanguage::Php => &["guzzle", "laravel", "symfony", "slim"],
                TargetLanguage::Ruby => &["faraday", "net_http", "rails", "sinatra", "hanami"],
                TargetLanguage::Swift => &["urlsession", "alamofire", "vapor", "hummingbird"],
                TargetLanguage::Sh => &["curl", "wget"],
            };
            if !allowed_frameworks.contains(&normalized.as_str()) {
                return Err(Error::BadRequest(format!(
                    "Unsupported framework '{normalized}' for language '{lang}'"
                )));
            }
        }

        if let Some(orm) = self.orm.as_deref() {
            let normalized = orm.trim().to_ascii_lowercase();
            if normalized.is_empty() {
                return Err(Error::BadRequest(
                    "ORM option cannot be empty when provided".into(),
                ));
            }
            let allowed_orms: &[&str] = match lang {
                TargetLanguage::Ts => &["typeorm", "prisma", "drizzle", "sequelize"],
                TargetLanguage::Rust => &["diesel", "seaorm", "sqlx"],
                TargetLanguage::Python => &["sqlalchemy", "sqlmodel", "tortoise", "django"],
                TargetLanguage::Go => &["gorm", "ent", "sqlc"],
                TargetLanguage::Java => &["hibernate", "jpa", "jooq", "mybatis"],
                TargetLanguage::Kotlin => &["exposed", "ktorm", "hibernate"],
                TargetLanguage::Csharp => &["efcore", "dapper", "nhibernate"],
                TargetLanguage::Cpp => &["odb", "sqlpp11", "soci"],
                TargetLanguage::C => &["sqlite3", "libpq"],
                TargetLanguage::Php => &["eloquent", "doctrine"],
                TargetLanguage::Ruby => &["activerecord", "sequel"],
                TargetLanguage::Swift => &["fluent", "grdb", "coredata"],
                TargetLanguage::Sh => &["psql", "sqlite3"],
            };
            if !allowed_orms.contains(&normalized.as_str()) {
                return Err(Error::BadRequest(format!(
                    "Unsupported ORM '{normalized}' for language '{lang}'"
                )));
            }
        }

        Ok(())
    }
}

/// Supported external registry, VCS, documentation, and PGP credential providers stored in `user_tokens`.
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
)]
#[serde(rename_all = "snake_case")]
pub enum TokenProvider {
    /// GitHub OAuth or Personal Access Token.
    #[display("github")]
    GitHub,
    /// GitLab Personal or Project Access Token.
    #[display("gitlab")]
    GitLab,
    /// Bitbucket App Password or Access Token.
    #[display("bitbucket")]
    Bitbucket,
    /// NPM registry authentication token (`npmjs.com`).
    #[display("npm")]
    Npm,
    /// `PyPI` registry authentication token (`pypi.org`).
    #[display("pypi")]
    PyPi,
    /// Cargo (`crates.io`) registry authentication token.
    #[display("cargo")]
    #[serde(alias = "crates")]
    Cargo,
    /// Maven Central / Sonatype Central Portal token (`maven`).
    #[display("maven")]
    Maven,
    /// `NuGet` gallery API key (`nuget.org`).
    #[display("nuget")]
    NuGet,
    /// `RubyGems` API key (`rubygems.org`).
    #[display("rubygems")]
    RubyGems,
    /// Packagist API token (`packagist.org`).
    #[display("packagist")]
    Packagist,
    /// Conan Center / Artifactory token (`conan`).
    #[display("conan")]
    Conan,
    /// Microsoft `vcpkg` registry token (`vcpkg`).
    #[display("vcpkg")]
    Vcpkg,
    /// Homebrew tap token (`homebrew`).
    #[display("homebrew")]
    Homebrew,
    /// `ReadTheDocs` API token (`readthedocs.org`).
    #[display("readthedocs")]
    ReadTheDocs,
    /// Armored PGP private key used to sign release artifacts (`pgp_private_key`).
    #[display("pgp_private_key")]
    PgpPrivateKey,
    /// Passphrase for unlocking the stored PGP private key (`pgp_passphrase`).
    #[display("pgp_passphrase")]
    PgpPassphrase,
}

impl TokenProvider {
    /// Returns the canonical lowercase string representation of the provider.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`TokenProvider`] instance.
    ///
    /// # Returns
    ///
    /// Static provider identifier string slice.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::GitLab => "gitlab",
            Self::Bitbucket => "bitbucket",
            Self::Npm => "npm",
            Self::PyPi => "pypi",
            Self::Cargo => "cargo",
            Self::Maven => "maven",
            Self::NuGet => "nuget",
            Self::RubyGems => "rubygems",
            Self::Packagist => "packagist",
            Self::Conan => "conan",
            Self::Vcpkg => "vcpkg",
            Self::Homebrew => "homebrew",
            Self::ReadTheDocs => "readthedocs",
            Self::PgpPrivateKey => "pgp_private_key",
            Self::PgpPassphrase => "pgp_passphrase",
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
    ///
    /// # Returns
    ///
    /// The validated [`TokenProvider`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the provider name is unsupported.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "github" => Ok(Self::GitHub),
            "gitlab" => Ok(Self::GitLab),
            "bitbucket" => Ok(Self::Bitbucket),
            "npm" => Ok(Self::Npm),
            "pypi" => Ok(Self::PyPi),
            "cargo" | "crates" | "crates_io" => Ok(Self::Cargo),
            "maven" | "maven_central" => Ok(Self::Maven),
            "nuget" => Ok(Self::NuGet),
            "rubygems" => Ok(Self::RubyGems),
            "packagist" => Ok(Self::Packagist),
            "conan" => Ok(Self::Conan),
            "vcpkg" => Ok(Self::Vcpkg),
            "homebrew" | "brew" => Ok(Self::Homebrew),
            "readthedocs" => Ok(Self::ReadTheDocs),
            "pgp_private_key" => Ok(Self::PgpPrivateKey),
            "pgp_passphrase" => Ok(Self::PgpPassphrase),
            other => Err(Error::BadRequest(format!(
                "Unsupported token provider: {other}"
            ))),
        }
    }
}

/// Package registries and distribution channels supported across all 13 target languages.
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
)]
#[serde(rename_all = "snake_case")]
pub enum Registry {
    /// Rust `crates.io` registry.
    #[display("crates_io")]
    CratesIo,
    /// JavaScript/TypeScript `npmjs.com` registry.
    #[display("npm")]
    Npm,
    /// Python `pypi.org` package index.
    #[display("pypi")]
    PyPi,
    /// Java/Kotlin Maven Central repository.
    #[display("maven_central")]
    MavenCentral,
    /// .NET / C# `nuget.org` gallery.
    #[display("nuget")]
    NuGet,
    /// Ruby `rubygems.org` registry.
    #[display("rubygems")]
    RubyGems,
    /// PHP `packagist.org` repository.
    #[display("packagist")]
    Packagist,
    /// Go `pkg.go.dev` module proxy/index.
    #[display("pkg_go_dev")]
    PkgGoDev,
    /// Swift Package Index (`swiftpackageindex.com`).
    #[display("swift_package_index")]
    SwiftPackageIndex,
    /// C/C++ `vcpkg` port registry.
    #[display("vcpkg")]
    Vcpkg,
    /// C/C++ Conan Center package manager.
    #[display("conan")]
    Conan,
    /// macOS/Linux Homebrew tap distribution.
    #[display("homebrew")]
    Homebrew,
    /// GitHub Releases binary & archive distribution.
    #[display("github_releases")]
    GithubReleases,
}

impl Registry {
    /// Returns the canonical `snake_case` string representation of the registry.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`Registry`] variant.
    ///
    /// # Returns
    ///
    /// Static registry identifier string slice.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::CratesIo => "crates_io",
            Self::Npm => "npm",
            Self::PyPi => "pypi",
            Self::MavenCentral => "maven_central",
            Self::NuGet => "nuget",
            Self::RubyGems => "rubygems",
            Self::Packagist => "packagist",
            Self::PkgGoDev => "pkg_go_dev",
            Self::SwiftPackageIndex => "swift_package_index",
            Self::Vcpkg => "vcpkg",
            Self::Conan => "conan",
            Self::Homebrew => "homebrew",
            Self::GithubReleases => "github_releases",
        }
    }

    /// Maps this [`Registry`] to the [`TokenProvider`] credential required in `user_tokens` to publish.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`Registry`] variant.
    ///
    /// # Returns
    ///
    /// The required [`TokenProvider`].
    #[must_use]
    pub const fn required_token_provider(&self) -> TokenProvider {
        match self {
            Self::CratesIo => TokenProvider::Cargo,
            Self::Npm => TokenProvider::Npm,
            Self::PyPi => TokenProvider::PyPi,
            Self::MavenCentral => TokenProvider::Maven,
            Self::NuGet => TokenProvider::NuGet,
            Self::RubyGems => TokenProvider::RubyGems,
            Self::Packagist => TokenProvider::Packagist,
            Self::Vcpkg => TokenProvider::Vcpkg,
            Self::Conan => TokenProvider::Conan,
            Self::Homebrew => TokenProvider::Homebrew,
            Self::PkgGoDev | Self::SwiftPackageIndex | Self::GithubReleases => {
                TokenProvider::GitHub
            }
        }
    }

    /// Returns the default package registries associated with a given [`TargetLanguage`].
    ///
    /// # Arguments
    ///
    /// * `lang` - The [`TargetLanguage`].
    ///
    /// # Returns
    ///
    /// Static slice of default [`Registry`] targets for `lang`.
    #[must_use]
    pub const fn default_for_language(lang: TargetLanguage) -> &'static [Registry] {
        match lang {
            TargetLanguage::C | TargetLanguage::Cpp => {
                &[Self::Vcpkg, Self::Conan, Self::GithubReleases]
            }
            TargetLanguage::Csharp => &[Self::NuGet],
            TargetLanguage::Go => &[Self::PkgGoDev, Self::GithubReleases],
            TargetLanguage::Java | TargetLanguage::Kotlin => &[Self::MavenCentral],
            TargetLanguage::Php => &[Self::Packagist],
            TargetLanguage::Python => &[Self::PyPi],
            TargetLanguage::Ruby => &[Self::RubyGems],
            TargetLanguage::Rust => &[Self::CratesIo],
            TargetLanguage::Sh => &[Self::Homebrew, Self::GithubReleases],
            TargetLanguage::Swift => &[Self::SwiftPackageIndex, Self::GithubReleases],
            TargetLanguage::Ts => &[Self::Npm],
        }
    }
}

impl FromStr for Registry {
    type Err = Error;

    /// Parses a [`Registry`] from its string representation.
    ///
    /// # Arguments
    ///
    /// * `s` - Input registry string slice.
    ///
    /// # Returns
    ///
    /// The parsed [`Registry`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the registry identifier is unrecognized.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "crates_io" | "crates" | "cargo" => Ok(Self::CratesIo),
            "npm" => Ok(Self::Npm),
            "pypi" => Ok(Self::PyPi),
            "maven_central" | "maven" => Ok(Self::MavenCentral),
            "nuget" => Ok(Self::NuGet),
            "rubygems" => Ok(Self::RubyGems),
            "packagist" => Ok(Self::Packagist),
            "pkg_go_dev" | "pkggodev" => Ok(Self::PkgGoDev),
            "swift_package_index" | "spi" => Ok(Self::SwiftPackageIndex),
            "vcpkg" => Ok(Self::Vcpkg),
            "conan" => Ok(Self::Conan),
            "homebrew" | "brew" => Ok(Self::Homebrew),
            "github_releases" | "github" => Ok(Self::GithubReleases),
            other => Err(Error::BadRequest(format!(
                "Unsupported package registry: {other}"
            ))),
        }
    }
}

/// Target hosting platform for generated SDK/API documentation.
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
)]
pub enum DocHostTarget {
    /// Official CDD hosted documentation portal (`cdd-storage` + `cdd-docs-ui`).
    #[display("official")]
    #[serde(rename = "official", alias = "official_cdd_site")]
    OfficialCddSite,
    /// GitHub Pages (`gh-pages` branch or Actions artifact).
    #[display("github_pages")]
    #[serde(rename = "github_pages")]
    GithubPages,
    /// `ReadTheDocs` (`readthedocs.org`).
    #[display("readthedocs")]
    #[serde(rename = "readthedocs")]
    ReadTheDocs,
    /// Rust `docs.rs` documentation host.
    #[display("docs_rs")]
    #[serde(rename = "docs_rs")]
    DocsRs,
    /// Go `pkg.go.dev` documentation host.
    #[display("pkg_go_dev")]
    #[serde(rename = "pkg_go_dev")]
    PkgGoDev,
    /// Java/Kotlin `javadoc.io` documentation host.
    #[display("javadoc_io")]
    #[serde(rename = "javadoc_io")]
    JavadocIo,
    /// Ruby `rubydoc.info` documentation host.
    #[display("rubydoc")]
    #[serde(rename = "rubydoc")]
    RubyDoc,
    /// Swift Package Index hosted `DocC` documentation (`spi_docc`).
    #[display("spi_docc")]
    #[serde(rename = "spi_docc")]
    SwiftPackageIndexDocs,
    /// User-specified custom documentation webhook/URL.
    #[display("custom")]
    #[serde(rename = "custom")]
    CustomUrl,
    /// Documentation hosting disabled.
    #[display("none")]
    #[serde(rename = "none")]
    None,
}

impl DocHostTarget {
    /// Returns the canonical string identifier of the documentation host target.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`DocHostTarget`] variant.
    ///
    /// # Returns
    ///
    /// Static identifier string slice.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::OfficialCddSite => "official",
            Self::GithubPages => "github_pages",
            Self::ReadTheDocs => "readthedocs",
            Self::DocsRs => "docs_rs",
            Self::PkgGoDev => "pkg_go_dev",
            Self::JavadocIo => "javadoc_io",
            Self::RubyDoc => "rubydoc",
            Self::SwiftPackageIndexDocs => "spi_docc",
            Self::CustomUrl => "custom",
            Self::None => "none",
        }
    }
}

impl Default for DocHostTarget {
    /// Returns [`DocHostTarget::OfficialCddSite`] as the default documentation host.
    ///
    /// # Returns
    ///
    /// [`DocHostTarget::OfficialCddSite`].
    fn default() -> Self {
        Self::OfficialCddSite
    }
}

impl FromStr for DocHostTarget {
    type Err = Error;

    /// Parses a [`DocHostTarget`] from its string representation.
    ///
    /// # Arguments
    ///
    /// * `s` - Input string slice.
    ///
    /// # Returns
    ///
    /// The parsed [`DocHostTarget`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the target identifier is unrecognized.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "official" | "official_cdd_site" => Ok(Self::OfficialCddSite),
            "github_pages" | "gh_pages" => Ok(Self::GithubPages),
            "readthedocs" | "rtd" => Ok(Self::ReadTheDocs),
            "docs_rs" | "docs.rs" => Ok(Self::DocsRs),
            "pkg_go_dev" | "pkg.go.dev" => Ok(Self::PkgGoDev),
            "javadoc_io" | "javadoc.io" => Ok(Self::JavadocIo),
            "rubydoc" | "rubydoc.info" => Ok(Self::RubyDoc),
            "spi_docc" | "swift_package_index_docs" => Ok(Self::SwiftPackageIndexDocs),
            "custom" | "custom_url" => Ok(Self::CustomUrl),
            "none" => Ok(Self::None),
            other => Err(Error::BadRequest(format!(
                "Unsupported documentation host target: {other}"
            ))),
        }
    }
}

/// Configuration for publishing SDK and API documentation upon release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(default)]
pub struct DocsPublishConfig {
    /// Whether documentation publishing is enabled for this repository.
    pub enabled: bool,
    /// Target documentation hosting platform.
    pub target: DocHostTarget,
    /// Required custom URL when `target == DocHostTarget::CustomUrl`.
    pub custom_url: Option<String>,
    /// Target branch when `target == DocHostTarget::GithubPages` (defaults to `"gh-pages"`).
    pub github_pages_branch: Option<String>,
}

impl Default for DocsPublishConfig {
    /// Returns the default [`DocsPublishConfig`] (`enabled: true`, `target: OfficialCddSite`).
    ///
    /// # Returns
    ///
    /// Default [`DocsPublishConfig`].
    fn default() -> Self {
        Self {
            enabled: true,
            target: DocHostTarget::OfficialCddSite,
            custom_url: None,
            github_pages_branch: None,
        }
    }
}

impl DocsPublishConfig {
    /// Validates the consistency of [`DocsPublishConfig`].
    ///
    /// When `enabled` is `true` and `target == DocHostTarget::CustomUrl`, requires a non-empty
    /// `http://` or `https://` `custom_url`.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`DocsPublishConfig`] instance.
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if `target == DocHostTarget::CustomUrl` and `custom_url` is missing or invalid.
    pub fn validate(&self) -> Result<(), Error> {
        if self.enabled && self.target == DocHostTarget::CustomUrl {
            let Some(url) = self.custom_url.as_deref() else {
                return Err(Error::BadRequest(
                    "custom_url is required when docs target is 'custom'".into(),
                ));
            };
            let trimmed = url.trim();
            if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
                return Err(Error::BadRequest(
                    "custom_url must start with 'http://' or 'https://'".into(),
                ));
            }
        }
        Ok(())
    }

    /// Returns the effective GitHub Pages branch (`"gh-pages"` if not explicitly overridden).
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`DocsPublishConfig`] instance.
    ///
    /// # Returns
    ///
    /// Branch name string slice.
    #[must_use]
    pub fn resolved_github_pages_branch(&self) -> &str {
        self.github_pages_branch
            .as_deref()
            .filter(|b| !b.trim().is_empty())
            .unwrap_or("gh-pages")
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
    ///
    /// # Returns
    ///
    /// Static lowercase role string (`"owner"`, `"admin"`, or `"member"`).
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
    ///
    /// # Returns
    ///
    /// `true` if `Self::Owner`, otherwise `false`.
    #[must_use]
    pub const fn can_manage_org(&self) -> bool {
        matches!(self, Self::Owner)
    }

    /// Returns whether this role is authorized to create repositories and releases.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`OrgRole`] instance.
    ///
    /// # Returns
    ///
    /// `true` if `Self::Owner` or `Self::Admin`, otherwise `false`.
    #[must_use]
    pub const fn can_write_repos(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    /// Returns whether this role is authorized to read organization resources and audit logs.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`OrgRole`] instance.
    ///
    /// # Returns
    ///
    /// `true` for all organization members (`Owner`, `Admin`, `Member`).
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
    ///
    /// # Returns
    ///
    /// The validated [`OrgRole`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the role is not `"owner"`, `"admin"`, or `"member"`.
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

/// Auditable control-plane events recorded in the `audit_logs` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Display)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    /// A new user registered an account.
    #[display("user_registered")]
    UserRegistered,
    /// An external identity or password credential was linked to a user account.
    #[display("account_connected")]
    AccountConnected,
    /// An external identity or password credential was unlinked from a user account.
    #[display("account_disconnected")]
    AccountDisconnected,
    /// An organization was created.
    #[display("org_created")]
    OrgCreated,
    /// A member was added or updated in an organization.
    #[display("member_added")]
    MemberAdded,
    /// A repository was created within an organization.
    #[display("repo_created")]
    RepoCreated,
    /// A repository's topology, language, registry, or documentation configuration was updated.
    #[display("repo_config_updated")]
    RepoConfigUpdated,
    /// Project repositories were provisioned (monorepo or polyrepo).
    #[display("project_provisioned")]
    ProjectProvisioned,
    /// An SDK release was created and queued for publishing.
    #[display("release_created")]
    ReleaseCreated,
    /// Documentation was published or queued for hosting.
    #[display("docs_published")]
    DocsPublished,
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
    ///
    /// # Returns
    ///
    /// Static `snake_case` action identifier.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::UserRegistered => "user_registered",
            Self::AccountConnected => "account_connected",
            Self::AccountDisconnected => "account_disconnected",
            Self::OrgCreated => "org_created",
            Self::MemberAdded => "member_added",
            Self::RepoCreated => "repo_created",
            Self::RepoConfigUpdated => "repo_config_updated",
            Self::ProjectProvisioned => "project_provisioned",
            Self::ReleaseCreated => "release_created",
            Self::DocsPublished => "docs_published",
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
    ///
    /// # Returns
    ///
    /// The parsed [`AuditAction`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the audit action is unrecognized.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "user_registered" => Ok(Self::UserRegistered),
            "account_connected" => Ok(Self::AccountConnected),
            "account_disconnected" => Ok(Self::AccountDisconnected),
            "org_created" => Ok(Self::OrgCreated),
            "member_added" => Ok(Self::MemberAdded),
            "repo_created" => Ok(Self::RepoCreated),
            "repo_config_updated" => Ok(Self::RepoConfigUpdated),
            "project_provisioned" => Ok(Self::ProjectProvisioned),
            "release_created" => Ok(Self::ReleaseCreated),
            "docs_published" => Ok(Self::DocsPublished),
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
    ///
    /// # Returns
    ///
    /// The wrapped [`SecretString`].
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Explicitly exposes the underlying secret string slice for cryptographic operations.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`SecretString`] wrapper.
    ///
    /// # Returns
    ///
    /// The raw secret string slice.
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
    ///
    /// # Returns
    ///
    /// [`fmt::Result`].
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
    ///
    /// # Returns
    ///
    /// [`fmt::Result`].
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
    ///
    /// # Returns
    ///
    /// The validated [`Password`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the password is empty or contains only whitespace.
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
    ///
    /// # Returns
    ///
    /// The raw password string slice.
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
    ///
    /// # Returns
    ///
    /// [`fmt::Result`].
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
    ///
    /// # Returns
    ///
    /// [`fmt::Result`].
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
    ///
    /// # Returns
    ///
    /// The validated [`Password`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the password is empty.
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
    ///
    /// # Returns
    ///
    /// The inner [`String`].
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
    ///
    /// # Returns
    ///
    /// The validated [`Username`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the username length is outside `1..=64` or contains invalid characters.
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
    ///
    /// # Returns
    ///
    /// The underlying username string slice.
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
    ///
    /// # Returns
    ///
    /// The validated [`Username`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if validation fails.
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
    ///
    /// # Returns
    ///
    /// The inner [`String`].
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
    ///
    /// # Returns
    ///
    /// The validated [`Email`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the email length is outside `1..=254` or lacks a valid `local@domain.tld` structure.
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
    ///
    /// # Returns
    ///
    /// The underlying email string slice.
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
    ///
    /// # Returns
    ///
    /// The validated [`Email`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if validation fails.
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
    ///
    /// # Returns
    ///
    /// The inner [`String`].
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
    ///
    /// # Returns
    ///
    /// The validated [`Pagination`] parameters.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if `limit` is outside `1..=100` or `offset < 0`.
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
    //! Unit tests for domain identifiers, enums, topologies, languages, and value objects.

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
    fn test_google_id_oauth_provider_auth_method_and_code() -> Result<(), Error> {
        let gid = GoogleId::from_str("  google-sub-12345  ")?;
        assert_eq!(gid.as_str(), "google-sub-12345");
        let gid_try = GoogleId::try_from("google-sub-67890".to_string())?;
        let gid_str: String = gid_try.into();
        assert_eq!(gid_str, "google-sub-67890");

        assert!(GoogleId::parse("").is_err());
        assert!(GoogleId::parse(&"a".repeat(256)).is_err());
        assert!(GoogleId::parse("bad space").is_err());

        assert_eq!(OAuthProvider::from_str("github")?, OAuthProvider::GitHub);
        assert_eq!(OAuthProvider::from_str("GOOGLE")?, OAuthProvider::Google);
        assert_eq!(OAuthProvider::GitHub.as_str(), "github");
        assert_eq!(OAuthProvider::Google.as_str(), "google");
        assert!(OAuthProvider::from_str("gitlab").is_err());

        assert_eq!(AuthMethod::from_str("password")?, AuthMethod::Password);
        assert_eq!(AuthMethod::from_str("github")?, AuthMethod::GitHub);
        assert_eq!(AuthMethod::from_str("GOOGLE")?, AuthMethod::Google);
        assert_eq!(AuthMethod::Password.as_str(), "password");
        assert_eq!(AuthMethod::GitHub.as_str(), "github");
        assert_eq!(AuthMethod::Google.as_str(), "google");
        assert!(AuthMethod::from_str("saml").is_err());

        let code = OAuthCode::from_str("  auth_code_xyz  ")?;
        assert_eq!(code.as_str(), "auth_code_xyz");
        let code_try = OAuthCode::try_from("code_2".to_string())?;
        let code_raw: String = code_try.into();
        assert_eq!(code_raw, "code_2");
        assert!(OAuthCode::parse("   ").is_err());

        Ok(())
    }

    #[test]
    fn test_github_org_name_repo_name_and_topology() -> Result<(), Error> {
        let org = GitHubOrgName::from_str("Acme-Corp-01")?;
        assert_eq!(org.as_str(), "Acme-Corp-01");
        let org_try = GitHubOrgName::try_from("SamuelMarks".to_string())?;
        let org_raw: String = org_try.into();
        assert_eq!(org_raw, "SamuelMarks");

        assert!(GitHubOrgName::parse("").is_err());
        assert!(GitHubOrgName::parse(&"a".repeat(40)).is_err());
        assert!(GitHubOrgName::parse("-leading").is_err());
        assert!(GitHubOrgName::parse("trailing-").is_err());
        assert!(GitHubOrgName::parse("double--hyphen").is_err());
        assert!(GitHubOrgName::parse("bad_underscore").is_err());

        let rn = RepoName::from_str("my_repo-1.0")?;
        assert_eq!(rn.as_str(), "my_repo-1.0");
        let rn_try = RepoName::try_from("custom-sdk".to_string())?;
        let rn_raw: String = rn_try.into();
        assert_eq!(rn_raw, "custom-sdk");

        assert!(RepoName::parse("").is_err());
        assert!(RepoName::parse(&"a".repeat(101)).is_err());
        assert!(RepoName::parse(".").is_err());
        assert!(RepoName::parse("..").is_err());
        assert!(RepoName::parse("bad/slash").is_err());

        assert_eq!(
            RepoTopologyMode::from_str("monorepo")?,
            RepoTopologyMode::Monorepo
        );
        assert_eq!(
            RepoTopologyMode::from_str("POLYREPO")?,
            RepoTopologyMode::Polyrepo
        );
        assert_eq!(RepoTopologyMode::Monorepo.as_str(), "monorepo");
        assert_eq!(RepoTopologyMode::Polyrepo.as_str(), "polyrepo");
        assert!(RepoTopologyMode::from_str("hybrid").is_err());

        // Default Monorepo name: ${GITHUB_ORG_NAME}-sdk
        let mono_default = RepoTopology::Monorepo { repo_name: None };
        assert_eq!(
            mono_default.resolve_monorepo_name(&org).as_str(),
            "Acme-Corp-01-sdk"
        );
        // Fallback when resolve_polyrepo_name is called on Monorepo variant
        assert_eq!(
            mono_default
                .resolve_polyrepo_name(&org, TargetLanguage::Rust)
                .as_str(),
            "Acme-Corp-01-sdk-rust"
        );

        let expanded_mono =
            mono_default.expand_targets(&org, &[TargetLanguage::Rust, TargetLanguage::Python]);
        assert_eq!(expanded_mono.len(), 1);
        assert_eq!(expanded_mono[0].repo_name.as_str(), "Acme-Corp-01-sdk");
        assert_eq!(expanded_mono[0].topology_mode, RepoTopologyMode::Monorepo);
        assert_eq!(
            expanded_mono[0]
                .sub_directory_by_language
                .get(&TargetLanguage::Rust),
            Some(&Some("sdk/rust".to_string()))
        );

        // Custom Monorepo name override
        let mono_custom = RepoTopology::Monorepo {
            repo_name: Some(RepoName::parse("acme-unified-sdk")?),
        };
        assert_eq!(
            mono_custom.resolve_monorepo_name(&org).as_str(),
            "acme-unified-sdk"
        );

        // Polyrepo default names (${GITHUB_ORG_NAME}-sdk-${LANGUAGE}) + custom override
        let mut custom_map = BTreeMap::new();
        custom_map.insert(
            TargetLanguage::Python,
            RepoName::parse("acme-python-client")?,
        );
        let poly = RepoTopology::Polyrepo {
            repo_names: custom_map,
        };
        // Fallback when resolve_monorepo_name is called on Polyrepo variant
        assert_eq!(
            poly.resolve_monorepo_name(&org).as_str(),
            "Acme-Corp-01-sdk"
        );

        for &lang in TargetLanguage::all() {
            let resolved = poly.resolve_polyrepo_name(&org, lang);
            if lang == TargetLanguage::Python {
                assert_eq!(resolved.as_str(), "acme-python-client");
            } else {
                assert_eq!(
                    resolved.as_str(),
                    format!("Acme-Corp-01-sdk-{}", lang.slug())
                );
            }
        }

        let expanded_poly = poly.expand_targets(&org, TargetLanguage::all());
        assert_eq!(expanded_poly.len(), 13);
        assert_eq!(expanded_poly[0].topology_mode, RepoTopologyMode::Polyrepo);
        assert_eq!(expanded_poly[0].repo_name.as_str(), "Acme-Corp-01-sdk-c");
        assert_eq!(expanded_poly[7].repo_name.as_str(), "acme-python-client");

        Ok(())
    }

    #[test]
    fn test_target_languages_and_generation_options() -> Result<(), Error> {
        let all = TargetLanguage::all();
        assert_eq!(all.len(), 13);

        let expected = [
            (TargetLanguage::C, "c", "cdd-c"),
            (TargetLanguage::Cpp, "cpp", "cdd-cpp"),
            (TargetLanguage::Csharp, "csharp", "cdd-csharp"),
            (TargetLanguage::Go, "go", "cdd-go"),
            (TargetLanguage::Java, "java", "cdd-java"),
            (TargetLanguage::Kotlin, "kotlin", "cdd-kotlin"),
            (TargetLanguage::Php, "php", "cdd-php"),
            (TargetLanguage::Python, "python", "cdd-python-all"),
            (TargetLanguage::Ruby, "ruby", "cdd-ruby"),
            (TargetLanguage::Rust, "rust", "cdd-rust"),
            (TargetLanguage::Sh, "sh", "cdd-sh"),
            (TargetLanguage::Swift, "swift", "cdd-swift"),
            (TargetLanguage::Ts, "ts", "cdd-ts"),
        ];

        for (lang, slug, repo) in expected {
            assert_eq!(lang.slug(), slug);
            assert_eq!(lang.compiler_repo(), repo);
            assert_eq!(TargetLanguage::from_str(slug)?, lang);
            assert_eq!(TargetLanguage::from_str(repo)?, lang);
        }
        assert!(TargetLanguage::from_str("fortran").is_err());

        assert_eq!(GeneratorTarget::default(), GeneratorTarget::ToSdk);
        assert_eq!(GeneratorTarget::ToSdk.as_str(), "to_sdk");
        assert_eq!(GeneratorTarget::ToSdkCli.as_str(), "to_sdk_cli");
        assert_eq!(GeneratorTarget::ToServer.as_str(), "to_server");
        assert_eq!(GeneratorTarget::from_str("to_sdk")?, GeneratorTarget::ToSdk);
        assert_eq!(
            GeneratorTarget::from_str("to_sdk_cli")?,
            GeneratorTarget::ToSdkCli
        );
        assert_eq!(
            GeneratorTarget::from_str("to_server")?,
            GeneratorTarget::ToServer
        );
        assert!(GeneratorTarget::from_str("invalid").is_err());

        let mut opts = LanguageGenerationOptions::default();
        assert_eq!(opts.effective_target(), GeneratorTarget::ToSdk);
        opts.cli = true;
        assert_eq!(opts.effective_target(), GeneratorTarget::ToSdkCli);
        opts.target = GeneratorTarget::ToServer;
        assert_eq!(opts.effective_target(), GeneratorTarget::ToServer);

        // Validate valid frameworks and ORMs across all 13 languages
        let valid_pairs = [
            (TargetLanguage::C, "libcurl", "libpq"),
            (TargetLanguage::Cpp, "drogon", "odb"),
            (TargetLanguage::Csharp, "aspnetcore", "efcore"),
            (TargetLanguage::Go, "gin", "gorm"),
            (TargetLanguage::Java, "spring", "hibernate"),
            (TargetLanguage::Kotlin, "ktor", "exposed"),
            (TargetLanguage::Php, "laravel", "eloquent"),
            (TargetLanguage::Python, "fastapi", "sqlalchemy"),
            (TargetLanguage::Ruby, "rails", "activerecord"),
            (TargetLanguage::Rust, "actix", "diesel"),
            (TargetLanguage::Sh, "curl", "psql"),
            (TargetLanguage::Swift, "vapor", "fluent"),
            (TargetLanguage::Ts, "angular", "prisma"),
        ];

        for (lang, fw, orm) in valid_pairs {
            let lang_opts = LanguageGenerationOptions {
                target: GeneratorTarget::ToServer,
                cli: true,
                mcp: true,
                framework: Some(fw.to_string()),
                orm: Some(orm.to_string()),
                auto_admin: true,
                tests: true,
                no_github_actions: false,
                no_installable_package: false,
            };
            lang_opts.validate_for(lang)?;
        }

        // Empty or incompatible framework/ORM rejected
        let empty_fw = LanguageGenerationOptions {
            framework: Some("   ".into()),
            ..LanguageGenerationOptions::default()
        };
        assert!(empty_fw.validate_for(TargetLanguage::Rust).is_err());

        let bad_fw = LanguageGenerationOptions {
            framework: Some("django".into()),
            ..LanguageGenerationOptions::default()
        };
        assert!(bad_fw.validate_for(TargetLanguage::Rust).is_err());

        let empty_orm = LanguageGenerationOptions {
            orm: Some("   ".into()),
            ..LanguageGenerationOptions::default()
        };
        assert!(empty_orm.validate_for(TargetLanguage::Rust).is_err());

        let bad_orm = LanguageGenerationOptions {
            orm: Some("hibernate".into()),
            ..LanguageGenerationOptions::default()
        };
        assert!(bad_orm.validate_for(TargetLanguage::Rust).is_err());

        Ok(())
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
    fn test_token_provider_registry_and_docs_config() -> Result<(), Error> {
        let providers = [
            ("github", TokenProvider::GitHub),
            ("gitlab", TokenProvider::GitLab),
            ("bitbucket", TokenProvider::Bitbucket),
            ("npm", TokenProvider::Npm),
            ("pypi", TokenProvider::PyPi),
            ("cargo", TokenProvider::Cargo),
            ("maven", TokenProvider::Maven),
            ("nuget", TokenProvider::NuGet),
            ("rubygems", TokenProvider::RubyGems),
            ("packagist", TokenProvider::Packagist),
            ("conan", TokenProvider::Conan),
            ("vcpkg", TokenProvider::Vcpkg),
            ("homebrew", TokenProvider::Homebrew),
            ("readthedocs", TokenProvider::ReadTheDocs),
            ("pgp_private_key", TokenProvider::PgpPrivateKey),
            ("pgp_passphrase", TokenProvider::PgpPassphrase),
        ];
        for (name, variant) in providers {
            assert_eq!(TokenProvider::from_str(name)?, variant);
            assert_eq!(variant.as_str(), name);
        }
        assert_eq!(TokenProvider::from_str("crates")?, TokenProvider::Cargo);
        assert!(TokenProvider::from_str("unknown_provider").is_err());

        let registries = [
            ("crates_io", Registry::CratesIo, TokenProvider::Cargo),
            ("npm", Registry::Npm, TokenProvider::Npm),
            ("pypi", Registry::PyPi, TokenProvider::PyPi),
            (
                "maven_central",
                Registry::MavenCentral,
                TokenProvider::Maven,
            ),
            ("nuget", Registry::NuGet, TokenProvider::NuGet),
            ("rubygems", Registry::RubyGems, TokenProvider::RubyGems),
            ("packagist", Registry::Packagist, TokenProvider::Packagist),
            ("pkg_go_dev", Registry::PkgGoDev, TokenProvider::GitHub),
            (
                "swift_package_index",
                Registry::SwiftPackageIndex,
                TokenProvider::GitHub,
            ),
            ("vcpkg", Registry::Vcpkg, TokenProvider::Vcpkg),
            ("conan", Registry::Conan, TokenProvider::Conan),
            ("homebrew", Registry::Homebrew, TokenProvider::Homebrew),
            (
                "github_releases",
                Registry::GithubReleases,
                TokenProvider::GitHub,
            ),
        ];
        for (name, reg, prov) in registries {
            assert_eq!(Registry::from_str(name)?, reg);
            assert_eq!(reg.as_str(), name);
            assert_eq!(reg.required_token_provider(), prov);
        }
        assert!(Registry::from_str("unknown_reg").is_err());

        for &lang in TargetLanguage::all() {
            assert_ne!(Registry::default_for_language(lang), []);
        }

        let doc_targets = [
            ("official", DocHostTarget::OfficialCddSite),
            ("github_pages", DocHostTarget::GithubPages),
            ("readthedocs", DocHostTarget::ReadTheDocs),
            ("docs_rs", DocHostTarget::DocsRs),
            ("pkg_go_dev", DocHostTarget::PkgGoDev),
            ("javadoc_io", DocHostTarget::JavadocIo),
            ("rubydoc", DocHostTarget::RubyDoc),
            ("spi_docc", DocHostTarget::SwiftPackageIndexDocs),
            ("custom", DocHostTarget::CustomUrl),
            ("none", DocHostTarget::None),
        ];
        assert_eq!(DocHostTarget::default(), DocHostTarget::OfficialCddSite);
        for (name, target) in doc_targets {
            assert_eq!(DocHostTarget::from_str(name)?, target);
            assert_eq!(target.as_str(), name);
        }
        assert!(DocHostTarget::from_str("unknown_doc_host").is_err());

        let mut docs_cfg = DocsPublishConfig::default();
        docs_cfg.validate()?;
        assert_eq!(docs_cfg.resolved_github_pages_branch(), "gh-pages");
        docs_cfg.github_pages_branch = Some("docs-branch".into());
        assert_eq!(docs_cfg.resolved_github_pages_branch(), "docs-branch");

        docs_cfg.target = DocHostTarget::CustomUrl;
        assert!(docs_cfg.validate().is_err());
        docs_cfg.custom_url = Some("ftp://invalid".into());
        assert!(docs_cfg.validate().is_err());
        docs_cfg.custom_url = Some("https://docs.example.com".into());
        docs_cfg.validate()?;
        docs_cfg.custom_url = Some("http://docs.example.com".into());
        docs_cfg.validate()?;

        Ok(())
    }

    #[test]
    fn test_audit_action_parsing() -> Result<(), Error> {
        let actions = [
            ("user_registered", AuditAction::UserRegistered),
            ("account_connected", AuditAction::AccountConnected),
            ("account_disconnected", AuditAction::AccountDisconnected),
            ("org_created", AuditAction::OrgCreated),
            ("member_added", AuditAction::MemberAdded),
            ("repo_created", AuditAction::RepoCreated),
            ("repo_config_updated", AuditAction::RepoConfigUpdated),
            ("project_provisioned", AuditAction::ProjectProvisioned),
            ("release_created", AuditAction::ReleaseCreated),
            ("docs_published", AuditAction::DocsPublished),
            ("token_stored", AuditAction::TokenStored),
            ("token_deleted", AuditAction::TokenDeleted),
        ];
        for (name, action) in actions {
            assert_eq!(AuditAction::from_str(name)?, action);
            assert_eq!(action.as_str(), name);
        }
        assert!(AuditAction::from_str("invalid_action").is_err());
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
