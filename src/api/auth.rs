//! Authentication endpoints for user registration, Argon2 password verification,
//! GitHub & Google OAuth login, account linking, and JWT issuance.

use crate::api::auth_middleware::{AuthenticatedUser, Claims};
use crate::config::AppConfig;
use crate::crypto::encrypt_local_secret;
use crate::db::repository::CddRepository;
use crate::domain::{AuthMethod, Email, GoogleId, OAuthCode, Password, TokenProvider, Username};
use crate::error::Error;
use actix_web::{web, HttpResponse};
use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use async_trait::async_trait;
use jsonwebtoken::{encode, EncodingKey, Header};
#[cfg(test)]
use mockall::automock;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use utoipa::ToSchema;

/// Pre-computed dummy Argon2id PHC hash used to equalize timing when a username does not exist.
const DUMMY_PASSWORD_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHR2YWx1ZQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG";

/// Response payload containing a signed Bearer JWT.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    /// Signed JWT token string.
    pub token: String,
}

/// Request payload for authenticating with username and password.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct LoginPayload {
    /// Account username.
    pub username: String,
    /// Plaintext password.
    pub password: Option<String>,
}

/// Request payload for registering a new user account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RegisterPayload {
    /// Desired unique username (`1..=64` alphanumeric, `-`, `_`).
    pub username: String,
    /// Validated email address.
    pub email: String,
    /// Optional plaintext password (if provided, must be non-empty).
    pub password: Option<String>,
}

/// Normalized user profile returned from an external OAuth code exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct OAuthUserProfile {
    /// Provider-specific unique identifier (`github_id` as string or Google `sub`).
    pub provider_id: String,
    /// Suggested login handle / username.
    pub username: String,
    /// Verified email address from the OAuth provider.
    pub email: String,
    /// Optional avatar URL.
    pub avatar_url: Option<String>,
    /// Optional OAuth access token to encrypt and persist in `user_tokens`.
    pub access_token: Option<String>,
}

/// Injectable OAuth authorization code exchange client.
#[allow(missing_docs, clippy::missing_docs_in_private_items)]
#[cfg_attr(test, automock)]
#[async_trait]
pub trait OAuthExchangeClient: Send + Sync {
    /// Exchanges a GitHub OAuth authorization `code` for a verified [`OAuthUserProfile`].
    ///
    /// # Arguments
    ///
    /// * `&self` - The exchange client instance.
    /// * `code` - Validated [`OAuthCode`].
    ///
    /// # Returns
    ///
    /// Verified [`OAuthUserProfile`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the upstream OAuth exchange fails or rejects the code.
    async fn exchange_github_code(&self, code: &str) -> Result<OAuthUserProfile, Error>;

    /// Exchanges a Google OAuth authorization `code` for a verified [`OAuthUserProfile`].
    ///
    /// # Arguments
    ///
    /// * `&self` - The exchange client instance.
    /// * `code` - Validated [`OAuthCode`].
    ///
    /// # Returns
    ///
    /// Verified [`OAuthUserProfile`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the upstream OAuth exchange fails or rejects the code.
    async fn exchange_google_code(&self, code: &str) -> Result<OAuthUserProfile, Error>;
}

/// Request payload for `POST /auth/github` (supports either OAuth `code` exchange or verified direct identity).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GitHubAuthPayload {
    /// Optional OAuth authorization code to exchange via [`OAuthExchangeClient`].
    pub code: Option<String>,
    /// Optional direct GitHub numeric user ID.
    pub github_id: Option<i64>,
    /// Optional username when registering/updating directly.
    pub username: Option<String>,
    /// Optional verified email address when registering/auto-linking directly.
    pub email: Option<String>,
    /// Optional GitHub OAuth access token to encrypt and store in `user_tokens`.
    pub access_token: Option<String>,
}

/// Request payload for `POST /auth/google` (supports either OAuth `code` exchange or verified direct identity).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GoogleAuthPayload {
    /// Optional OAuth authorization code to exchange via [`OAuthExchangeClient`].
    pub code: Option<String>,
    /// Optional direct Google subject identifier (`sub`).
    pub google_id: Option<String>,
    /// Optional username when registering directly.
    pub username: Option<String>,
    /// Optional verified email address when registering/auto-linking directly.
    pub email: Option<String>,
    /// Optional avatar URL.
    pub avatar_url: Option<String>,
}

/// Request payload for `POST /auth/me/connect/github`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ConnectGitHubPayload {
    /// Optional OAuth authorization code to exchange via [`OAuthExchangeClient`].
    pub code: Option<String>,
    /// Optional direct GitHub numeric user ID.
    pub github_id: Option<i64>,
    /// Optional GitHub OAuth access token to encrypt and store in `user_tokens`.
    pub access_token: Option<String>,
}

/// Request payload for `POST /auth/me/connect/google`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ConnectGooglePayload {
    /// Optional OAuth authorization code to exchange via [`OAuthExchangeClient`].
    pub code: Option<String>,
    /// Optional direct Google subject identifier (`sub`).
    pub google_id: Option<String>,
}

/// Request payload for `POST /auth/me/connect/password`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ConnectPasswordPayload {
    /// Plaintext password to hash with Argon2id and attach to the authenticated account.
    pub password: String,
}

/// Response payload for `GET /auth/me` describing the caller's profile and linked credentials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UserProfileResponse {
    /// Internal user primary key ID.
    pub id: i32,
    /// Unique account username.
    pub username: String,
    /// Account email address.
    pub email: String,
    /// Optional linked GitHub numeric user ID.
    pub github_id: Option<i64>,
    /// Optional linked Google subject identifier (`sub`).
    pub google_id: Option<String>,
    /// Optional user avatar URL.
    pub avatar_url: Option<String>,
    /// Whether a GitHub identity is linked to this account.
    pub github_linked: bool,
    /// Whether a Google identity is linked to this account.
    pub google_linked: bool,
    /// Whether a password credential is set on this account.
    pub has_password: bool,
    /// List of active authentication methods (`password`, `github`, `google`).
    pub connected_providers: Vec<AuthMethod>,
}

impl From<crate::db::models::User> for UserProfileResponse {
    /// Converts a persistent [`crate::db::models::User`] into a [`UserProfileResponse`].
    ///
    /// # Arguments
    ///
    /// * `user` - The database [`crate::db::models::User`] record.
    ///
    /// # Returns
    ///
    /// Populated [`UserProfileResponse`].
    fn from(user: crate::db::models::User) -> Self {
        let connected_providers = user.connected_providers();
        let github_linked = user.github_id.is_some();
        let google_linked = user.google_id.is_some();
        let has_password = user.password_hash.is_some();
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            github_id: user.github_id,
            google_id: user.google_id,
            avatar_url: user.avatar_url,
            github_linked,
            google_linked,
            has_password,
            connected_providers,
        }
    }
}

/// Generates a signed 24-hour JWT for the given user ID and username.
///
/// # Arguments
///
/// * `user_id` - Database primary key of the user.
/// * `username` - Canonical username of the user.
/// * `jwt_secret` - HMAC secret bytes from [`AppConfig`].
///
/// # Returns
///
/// Signed JWT token string.
///
/// # Errors
///
/// Returns [`Error::Jwt`] or [`Error::InternalError`] if token encoding fails.
fn generate_token(user_id: i32, username: &str, jwt_secret: &[u8]) -> Result<String, Error> {
    let ts = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp();
    let exp = usize::try_from(ts).unwrap_or(usize::MAX);
    let claims = Claims {
        sub: user_id,
        exp,
        username: username.to_string(),
    };
    let key = EncodingKey::from_secret(jwt_secret);
    Ok(encode(&Header::default(), &claims, &key)?)
}

/// Hashes a plaintext password using Argon2id on a randomly generated salt.
///
/// # Arguments
///
/// * `password` - The plaintext password to hash.
///
/// # Returns
///
/// Argon2id PHC string.
///
/// # Errors
///
/// Returns [`Error::PasswordHash`] if hashing fails.
fn hash_password(password: &str) -> Result<String, Error> {
    let hash: PasswordHash = Argon2::default().hash_password(password.as_bytes())?;
    Ok(hash.to_string())
}

/// Verifies a plaintext password against an Argon2 PHC hash string.
///
/// # Arguments
///
/// * `password` - The candidate plaintext password.
/// * `hash` - The stored Argon2 PHC string.
///
/// # Returns
///
/// `true` if `password` matches `hash`, otherwise `false`.
fn verify_password(password: &str, hash: &str) -> bool {
    if let Ok(parsed_hash) = PasswordHash::new(hash) {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    } else {
        false
    }
}

/// Registers a new user account and returns a signed JWT.
///
/// # Arguments
///
/// * `payload` - JSON [`RegisterPayload`] containing username, email, and optional password.
/// * `repo` - Database repository handle.
/// * `cfg` - Application configuration containing the JWT signing secret.
///
/// # Returns
///
/// `201 Created` with [`AuthResponse`].
///
/// # Errors
///
/// Returns [`Error::BadRequest`] on invalid inputs or [`Error::Conflict`] if username/email already exists.
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterPayload,
    responses(
        (status = 201, description = "Successfully registered", body = AuthResponse),
        (status = 400, description = "Bad Request"),
        (status = 409, description = "Conflict")
    )
)]
pub async fn register(
    payload: web::Json<RegisterPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
    cfg: web::Data<AppConfig>,
) -> Result<HttpResponse, Error> {
    let username = Username::parse(&payload.username)?;
    let email = Email::parse(&payload.email)?;

    let hashed_pw = if let Some(pw) = payload.password.as_ref() {
        let validated_pw = Password::parse(pw)?;
        let pw_owned = validated_pw.expose_secret().to_string();
        Some(web::block(move || hash_password(&pw_owned)).await??)
    } else {
        None
    };

    let user = repo
        .create_user(
            None,
            username.as_str().to_string(),
            email.as_str().to_string(),
            hashed_pw,
        )
        .await?;

    let token = generate_token(user.id, &user.username, cfg.jwt_secret.as_bytes())?;

    Ok(HttpResponse::Created().json(AuthResponse { token }))
}

/// Authenticates a user with username and password and returns a signed JWT.
///
/// Performs constant-path Argon2 verification even when the user does not exist to
/// mitigate timing-based username enumeration.
///
/// # Arguments
///
/// * `payload` - JSON [`LoginPayload`] containing username and password.
/// * `repo` - Database repository handle.
/// * `cfg` - Application configuration containing the JWT signing secret.
///
/// # Returns
///
/// `200 OK` with [`AuthResponse`].
///
/// # Errors
///
/// Returns [`Error::Unauthorized`] on invalid credentials.
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginPayload,
    responses(
        (status = 200, description = "Successfully authenticated", body = AuthResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn login_password(
    payload: web::Json<LoginPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
    cfg: web::Data<AppConfig>,
) -> Result<HttpResponse, Error> {
    let Some(pw) = payload.password.clone() else {
        return Err(Error::Unauthorized);
    };

    let user_opt = repo.find_user_by_username(payload.username.clone()).await?;

    if let Some(user) = user_opt {
        let Some(stored_hash) = user.password_hash.clone() else {
            let _ = web::block(move || verify_password(&pw, DUMMY_PASSWORD_HASH)).await?;
            return Err(Error::Unauthorized);
        };
        let is_valid = web::block(move || verify_password(&pw, &stored_hash)).await?;
        if is_valid {
            let token = generate_token(user.id, &user.username, cfg.jwt_secret.as_bytes())?;
            Ok(HttpResponse::Ok().json(AuthResponse { token }))
        } else {
            Err(Error::Unauthorized)
        }
    } else {
        let _ = web::block(move || verify_password(&pw, DUMMY_PASSWORD_HASH)).await?;
        Err(Error::Unauthorized)
    }
}

/// Authenticates or registers a user via GitHub OAuth, automatically linking to an existing
/// account if a verified email matches.
///
/// # Arguments
///
/// * `payload` - JSON [`GitHubAuthPayload`].
/// * `repo` - Database repository handle.
/// * `cfg` - Application configuration.
/// * `oauth` - Optional injectable [`OAuthExchangeClient`].
///
/// # Returns
///
/// `200 OK` with [`AuthResponse`].
///
/// # Errors
///
/// Returns [`Error::BadRequest`] on invalid input or missing OAuth exchange client.
#[utoipa::path(
    post,
    path = "/auth/github",
    request_body = GitHubAuthPayload,
    responses(
        (status = 200, description = "Authenticated via GitHub", body = AuthResponse),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn github_login(
    payload: web::Json<GitHubAuthPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
    cfg: web::Data<AppConfig>,
    oauth: Option<web::Data<Arc<dyn OAuthExchangeClient>>>,
) -> Result<HttpResponse, Error> {
    let (gh_id, username_str, email_str, access_token) =
        if let Some(raw_code) = payload.code.as_deref() {
            let code = OAuthCode::parse(raw_code)?;
            let Some(client) = oauth else {
                return Err(Error::BadRequest(
                    "GitHub OAuth code exchange client is not configured".into(),
                ));
            };
            let profile = client.exchange_github_code(code.as_str()).await?;
            let Ok(parsed_id) = profile.provider_id.parse::<i64>() else {
                return Err(Error::BadRequest("Invalid numeric GitHub user ID".into()));
            };
            let resolved_token = profile.access_token.or(payload.access_token.clone());
            (parsed_id, profile.username, profile.email, resolved_token)
        } else {
            let Some(id) = payload.github_id else {
                return Err(Error::BadRequest(
                    "Either 'code' or 'github_id' must be provided".into(),
                ));
            };
            let Some(uname) = payload.username.clone() else {
                return Err(Error::BadRequest(
                    "'username' is required when authenticating with 'github_id'".into(),
                ));
            };
            let Some(em) = payload.email.clone() else {
                return Err(Error::BadRequest(
                    "'email' is required when authenticating with 'github_id'".into(),
                ));
            };
            (id, uname, em, payload.access_token.clone())
        };

    let username = Username::parse(&username_str)?;
    let email = Email::parse(&email_str)?;

    let user = if let Some(existing) = repo.find_user_by_github_id(gh_id).await? {
        existing
    } else if let Some(by_email) = repo.find_user_by_email(email.as_str().to_string()).await? {
        repo.link_github_to_user(by_email.id, gh_id).await?
    } else {
        repo.upsert_user(
            gh_id,
            username.as_str().to_string(),
            email.as_str().to_string(),
        )
        .await?
    };

    if let Some(tok) = access_token.as_deref() {
        if !tok.trim().is_empty() {
            let encrypted = encrypt_local_secret(&cfg.encryption_key, tok)?;
            repo.upsert_user_token(user.id, TokenProvider::GitHub, encrypted)
                .await?;
        }
    }

    let token = generate_token(user.id, &user.username, cfg.jwt_secret.as_bytes())?;
    Ok(HttpResponse::Ok().json(AuthResponse { token }))
}

/// Authenticates or registers a user via Google OAuth, automatically linking to an existing
/// account if a verified email matches.
///
/// # Arguments
///
/// * `payload` - JSON [`GoogleAuthPayload`].
/// * `repo` - Database repository handle.
/// * `cfg` - Application configuration.
/// * `oauth` - Optional injectable [`OAuthExchangeClient`].
///
/// # Returns
///
/// `200 OK` with [`AuthResponse`].
///
/// # Errors
///
/// Returns [`Error::BadRequest`] on invalid input or missing OAuth exchange client.
#[utoipa::path(
    post,
    path = "/auth/google",
    request_body = GoogleAuthPayload,
    responses(
        (status = 200, description = "Authenticated via Google", body = AuthResponse),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn google_login(
    payload: web::Json<GoogleAuthPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
    cfg: web::Data<AppConfig>,
    oauth: Option<web::Data<Arc<dyn OAuthExchangeClient>>>,
) -> Result<HttpResponse, Error> {
    let (google_id, username_str, email_str, avatar_url) =
        if let Some(raw_code) = payload.code.as_deref() {
            let code = OAuthCode::parse(raw_code)?;
            let Some(client) = oauth else {
                return Err(Error::BadRequest(
                    "Google OAuth code exchange client is not configured".into(),
                ));
            };
            let profile = client.exchange_google_code(code.as_str()).await?;
            let gid = GoogleId::parse(&profile.provider_id)?;
            let resolved_avatar = profile.avatar_url.or(payload.avatar_url.clone());
            (gid, profile.username, profile.email, resolved_avatar)
        } else {
            let Some(raw_gid) = payload.google_id.as_deref() else {
                return Err(Error::BadRequest(
                    "Either 'code' or 'google_id' must be provided".into(),
                ));
            };
            let gid = GoogleId::parse(raw_gid)?;
            let Some(uname) = payload.username.clone() else {
                return Err(Error::BadRequest(
                    "'username' is required when authenticating with 'google_id'".into(),
                ));
            };
            let Some(em) = payload.email.clone() else {
                return Err(Error::BadRequest(
                    "'email' is required when authenticating with 'google_id'".into(),
                ));
            };
            (gid, uname, em, payload.avatar_url.clone())
        };

    let username = Username::parse(&username_str)?;
    let email = Email::parse(&email_str)?;

    let user = if let Some(existing) = repo
        .find_user_by_google_id(google_id.as_str().to_string())
        .await?
    {
        existing
    } else if let Some(by_email) = repo.find_user_by_email(email.as_str().to_string()).await? {
        repo.link_google_to_user(by_email.id, google_id.as_str().to_string())
            .await?
    } else {
        repo.create_google_user(
            google_id.as_str().to_string(),
            username.as_str().to_string(),
            email.as_str().to_string(),
            avatar_url,
        )
        .await?
    };

    let token = generate_token(user.id, &user.username, cfg.jwt_secret.as_bytes())?;
    Ok(HttpResponse::Ok().json(AuthResponse { token }))
}

/// Returns the authenticated user's profile and connected authentication methods.
///
/// # Arguments
///
/// * `auth_user` - Authenticated caller extracted from the Bearer JWT.
/// * `repo` - Database repository handle.
///
/// # Returns
///
/// `200 OK` with [`UserProfileResponse`].
///
/// # Errors
///
/// Returns [`Error::NotFound`] if the user record no longer exists.
#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "Current user profile", body = UserProfileResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not Found")
    )
)]
pub async fn get_me(
    auth_user: AuthenticatedUser,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let Some(user) = repo.find_user_by_id(auth_user.user_id).await? else {
        return Err(Error::NotFound("User not found".into()));
    };
    Ok(HttpResponse::Ok().json(UserProfileResponse::from(user)))
}

/// Connects a GitHub identity to the currently authenticated user's account.
///
/// # Arguments
///
/// * `auth_user` - Authenticated caller identity.
/// * `payload` - JSON [`ConnectGitHubPayload`].
/// * `repo` - Database repository handle.
/// * `cfg` - Application configuration.
/// * `oauth` - Optional injectable [`OAuthExchangeClient`].
///
/// # Returns
///
/// `200 OK` with updated [`UserProfileResponse`].
///
/// # Errors
///
/// Returns [`Error::BadRequest`] on invalid input or [`Error::Conflict`] if already linked to another user.
#[utoipa::path(
    post,
    path = "/auth/me/connect/github",
    request_body = ConnectGitHubPayload,
    responses(
        (status = 200, description = "GitHub account linked", body = UserProfileResponse),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Conflict")
    )
)]
pub async fn connect_github(
    auth_user: AuthenticatedUser,
    payload: web::Json<ConnectGitHubPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
    cfg: web::Data<AppConfig>,
    oauth: Option<web::Data<Arc<dyn OAuthExchangeClient>>>,
) -> Result<HttpResponse, Error> {
    let (gh_id, access_token) = if let Some(raw_code) = payload.code.as_deref() {
        let code = OAuthCode::parse(raw_code)?;
        let Some(client) = oauth else {
            return Err(Error::BadRequest(
                "GitHub OAuth code exchange client is not configured".into(),
            ));
        };
        let profile = client.exchange_github_code(code.as_str()).await?;
        let Ok(parsed_id) = profile.provider_id.parse::<i64>() else {
            return Err(Error::BadRequest("Invalid numeric GitHub user ID".into()));
        };
        let resolved_token = profile.access_token.or(payload.access_token.clone());
        (parsed_id, resolved_token)
    } else {
        let Some(id) = payload.github_id else {
            return Err(Error::BadRequest(
                "Either 'code' or 'github_id' must be provided".into(),
            ));
        };
        (id, payload.access_token.clone())
    };

    if repo
        .find_user_by_github_id(gh_id)
        .await?
        .is_some_and(|existing| existing.id != auth_user.user_id)
    {
        return Err(Error::Conflict(
            "This GitHub account is already linked to another user".into(),
        ));
    }

    let updated = repo.link_github_to_user(auth_user.user_id, gh_id).await?;

    if let Some(tok) = access_token.as_deref().filter(|t| !t.trim().is_empty()) {
        let encrypted = encrypt_local_secret(&cfg.encryption_key, tok)?;
        repo.upsert_user_token(auth_user.user_id, TokenProvider::GitHub, encrypted)
            .await?;
    }

    Ok(HttpResponse::Ok().json(UserProfileResponse::from(updated)))
}

/// Connects a Google identity to the currently authenticated user's account.
///
/// # Arguments
///
/// * `auth_user` - Authenticated caller identity.
/// * `payload` - JSON [`ConnectGooglePayload`].
/// * `repo` - Database repository handle.
/// * `oauth` - Optional injectable [`OAuthExchangeClient`].
///
/// # Returns
///
/// `200 OK` with updated [`UserProfileResponse`].
///
/// # Errors
///
/// Returns [`Error::BadRequest`] on invalid input or [`Error::Conflict`] if already linked to another user.
#[utoipa::path(
    post,
    path = "/auth/me/connect/google",
    request_body = ConnectGooglePayload,
    responses(
        (status = 200, description = "Google account linked", body = UserProfileResponse),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Conflict")
    )
)]
pub async fn connect_google(
    auth_user: AuthenticatedUser,
    payload: web::Json<ConnectGooglePayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
    oauth: Option<web::Data<Arc<dyn OAuthExchangeClient>>>,
) -> Result<HttpResponse, Error> {
    let google_id = if let Some(raw_code) = payload.code.as_deref() {
        let code = OAuthCode::parse(raw_code)?;
        let Some(client) = oauth else {
            return Err(Error::BadRequest(
                "Google OAuth code exchange client is not configured".into(),
            ));
        };
        let profile = client.exchange_google_code(code.as_str()).await?;
        GoogleId::parse(&profile.provider_id)?
    } else {
        let Some(raw_gid) = payload.google_id.as_deref() else {
            return Err(Error::BadRequest(
                "Either 'code' or 'google_id' must be provided".into(),
            ));
        };
        GoogleId::parse(raw_gid)?
    };

    if repo
        .find_user_by_google_id(google_id.as_str().to_string())
        .await?
        .is_some_and(|existing| existing.id != auth_user.user_id)
    {
        return Err(Error::Conflict(
            "This Google account is already linked to another user".into(),
        ));
    }

    let updated = repo
        .link_google_to_user(auth_user.user_id, google_id.as_str().to_string())
        .await?;
    Ok(HttpResponse::Ok().json(UserProfileResponse::from(updated)))
}

/// Sets or updates an Argon2id password credential on the currently authenticated user's account.
///
/// # Arguments
///
/// * `auth_user` - Authenticated caller identity.
/// * `payload` - JSON [`ConnectPasswordPayload`].
/// * `repo` - Database repository handle.
///
/// # Returns
///
/// `200 OK` with updated [`UserProfileResponse`].
///
/// # Errors
///
/// Returns [`Error::BadRequest`] if the password is empty.
#[utoipa::path(
    post,
    path = "/auth/me/connect/password",
    request_body = ConnectPasswordPayload,
    responses(
        (status = 200, description = "Password credential linked", body = UserProfileResponse),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn connect_password(
    auth_user: AuthenticatedUser,
    payload: web::Json<ConnectPasswordPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let validated_pw = Password::parse(&payload.password)?;
    let pw_owned = validated_pw.expose_secret().to_string();
    let hashed = web::block(move || hash_password(&pw_owned)).await??;
    let updated = repo.set_user_password(auth_user.user_id, hashed).await?;
    Ok(HttpResponse::Ok().json(UserProfileResponse::from(updated)))
}

/// Disconnects an authentication method (`github`, `google`, or `password`) from the caller's account
/// provided at least one other authentication method remains active.
///
/// # Arguments
///
/// * `auth_user` - Authenticated caller identity.
/// * `path` - Provider name (`github`, `google`, or `password`).
/// * `repo` - Database repository handle.
///
/// # Returns
///
/// `200 OK` with updated [`UserProfileResponse`].
///
/// # Errors
///
/// Returns [`Error::BadRequest`] if the provider is invalid or is the user's last remaining credential.
#[utoipa::path(
    delete,
    path = "/auth/me/connect/{provider}",
    params(
        ("provider" = String, Path, description = "Authentication method to disconnect (github, google, password)")
    ),
    responses(
        (status = 200, description = "Authentication method disconnected", body = UserProfileResponse),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn disconnect_provider(
    auth_user: AuthenticatedUser,
    path: web::Path<String>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let method = AuthMethod::from_str(&path.into_inner())?;
    let updated = repo
        .unlink_provider_from_user(auth_user.user_id, method)
        .await?;
    Ok(HttpResponse::Ok().json(UserProfileResponse::from(updated)))
}

/// Configures the `/auth` routes on the provided [`web::ServiceConfig`].
///
/// # Arguments
///
/// * `cfg` - Actix service configuration builder.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login_password))
            .route("/github", web::post().to(github_login))
            .route("/google", web::post().to(google_login))
            .route("/me", web::get().to(get_me))
            .route("/me/connect/github", web::post().to(connect_github))
            .route("/me/connect/google", web::post().to(connect_google))
            .route("/me/connect/password", web::post().to(connect_password))
            .route(
                "/me/connect/{provider}",
                web::delete().to(disconnect_provider),
            ),
    );
}

#[cfg(test)]
mod tests {
    //! Unit tests for registration, password hashing, login, GitHub/Google OAuth, and account linking.

    use super::*;
    use crate::api::auth_middleware::{generate_test_token, test_app_config};
    use crate::db::models::{User, UserToken};
    use crate::db::repository::MockCddRepository;
    use actix_web::{body::to_bytes, http::StatusCode, test, App};
    use jsonwebtoken::{decode, DecodingKey, Validation};

    fn sample_user(
        id: i32,
        username: &str,
        email: &str,
        github_id: Option<i64>,
        google_id: Option<&str>,
        password_hash: Option<&str>,
    ) -> User {
        User {
            id,
            github_id,
            username: username.to_string(),
            email: email.to_string(),
            password_hash: password_hash.map(str::to_string),
            google_id: google_id.map(str::to_string),
            avatar_url: None,
        }
    }

    #[actix_web::test]
    async fn test_verify_password_invalid_hash() {
        assert!(!verify_password("pwd", "invalid_hash_string"));
    }

    #[actix_web::test]
    async fn test_register_with_password_verifies_jwt_and_args() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_create_user()
            .withf(|gh_id, username, email, pw_hash| {
                gh_id.is_none()
                    && username == "test_user"
                    && email == "test@example.com"
                    && pw_hash
                        .as_deref()
                        .is_some_and(|h| h != "StrongPass123" && h.starts_with("$argon2"))
            })
            .times(1)
            .returning(|_, username, email, password_hash| {
                Ok(User {
                    id: 42,
                    github_id: None,
                    username,
                    email,
                    password_hash,
                    google_id: None,
                    avatar_url: None,
                })
            });

        let cfg = test_app_config()?;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(cfg.clone())
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(RegisterPayload {
                username: "test_user".into(),
                email: "test@example.com".into(),
                password: Some("StrongPass123".into()),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = to_bytes(resp.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let auth_resp: AuthResponse = serde_json::from_slice(&body)?;
        let decoded = decode::<Claims>(
            &auth_resp.token,
            &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
            &Validation::default(),
        )?;
        assert_eq!(decoded.claims.sub, 42);
        assert_eq!(decoded.claims.username, "test_user");
        Ok(())
    }

    #[actix_web::test]
    async fn test_register_no_password_and_validation_errors() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_create_user()
            .withf(|_, username, email, pw_hash| {
                username == "oauth_user" && email == "oauth@example.com" && pw_hash.is_none()
            })
            .times(1)
            .returning(|_, username, email, _| {
                Ok(sample_user(7, &username, &email, None, None, None))
            });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(test_app_config()?)
                .configure(configure),
        )
        .await;

        let req_ok = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(RegisterPayload {
                username: "oauth_user".into(),
                email: "oauth@example.com".into(),
                password: None,
            })
            .to_request();
        let resp_ok = test::call_service(&app, req_ok).await;
        assert_eq!(resp_ok.status(), StatusCode::CREATED);

        let req_empty_pw = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(RegisterPayload {
                username: "oauth_user".into(),
                email: "oauth@example.com".into(),
                password: Some("   ".into()),
            })
            .to_request();
        let resp_empty_pw = test::call_service(&app, req_empty_pw).await;
        assert_eq!(resp_empty_pw.status(), StatusCode::BAD_REQUEST);

        let req_bad_email = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(RegisterPayload {
                username: "oauth_user".into(),
                email: "invalid-email".into(),
                password: None,
            })
            .to_request();
        let resp_bad_email = test::call_service(&app, req_bad_email).await;
        assert_eq!(resp_bad_email.status(), StatusCode::BAD_REQUEST);
        Ok(())
    }

    #[actix_web::test]
    async fn test_register_db_error() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_create_user()
            .times(1)
            .returning(|_, _, _, _| Err(Error::InternalError));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(test_app_config()?)
                .configure(configure),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(RegisterPayload {
                username: "test".into(),
                email: "test@test.com".into(),
                password: Some("pwd12345".into()),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }

    #[actix_web::test]
    async fn test_login_success_and_failure_branches() -> Result<(), Error> {
        let hashed_pw = hash_password("correct_pw")?;
        let mut mock_repo = MockCddRepository::new();

        let hash_clone = hashed_pw.clone();
        mock_repo
            .expect_find_user_by_username()
            .withf(|u| u == "alice")
            .times(2)
            .returning(move |_| {
                Ok(Some(sample_user(
                    11,
                    "alice",
                    "alice@example.com",
                    None,
                    None,
                    Some(&hash_clone),
                )))
            });

        mock_repo
            .expect_find_user_by_username()
            .withf(|u| u == "oauth_only")
            .times(1)
            .returning(|_| {
                Ok(Some(sample_user(
                    12,
                    "oauth_only",
                    "oauth@example.com",
                    Some(99),
                    None,
                    None,
                )))
            });

        mock_repo
            .expect_find_user_by_username()
            .withf(|u| u == "missing_user")
            .times(1)
            .returning(|_| Ok(None));

        mock_repo
            .expect_find_user_by_username()
            .withf(|u| u == "db_fail")
            .times(1)
            .returning(|_| Err(Error::InternalError));

        let cfg = test_app_config()?;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(cfg.clone())
                .configure(configure),
        )
        .await;

        // 1. Valid password -> 200 OK + decodable JWT
        let req_ok = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "alice".into(),
                password: Some("correct_pw".into()),
            })
            .to_request();
        let resp_ok = test::call_service(&app, req_ok).await;
        assert_eq!(resp_ok.status(), StatusCode::OK);
        let body = to_bytes(resp_ok.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let auth_resp: AuthResponse = serde_json::from_slice(&body)?;
        let decoded = decode::<Claims>(
            &auth_resp.token,
            &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
            &Validation::default(),
        )?;
        assert_eq!(decoded.claims.sub, 11);
        assert_eq!(decoded.claims.username, "alice");

        // 2. Wrong password -> 401
        let req_wrong = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "alice".into(),
                password: Some("wrong_pw".into()),
            })
            .to_request();
        let resp_wrong = test::call_service(&app, req_wrong).await;
        assert_eq!(resp_wrong.status(), StatusCode::UNAUTHORIZED);

        // 3. No password in payload -> 401
        let req_no_pw = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "alice".into(),
                password: None,
            })
            .to_request();
        let resp_no_pw = test::call_service(&app, req_no_pw).await;
        assert_eq!(resp_no_pw.status(), StatusCode::UNAUTHORIZED);

        // 4. User without password hash -> 401
        let req_no_hash = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "oauth_only".into(),
                password: Some("any_pw".into()),
            })
            .to_request();
        let resp_no_hash = test::call_service(&app, req_no_hash).await;
        assert_eq!(resp_no_hash.status(), StatusCode::UNAUTHORIZED);

        // 5. Non-existent user -> 401
        let req_missing = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "missing_user".into(),
                password: Some("any_pw".into()),
            })
            .to_request();
        let resp_missing = test::call_service(&app, req_missing).await;
        assert_eq!(resp_missing.status(), StatusCode::UNAUTHORIZED);

        // 6. Database failure -> 500 Internal Server Error
        let req_db_err = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginPayload {
                username: "db_fail".into(),
                password: Some("any_pw".into()),
            })
            .to_request();
        let resp_db_err = test::call_service(&app, req_db_err).await;
        assert_eq!(resp_db_err.status(), StatusCode::INTERNAL_SERVER_ERROR);

        Ok(())
    }

    #[actix_web::test]
    async fn test_github_and_google_oauth_login_and_autolinking() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        let mut mock_oauth = MockOAuthExchangeClient::new();

        // 1. GitHub code exchange -> existing github_id match + stores access token
        mock_oauth
            .expect_exchange_github_code()
            .withf(|c| c == "gh_code_existing")
            .times(1)
            .returning(|_| {
                Ok(OAuthUserProfile {
                    provider_id: "1001".into(),
                    username: "gh_alice".into(),
                    email: "alice@example.com".into(),
                    avatar_url: None,
                    access_token: Some("gho_token_123".into()),
                })
            });
        mock_repo
            .expect_find_user_by_github_id()
            .withf(|id| *id == 1001)
            .times(1)
            .returning(|_| {
                Ok(Some(sample_user(
                    1,
                    "gh_alice",
                    "alice@example.com",
                    Some(1001),
                    None,
                    None,
                )))
            });
        mock_repo
            .expect_upsert_user_token()
            .withf(|uid, prov, _| *uid == 1 && *prov == TokenProvider::GitHub)
            .times(1)
            .returning(|user_id, provider, encrypted_token| {
                Ok(UserToken {
                    id: 1,
                    user_id,
                    provider: provider.as_str().to_string(),
                    encrypted_token,
                    created_at: chrono::Utc::now().naive_utc(),
                    updated_at: chrono::Utc::now().naive_utc(),
                })
            });

        // 1b. GitHub code exchange with non-numeric provider_id -> 400
        mock_oauth
            .expect_exchange_github_code()
            .withf(|c| c == "gh_code_bad_id")
            .times(1)
            .returning(|_| {
                Ok(OAuthUserProfile {
                    provider_id: "not-a-number".into(),
                    username: "gh_alice".into(),
                    email: "alice@example.com".into(),
                    avatar_url: None,
                    access_token: None,
                })
            });

        // 2. GitHub direct payload -> auto-links by verified email
        mock_repo
            .expect_find_user_by_github_id()
            .withf(|id| *id == 1002)
            .times(1)
            .returning(|_| Ok(None));
        mock_repo
            .expect_find_user_by_email()
            .withf(|em| em == "bob@example.com")
            .times(1)
            .returning(|_| {
                Ok(Some(sample_user(
                    2,
                    "bob",
                    "bob@example.com",
                    None,
                    Some("google-bob"),
                    None,
                )))
            });
        mock_repo
            .expect_link_github_to_user()
            .withf(|uid, gh| *uid == 2 && *gh == 1002)
            .times(1)
            .returning(|_, gh| {
                Ok(sample_user(
                    2,
                    "bob",
                    "bob@example.com",
                    Some(gh),
                    Some("google-bob"),
                    None,
                ))
            });

        // 3. GitHub direct payload -> creates new user via upsert_user
        mock_repo
            .expect_find_user_by_github_id()
            .withf(|id| *id == 1003)
            .times(1)
            .returning(|_| Ok(None));
        mock_repo
            .expect_find_user_by_email()
            .withf(|em| em == "newgh@example.com")
            .times(1)
            .returning(|_| Ok(None));
        mock_repo
            .expect_upsert_user()
            .withf(|gh, u, e| *gh == 1003 && u == "newgh" && e == "newgh@example.com")
            .times(1)
            .returning(|gh, u, e| Ok(sample_user(3, &u, &e, Some(gh), None, None)));

        // 4. Google code exchange -> existing google_id match
        mock_oauth
            .expect_exchange_google_code()
            .withf(|c| c == "google_code_existing")
            .times(1)
            .returning(|_| {
                Ok(OAuthUserProfile {
                    provider_id: "google-sub-1".into(),
                    username: "carol".into(),
                    email: "carol@example.com".into(),
                    avatar_url: Some("https://example.com/c.png".into()),
                    access_token: None,
                })
            });
        mock_repo
            .expect_find_user_by_google_id()
            .withf(|gid| gid == "google-sub-1")
            .times(1)
            .returning(|gid| {
                Ok(Some(sample_user(
                    4,
                    "carol",
                    "carol@example.com",
                    None,
                    Some(&gid),
                    None,
                )))
            });

        // 5. Google direct payload -> auto-links by verified email
        mock_repo
            .expect_find_user_by_google_id()
            .withf(|gid| gid == "google-sub-2")
            .times(1)
            .returning(|_| Ok(None));
        mock_repo
            .expect_find_user_by_email()
            .withf(|em| em == "dave@example.com")
            .times(1)
            .returning(|_| {
                Ok(Some(sample_user(
                    5,
                    "dave",
                    "dave@example.com",
                    Some(555),
                    None,
                    None,
                )))
            });
        mock_repo
            .expect_link_google_to_user()
            .withf(|uid, gid| *uid == 5 && gid == "google-sub-2")
            .times(1)
            .returning(|_, gid| {
                Ok(sample_user(
                    5,
                    "dave",
                    "dave@example.com",
                    Some(555),
                    Some(&gid),
                    None,
                ))
            });

        // 6. Google direct payload -> creates new user via create_google_user
        mock_repo
            .expect_find_user_by_google_id()
            .withf(|gid| gid == "google-sub-3")
            .times(1)
            .returning(|_| Ok(None));
        mock_repo
            .expect_find_user_by_email()
            .withf(|em| em == "erin@example.com")
            .times(1)
            .returning(|_| Ok(None));
        mock_repo
            .expect_create_google_user()
            .withf(|gid, u, e, _| gid == "google-sub-3" && u == "erin" && e == "erin@example.com")
            .times(1)
            .returning(|gid, u, e, _| Ok(sample_user(6, &u, &e, None, Some(&gid), None)));

        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(web::Data::new(
                    Arc::new(mock_oauth) as Arc<dyn OAuthExchangeClient>
                ))
                .configure(configure),
        )
        .await;

        let r1 = test::TestRequest::post()
            .uri("/auth/github")
            .set_json(GitHubAuthPayload {
                code: Some("gh_code_existing".into()),
                github_id: None,
                username: None,
                email: None,
                access_token: None,
            })
            .to_request();
        assert_eq!(test::call_service(&app, r1).await.status(), StatusCode::OK);

        let r1_bad = test::TestRequest::post()
            .uri("/auth/github")
            .set_json(GitHubAuthPayload {
                code: Some("gh_code_bad_id".into()),
                github_id: None,
                username: None,
                email: None,
                access_token: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, r1_bad).await.status(),
            StatusCode::BAD_REQUEST
        );

        let r2 = test::TestRequest::post()
            .uri("/auth/github")
            .set_json(GitHubAuthPayload {
                code: None,
                github_id: Some(1002),
                username: Some("bob".into()),
                email: Some("bob@example.com".into()),
                access_token: Some("   ".into()),
            })
            .to_request();
        assert_eq!(test::call_service(&app, r2).await.status(), StatusCode::OK);

        let r3 = test::TestRequest::post()
            .uri("/auth/github")
            .set_json(GitHubAuthPayload {
                code: None,
                github_id: Some(1003),
                username: Some("newgh".into()),
                email: Some("newgh@example.com".into()),
                access_token: None,
            })
            .to_request();
        assert_eq!(test::call_service(&app, r3).await.status(), StatusCode::OK);

        // Missing fields on direct github_login -> 400
        for bad_gh in [
            GitHubAuthPayload {
                code: None,
                github_id: None,
                username: None,
                email: None,
                access_token: None,
            },
            GitHubAuthPayload {
                code: None,
                github_id: Some(1),
                username: None,
                email: None,
                access_token: None,
            },
            GitHubAuthPayload {
                code: None,
                github_id: Some(1),
                username: Some("u".into()),
                email: None,
                access_token: None,
            },
        ] {
            let req = test::TestRequest::post()
                .uri("/auth/github")
                .set_json(bad_gh)
                .to_request();
            assert_eq!(
                test::call_service(&app, req).await.status(),
                StatusCode::BAD_REQUEST
            );
        }

        let g1 = test::TestRequest::post()
            .uri("/auth/google")
            .set_json(GoogleAuthPayload {
                code: Some("google_code_existing".into()),
                google_id: None,
                username: None,
                email: None,
                avatar_url: None,
            })
            .to_request();
        assert_eq!(test::call_service(&app, g1).await.status(), StatusCode::OK);

        let g2 = test::TestRequest::post()
            .uri("/auth/google")
            .set_json(GoogleAuthPayload {
                code: None,
                google_id: Some("google-sub-2".into()),
                username: Some("dave".into()),
                email: Some("dave@example.com".into()),
                avatar_url: None,
            })
            .to_request();
        assert_eq!(test::call_service(&app, g2).await.status(), StatusCode::OK);

        let g3 = test::TestRequest::post()
            .uri("/auth/google")
            .set_json(GoogleAuthPayload {
                code: None,
                google_id: Some("google-sub-3".into()),
                username: Some("erin".into()),
                email: Some("erin@example.com".into()),
                avatar_url: None,
            })
            .to_request();
        assert_eq!(test::call_service(&app, g3).await.status(), StatusCode::OK);

        // Missing fields on direct google_login -> 400
        for bad_google in [
            GoogleAuthPayload {
                code: None,
                google_id: None,
                username: None,
                email: None,
                avatar_url: None,
            },
            GoogleAuthPayload {
                code: None,
                google_id: Some("sub".into()),
                username: None,
                email: None,
                avatar_url: None,
            },
            GoogleAuthPayload {
                code: None,
                google_id: Some("sub".into()),
                username: Some("u".into()),
                email: None,
                avatar_url: None,
            },
        ] {
            let req = test::TestRequest::post()
                .uri("/auth/google")
                .set_json(bad_google)
                .to_request();
            assert_eq!(
                test::call_service(&app, req).await.status(),
                StatusCode::BAD_REQUEST
            );
        }

        Ok(())
    }

    #[actix_web::test]
    async fn test_account_linking_and_me_endpoints() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        let mut mock_oauth = MockOAuthExchangeClient::new();

        // GET /auth/me -> found, then not found
        mock_repo
            .expect_find_user_by_id()
            .withf(|id| *id == 1)
            .times(1)
            .returning(|_| {
                Ok(Some(sample_user(
                    1,
                    "testuser",
                    "test@example.com",
                    Some(100),
                    Some("g-100"),
                    Some("hash"),
                )))
            });
        mock_repo
            .expect_find_user_by_id()
            .withf(|id| *id == 1)
            .times(1)
            .returning(|_| Ok(None));

        // Connect GitHub via code -> success + token stored (same user already owns 2002)
        mock_oauth
            .expect_exchange_github_code()
            .withf(|c| c == "connect_gh_code")
            .times(1)
            .returning(|_| {
                Ok(OAuthUserProfile {
                    provider_id: "2002".into(),
                    username: "testuser".into(),
                    email: "test@example.com".into(),
                    avatar_url: None,
                    access_token: None,
                })
            });
        mock_oauth
            .expect_exchange_github_code()
            .withf(|c| c == "connect_gh_bad_id")
            .times(1)
            .returning(|_| {
                Ok(OAuthUserProfile {
                    provider_id: "non_numeric".into(),
                    username: "testuser".into(),
                    email: "test@example.com".into(),
                    avatar_url: None,
                    access_token: None,
                })
            });
        mock_repo
            .expect_find_user_by_github_id()
            .withf(|id| *id == 2002)
            .times(2)
            .returning(|_| {
                Ok(Some(sample_user(
                    1,
                    "testuser",
                    "test@example.com",
                    Some(2002),
                    None,
                    Some("hash"),
                )))
            });
        mock_repo
            .expect_link_github_to_user()
            .withf(|uid, gh| *uid == 1 && *gh == 2002)
            .times(2)
            .returning(|uid, gh| {
                Ok(sample_user(
                    uid,
                    "testuser",
                    "test@example.com",
                    Some(gh),
                    None,
                    Some("hash"),
                ))
            });
        mock_repo
            .expect_upsert_user_token()
            .withf(|uid, prov, _| *uid == 1 && *prov == TokenProvider::GitHub)
            .times(1)
            .returning(|user_id, provider, encrypted_token| {
                Ok(UserToken {
                    id: 1,
                    user_id,
                    provider: provider.as_str().to_string(),
                    encrypted_token,
                    created_at: chrono::Utc::now().naive_utc(),
                    updated_at: chrono::Utc::now().naive_utc(),
                })
            });

        // Connect GitHub conflict when owned by user 99
        mock_repo
            .expect_find_user_by_github_id()
            .withf(|id| *id == 9999)
            .times(1)
            .returning(|_| {
                Ok(Some(sample_user(
                    99,
                    "other",
                    "other@example.com",
                    Some(9999),
                    None,
                    None,
                )))
            });

        // Connect Google via code & direct + conflict (same user already owns g-sub-new)
        mock_oauth
            .expect_exchange_google_code()
            .withf(|c| c == "connect_google_code")
            .times(1)
            .returning(|_| {
                Ok(OAuthUserProfile {
                    provider_id: "g-sub-new".into(),
                    username: "testuser".into(),
                    email: "test@example.com".into(),
                    avatar_url: None,
                    access_token: None,
                })
            });
        mock_repo
            .expect_find_user_by_google_id()
            .withf(|gid| gid == "g-sub-new")
            .times(1)
            .returning(|gid| {
                Ok(Some(sample_user(
                    1,
                    "testuser",
                    "test@example.com",
                    None,
                    Some(&gid),
                    Some("hash"),
                )))
            });
        mock_repo
            .expect_link_google_to_user()
            .withf(|uid, gid| *uid == 1 && gid == "g-sub-new")
            .times(1)
            .returning(|uid, gid| {
                Ok(sample_user(
                    uid,
                    "testuser",
                    "test@example.com",
                    None,
                    Some(&gid),
                    Some("hash"),
                ))
            });

        mock_repo
            .expect_find_user_by_google_id()
            .withf(|gid| gid == "g-sub-taken")
            .times(1)
            .returning(|gid| {
                Ok(Some(sample_user(
                    99,
                    "other",
                    "other@example.com",
                    None,
                    Some(&gid),
                    None,
                )))
            });

        // Connect password
        mock_repo
            .expect_set_user_password()
            .withf(|uid, h| *uid == 1 && h.starts_with("$argon2"))
            .times(1)
            .returning(|uid, h| {
                Ok(sample_user(
                    uid,
                    "testuser",
                    "test@example.com",
                    Some(2002),
                    None,
                    Some(&h),
                ))
            });

        // Disconnect provider
        mock_repo
            .expect_unlink_provider_from_user()
            .withf(|uid, m| *uid == 1 && *m == AuthMethod::Google)
            .times(1)
            .returning(|uid, _| {
                Ok(sample_user(
                    uid,
                    "testuser",
                    "test@example.com",
                    Some(2002),
                    None,
                    Some("hash"),
                ))
            });

        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(web::Data::new(
                    Arc::new(mock_oauth) as Arc<dyn OAuthExchangeClient>
                ))
                .configure(configure),
        )
        .await;

        let token = generate_test_token()?;

        // GET /auth/me -> 200 OK
        let req_me = test::TestRequest::get()
            .uri("/auth/me")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_me = test::call_service(&app, req_me).await;
        assert_eq!(resp_me.status(), StatusCode::OK);
        let me_bytes = to_bytes(resp_me.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let profile: UserProfileResponse = serde_json::from_slice(&me_bytes)?;
        assert!(profile.github_linked);
        assert!(profile.google_linked);
        assert!(profile.has_password);
        assert_eq!(profile.connected_providers.len(), 3);

        // GET /auth/me -> 404 Not Found
        let req_me_nf = test::TestRequest::get()
            .uri("/auth/me")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_me_nf).await.status(),
            StatusCode::NOT_FOUND
        );

        // POST /auth/me/connect/github via code -> 200 OK
        let req_c_gh = test::TestRequest::post()
            .uri("/auth/me/connect/github")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(ConnectGitHubPayload {
                code: Some("connect_gh_code".into()),
                github_id: None,
                access_token: Some("gho_pat".into()),
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_c_gh).await.status(),
            StatusCode::OK
        );

        let req_c_gh_direct = test::TestRequest::post()
            .uri("/auth/me/connect/github")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(ConnectGitHubPayload {
                code: None,
                github_id: Some(2002),
                access_token: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_c_gh_direct).await.status(),
            StatusCode::OK
        );

        let req_c_gh_bad_id = test::TestRequest::post()
            .uri("/auth/me/connect/github")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(ConnectGitHubPayload {
                code: Some("connect_gh_bad_id".into()),
                github_id: None,
                access_token: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_c_gh_bad_id).await.status(),
            StatusCode::BAD_REQUEST
        );

        // POST /auth/me/connect/github conflict -> 409
        let req_c_gh_conf = test::TestRequest::post()
            .uri("/auth/me/connect/github")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(ConnectGitHubPayload {
                code: None,
                github_id: Some(9999),
                access_token: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_c_gh_conf).await.status(),
            StatusCode::CONFLICT
        );

        // POST /auth/me/connect/github missing both code and github_id -> 400
        let req_c_gh_bad = test::TestRequest::post()
            .uri("/auth/me/connect/github")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(ConnectGitHubPayload {
                code: None,
                github_id: None,
                access_token: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_c_gh_bad).await.status(),
            StatusCode::BAD_REQUEST
        );

        // POST /auth/me/connect/google via code -> 200 OK
        let req_c_g = test::TestRequest::post()
            .uri("/auth/me/connect/google")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(ConnectGooglePayload {
                code: Some("connect_google_code".into()),
                google_id: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_c_g).await.status(),
            StatusCode::OK
        );

        // POST /auth/me/connect/google conflict -> 409
        let req_c_g_conf = test::TestRequest::post()
            .uri("/auth/me/connect/google")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(ConnectGooglePayload {
                code: None,
                google_id: Some("g-sub-taken".into()),
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_c_g_conf).await.status(),
            StatusCode::CONFLICT
        );

        // POST /auth/me/connect/google missing both -> 400
        let req_c_g_bad = test::TestRequest::post()
            .uri("/auth/me/connect/google")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(ConnectGooglePayload {
                code: None,
                google_id: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_c_g_bad).await.status(),
            StatusCode::BAD_REQUEST
        );

        // POST /auth/me/connect/password -> 200 OK
        let req_c_pw = test::TestRequest::post()
            .uri("/auth/me/connect/password")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(ConnectPasswordPayload {
                password: "NewPassword123!".into(),
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_c_pw).await.status(),
            StatusCode::OK
        );

        // DELETE /auth/me/connect/google -> 200 OK
        let req_del = test::TestRequest::delete()
            .uri("/auth/me/connect/google")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_del).await.status(),
            StatusCode::OK
        );

        Ok(())
    }

    #[actix_web::test]
    async fn test_oauth_code_without_configured_exchange_client_returns_bad_request(
    ) -> Result<(), Error> {
        let mock_repo = MockCddRepository::new();
        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let token = generate_test_token()?;

        let req_gh = test::TestRequest::post()
            .uri("/auth/github")
            .set_json(GitHubAuthPayload {
                code: Some("some_code".into()),
                github_id: None,
                username: None,
                email: None,
                access_token: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_gh).await.status(),
            StatusCode::BAD_REQUEST
        );

        let req_google = test::TestRequest::post()
            .uri("/auth/google")
            .set_json(GoogleAuthPayload {
                code: Some("some_code".into()),
                google_id: None,
                username: None,
                email: None,
                avatar_url: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_google).await.status(),
            StatusCode::BAD_REQUEST
        );

        let req_c_gh = test::TestRequest::post()
            .uri("/auth/me/connect/github")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(ConnectGitHubPayload {
                code: Some("some_code".into()),
                github_id: None,
                access_token: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_c_gh).await.status(),
            StatusCode::BAD_REQUEST
        );

        let req_c_google = test::TestRequest::post()
            .uri("/auth/me/connect/google")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(ConnectGooglePayload {
                code: Some("some_code".into()),
                google_id: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_c_google).await.status(),
            StatusCode::BAD_REQUEST
        );

        Ok(())
    }
}
