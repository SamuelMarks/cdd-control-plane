//! Database repository trait and PostgreSQL implementation.

use crate::db::models::{
    AuditLog, NewAuditLog, NewOrganization, NewOrganizationUser, NewRelease, NewRepository,
    NewUser, NewUserToken, Organization, OrganizationUser, Release, Repository,
    UpdateRepositoryConfig, User, UserToken,
};
use crate::db::schema::{
    audit_logs, organization_users, organizations, releases, repositories, user_tokens, users,
};
use crate::domain::{AuditAction, AuthMethod, OrgRole, TokenProvider};
use crate::error::Error;
use actix_web::web;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
#[cfg(test)]
use mockall::automock;
use std::str::FromStr;

/// Database connection pool type alias for PostgreSQL connections.
pub type DbPool = r2d2::Pool<ConnectionManager<diesel::PgConnection>>;

/// Abstract persistence repository trait for control-plane CRUD operations.
#[allow(missing_docs, clippy::missing_docs_in_private_items)]
#[cfg_attr(test, automock)]
#[async_trait]
pub trait CddRepository: Send + Sync {
    /// Finds a user by their unique username.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `username` - The username to query.
    ///
    /// # Returns
    ///
    /// Optional matching [`User`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn find_user_by_username(&self, username: String) -> Result<Option<User>, Error>;

    /// Finds a user by their primary key ID.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `id` - The numeric user ID.
    ///
    /// # Returns
    ///
    /// Optional matching [`User`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn find_user_by_id(&self, id: i32) -> Result<Option<User>, Error>;

    /// Finds a user by their upstream GitHub numeric user ID.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `github_id` - Upstream GitHub numeric ID.
    ///
    /// # Returns
    ///
    /// Optional matching [`User`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn find_user_by_github_id(&self, github_id: i64) -> Result<Option<User>, Error>;

    /// Finds a user by their upstream Google subject identifier (`sub`).
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `google_id` - Google OAuth subject identifier string.
    ///
    /// # Returns
    ///
    /// Optional matching [`User`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn find_user_by_google_id(&self, google_id: String) -> Result<Option<User>, Error>;

    /// Finds a user by their unique email address.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `email` - Email address string.
    ///
    /// # Returns
    ///
    /// Optional matching [`User`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn find_user_by_email(&self, email: String) -> Result<Option<User>, Error>;

    /// Creates a new user record.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `github_id` - Optional upstream GitHub ID.
    /// * `username` - Unique username.
    /// * `email` - Unique email address.
    /// * `password_hash` - Optional Argon2 password hash.
    ///
    /// # Returns
    ///
    /// Newly created [`User`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Conflict`] on duplicate constraints or [`Error`] on database failure.
    async fn create_user(
        &self,
        github_id: Option<i64>,
        username: String,
        email: String,
        password_hash: Option<String>,
    ) -> Result<User, Error>;

    /// Creates a new user record initialized via Google OAuth.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `google_id` - Upstream Google subject identifier (`sub`).
    /// * `username` - Unique username.
    /// * `email` - Unique email address.
    /// * `avatar_url` - Optional avatar URL.
    ///
    /// # Returns
    ///
    /// Newly created [`User`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Conflict`] on duplicate constraints or [`Error`] on database failure.
    async fn create_google_user(
        &self,
        google_id: String,
        username: String,
        email: String,
        avatar_url: Option<String>,
    ) -> Result<User, Error>;

    /// Upserts a user record keyed on their upstream GitHub ID.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `github_id` - Upstream GitHub numeric ID.
    /// * `username` - Updated username.
    /// * `email` - Updated email address.
    ///
    /// # Returns
    ///
    /// Upserted [`User`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn upsert_user(
        &self,
        github_id: i64,
        username: String,
        email: String,
    ) -> Result<User, Error>;

    /// Links an upstream GitHub numeric user ID to an existing [`User`] account.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `user_id` - Target user primary key ID.
    /// * `github_id` - Upstream GitHub numeric ID to link.
    ///
    /// # Returns
    ///
    /// Updated [`User`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Conflict`] if `github_id` is already linked to another user.
    async fn link_github_to_user(&self, user_id: i32, github_id: i64) -> Result<User, Error>;

    /// Links an upstream Google subject identifier (`sub`) to an existing [`User`] account.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `user_id` - Target user primary key ID.
    /// * `google_id` - Upstream Google subject identifier to link.
    ///
    /// # Returns
    ///
    /// Updated [`User`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Conflict`] if `google_id` is already linked to another user.
    async fn link_google_to_user(&self, user_id: i32, google_id: String) -> Result<User, Error>;

    /// Sets or updates the Argon2id password hash on an existing [`User`] account.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `user_id` - Target user primary key ID.
    /// * `password_hash` - Argon2 PHC hash string.
    ///
    /// # Returns
    ///
    /// Updated [`User`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn set_user_password(&self, user_id: i32, password_hash: String) -> Result<User, Error>;

    /// Unlinks an [`AuthMethod`] (`Password`, `GitHub`, or `Google`) from a [`User`] account,
    /// enforcing that at least one active authentication credential remains.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `user_id` - Target user primary key ID.
    /// * `provider` - The [`AuthMethod`] credential to remove.
    ///
    /// # Returns
    ///
    /// Updated [`User`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadRequest`] if the provider is not linked or if unlinking would leave
    /// the account with zero authentication methods, or [`Error::NotFound`] if the user does not exist.
    async fn unlink_provider_from_user(
        &self,
        user_id: i32,
        provider: AuthMethod,
    ) -> Result<User, Error>;

    /// Creates a new organization record.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `github_id` - Optional upstream GitHub organization ID.
    /// * `login` - Unique organization login handle.
    /// * `description` - Optional description.
    ///
    /// # Returns
    ///
    /// Newly created [`Organization`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn create_organization(
        &self,
        github_id: Option<i64>,
        login: String,
        description: Option<String>,
    ) -> Result<Organization, Error>;

    /// Atomically creates a new organization and assigns `owner_id` as [`OrgRole::Owner`].
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `owner_id` - User ID of the organization creator.
    /// * `login` - Unique organization login handle.
    /// * `description` - Optional description.
    ///
    /// # Returns
    ///
    /// Newly created [`Organization`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn create_organization_with_owner(
        &self,
        owner_id: i32,
        login: String,
        description: Option<String>,
    ) -> Result<Organization, Error>;

    /// Upserts an organization record keyed on its upstream GitHub ID.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `github_id` - Upstream GitHub organization ID.
    /// * `login` - Organization login handle.
    /// * `description` - Optional description.
    ///
    /// # Returns
    ///
    /// Upserted [`Organization`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn upsert_organization(
        &self,
        github_id: i64,
        login: String,
        description: Option<String>,
    ) -> Result<Organization, Error>;

    /// Retrieves an organization by its primary key ID.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `org_id` - Organization primary key ID.
    ///
    /// # Returns
    ///
    /// Optional matching [`Organization`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn get_organization(&self, org_id: i32) -> Result<Option<Organization>, Error>;

    /// Lists all organizations where `user_id` holds a membership role.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `user_id` - User primary key ID.
    ///
    /// # Returns
    ///
    /// Vector of [`Organization`] records.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn list_organizations_for_user(&self, user_id: i32) -> Result<Vec<Organization>, Error>;

    /// Associates a user with an organization under the specified [`OrgRole`].
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `org_id` - Organization primary key ID.
    /// * `user_id` - User primary key ID.
    /// * `role` - Assigned [`OrgRole`].
    ///
    /// # Returns
    ///
    /// Upserted [`OrganizationUser`] membership record.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn add_user_to_organization(
        &self,
        org_id: i32,
        user_id: i32,
        role: OrgRole,
    ) -> Result<OrganizationUser, Error>;

    /// Retrieves a user's [`OrgRole`] within an organization, if they are a member.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `org_id` - Organization primary key ID.
    /// * `user_id` - User primary key ID.
    ///
    /// # Returns
    ///
    /// Optional [`OrgRole`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn get_user_role(&self, org_id: i32, user_id: i32) -> Result<Option<OrgRole>, Error>;

    /// Creates a new repository within an organization.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `org_id` - Owning organization ID.
    /// * `github_id` - Optional upstream GitHub repository ID.
    /// * `name` - Repository name.
    /// * `description` - Optional description.
    ///
    /// # Returns
    ///
    /// Newly created [`Repository`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn create_repository(
        &self,
        org_id: i32,
        github_id: Option<i64>,
        name: String,
        description: Option<String>,
    ) -> Result<Repository, Error>;

    /// Updates a repository's topology, target languages, generator options, registries, PGP, and docs configuration.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `repo_id` - Target repository primary key ID.
    /// * `update` - [`UpdateRepositoryConfig`] changeset.
    ///
    /// # Returns
    ///
    /// Updated [`Repository`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn update_repository_config(
        &self,
        repo_id: i32,
        update: UpdateRepositoryConfig,
    ) -> Result<Repository, Error>;

    /// Upserts a repository keyed on its upstream GitHub ID.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `org_id` - Owning organization ID.
    /// * `github_id` - Upstream GitHub repository ID.
    /// * `name` - Repository name.
    /// * `description` - Optional description.
    ///
    /// # Returns
    ///
    /// Upserted [`Repository`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn upsert_repository(
        &self,
        org_id: i32,
        github_id: i64,
        name: String,
        description: Option<String>,
    ) -> Result<Repository, Error>;

    /// Retrieves a repository by its primary key ID.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `repo_id` - Repository primary key ID.
    ///
    /// # Returns
    ///
    /// Optional matching [`Repository`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn get_repository(&self, repo_id: i32) -> Result<Option<Repository>, Error>;

    /// Lists all repositories belonging to `org_id`.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `org_id` - Organization primary key ID.
    ///
    /// # Returns
    ///
    /// Vector of [`Repository`] records.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn list_repositories_for_org(&self, org_id: i32) -> Result<Vec<Repository>, Error>;

    /// Creates a new SDK release record for a repository.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `repo_id` - Parent repository ID.
    /// * `github_id` - Optional upstream GitHub release ID.
    /// * `tag_name` - Git tag name (e.g., `v1.0.0`).
    /// * `name` - Optional release title.
    /// * `body` - Optional release notes.
    ///
    /// # Returns
    ///
    /// Newly created [`Release`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn create_release(
        &self,
        repo_id: i32,
        github_id: Option<i64>,
        tag_name: String,
        name: Option<String>,
        body: Option<String>,
    ) -> Result<Release, Error>;

    /// Upserts a release record keyed on its upstream GitHub ID.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `repo_id` - Parent repository ID.
    /// * `github_id` - Upstream GitHub release ID.
    /// * `tag_name` - Git tag name.
    /// * `name` - Optional release title.
    /// * `body` - Optional release notes.
    ///
    /// # Returns
    ///
    /// Upserted [`Release`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn upsert_release(
        &self,
        repo_id: i32,
        github_id: i64,
        tag_name: String,
        name: Option<String>,
        body: Option<String>,
    ) -> Result<Release, Error>;

    /// Lists all releases belonging to `repo_id`.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `repo_id` - Repository primary key ID.
    ///
    /// # Returns
    ///
    /// Vector of [`Release`] records.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn list_releases_for_repo(&self, repo_id: i32) -> Result<Vec<Release>, Error>;

    /// Retrieves a release by its primary key ID.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `release_id` - Release primary key ID.
    ///
    /// # Returns
    ///
    /// Optional matching [`Release`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn get_release(&self, release_id: i32) -> Result<Option<Release>, Error>;

    /// Upserts an encrypted token for a user and [`TokenProvider`].
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `user_id` - Owning user ID.
    /// * `provider` - Validated [`TokenProvider`].
    /// * `encrypted_token` - Base64-encoded encrypted token payload.
    ///
    /// # Returns
    ///
    /// Upserted [`UserToken`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn upsert_user_token(
        &self,
        user_id: i32,
        provider: TokenProvider,
        encrypted_token: String,
    ) -> Result<UserToken, Error>;

    /// Lists all stored token records for `user_id`.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `user_id` - User primary key ID.
    ///
    /// # Returns
    ///
    /// Vector of [`UserToken`] records.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn list_user_tokens(&self, user_id: i32) -> Result<Vec<UserToken>, Error>;

    /// Deletes a stored token for `user_id` and `provider`.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `user_id` - User primary key ID.
    /// * `provider` - Validated [`TokenProvider`].
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn delete_user_token(&self, user_id: i32, provider: TokenProvider) -> Result<(), Error>;

    /// Records a new audit log event.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `org_id` - Organization primary key ID.
    /// * `repo_id` - Optional affected repository ID.
    /// * `user_id` - Acting user ID.
    /// * `action` - Auditable [`AuditAction`].
    /// * `metadata_json` - Optional structured event metadata.
    ///
    /// # Returns
    ///
    /// Newly created [`AuditLog`].
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn create_audit_log(
        &self,
        org_id: i32,
        repo_id: Option<i32>,
        user_id: i32,
        action: AuditAction,
        metadata_json: Option<serde_json::Value>,
    ) -> Result<AuditLog, Error>;

    /// Lists audit log events for `org_id` ordered newest-first.
    ///
    /// # Arguments
    ///
    /// * `&self` - The repository instance.
    /// * `org_id` - Organization primary key ID.
    /// * `limit` - Maximum rows to return.
    /// * `offset` - Number of rows to skip.
    ///
    /// # Returns
    ///
    /// Vector of [`AuditLog`] records.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] on database or pool failure.
    async fn list_audit_logs(
        &self,
        org_id: i32,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>, Error>;
}

/// PostgreSQL-backed implementation of [`CddRepository`].
pub struct PgRepository {
    /// R2D2 connection pool for PostgreSQL connections.
    pub pool: DbPool,
}

impl PgRepository {
    /// Checks out a pooled PostgreSQL connection, propagating [`Error::Pool`] on failure.
    ///
    /// # Arguments
    ///
    /// * `&self` - The [`PgRepository`] instance.
    ///
    /// # Returns
    ///
    /// Pooled PostgreSQL connection.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Pool`] if connection checkout times out or fails.
    pub fn get_conn(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, Error> {
        Ok(self.pool.get()?)
    }
}

#[async_trait]
impl CddRepository for PgRepository {
    async fn find_user_by_username(&self, username: String) -> Result<Option<User>, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            users::table
                .filter(users::username.eq(username))
                .first::<User>(&mut conn)
                .optional()
        })
        .await??)
    }

    async fn find_user_by_id(&self, id: i32) -> Result<Option<User>, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || users::table.find(id).first::<User>(&mut conn).optional()).await??)
    }

    async fn find_user_by_github_id(&self, github_id: i64) -> Result<Option<User>, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            users::table
                .filter(users::github_id.eq(github_id))
                .first::<User>(&mut conn)
                .optional()
        })
        .await??)
    }

    async fn find_user_by_google_id(&self, google_id: String) -> Result<Option<User>, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            users::table
                .filter(users::google_id.eq(google_id))
                .first::<User>(&mut conn)
                .optional()
        })
        .await??)
    }

    async fn find_user_by_email(&self, email: String) -> Result<Option<User>, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            users::table
                .filter(users::email.eq(email))
                .first::<User>(&mut conn)
                .optional()
        })
        .await??)
    }

    async fn create_user(
        &self,
        github_id: Option<i64>,
        username: String,
        email: String,
        password_hash: Option<String>,
    ) -> Result<User, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            let new_user = NewUser {
                github_id,
                username: &username,
                email: &email,
                password_hash: password_hash.as_deref(),
                google_id: None,
                avatar_url: None,
            };
            diesel::insert_into(users::table)
                .values(&new_user)
                .get_result::<User>(&mut conn)
        })
        .await??)
    }

    async fn create_google_user(
        &self,
        google_id: String,
        username: String,
        email: String,
        avatar_url: Option<String>,
    ) -> Result<User, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            let new_user = NewUser {
                github_id: None,
                username: &username,
                email: &email,
                password_hash: None,
                google_id: Some(&google_id),
                avatar_url: avatar_url.as_deref(),
            };
            diesel::insert_into(users::table)
                .values(&new_user)
                .get_result::<User>(&mut conn)
        })
        .await??)
    }

    async fn upsert_user(
        &self,
        github_id: i64,
        username: String,
        email: String,
    ) -> Result<User, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            let new_user = NewUser {
                github_id: Some(github_id),
                username: &username,
                email: &email,
                password_hash: None,
                google_id: None,
                avatar_url: None,
            };
            diesel::insert_into(users::table)
                .values(&new_user)
                .on_conflict(users::github_id)
                .do_update()
                .set((users::username.eq(&username), users::email.eq(&email)))
                .get_result::<User>(&mut conn)
        })
        .await??)
    }

    async fn link_github_to_user(&self, user_id: i32, github_id: i64) -> Result<User, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            diesel::update(users::table.find(user_id))
                .set(users::github_id.eq(Some(github_id)))
                .get_result::<User>(&mut conn)
        })
        .await??)
    }

    async fn link_google_to_user(&self, user_id: i32, google_id: String) -> Result<User, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            diesel::update(users::table.find(user_id))
                .set(users::google_id.eq(Some(google_id)))
                .get_result::<User>(&mut conn)
        })
        .await??)
    }

    async fn set_user_password(&self, user_id: i32, password_hash: String) -> Result<User, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            diesel::update(users::table.find(user_id))
                .set(users::password_hash.eq(Some(password_hash)))
                .get_result::<User>(&mut conn)
        })
        .await??)
    }

    async fn unlink_provider_from_user(
        &self,
        user_id: i32,
        provider: AuthMethod,
    ) -> Result<User, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            conn.transaction::<User, Error, _>(|tx_conn| {
                let Some(user) = users::table
                    .find(user_id)
                    .first::<User>(tx_conn)
                    .optional()?
                else {
                    return Err(Error::NotFound("User not found".into()));
                };

                let connected = user.connected_providers();
                if !connected.contains(&provider) {
                    return Err(Error::BadRequest(format!(
                        "Authentication method '{provider}' is not linked to this account"
                    )));
                }
                if connected.len() <= 1 {
                    return Err(Error::BadRequest(
                        "Cannot disconnect the last remaining authentication method".into(),
                    ));
                }

                let updated = match provider {
                    AuthMethod::Password => diesel::update(users::table.find(user_id))
                        .set(users::password_hash.eq(None::<String>))
                        .get_result::<User>(tx_conn)?,
                    AuthMethod::GitHub => diesel::update(users::table.find(user_id))
                        .set(users::github_id.eq(None::<i64>))
                        .get_result::<User>(tx_conn)?,
                    AuthMethod::Google => diesel::update(users::table.find(user_id))
                        .set(users::google_id.eq(None::<String>))
                        .get_result::<User>(tx_conn)?,
                };
                Ok(updated)
            })
        })
        .await?
    }

    async fn create_organization(
        &self,
        github_id: Option<i64>,
        login: String,
        description: Option<String>,
    ) -> Result<Organization, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            let new_org = NewOrganization {
                github_id,
                login: &login,
                description: description.as_deref(),
                github_org_name: Some(&login),
            };
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .get_result::<Organization>(&mut conn)
        })
        .await??)
    }

    async fn create_organization_with_owner(
        &self,
        owner_id: i32,
        login: String,
        description: Option<String>,
    ) -> Result<Organization, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            conn.transaction::<Organization, diesel::result::Error, _>(|tx_conn| {
                let new_org = NewOrganization {
                    github_id: None,
                    login: &login,
                    description: description.as_deref(),
                    github_org_name: Some(&login),
                };
                let org = diesel::insert_into(organizations::table)
                    .values(&new_org)
                    .get_result::<Organization>(tx_conn)?;

                let owner_link = NewOrganizationUser {
                    organization_id: org.id,
                    user_id: owner_id,
                    role: OrgRole::Owner.as_str(),
                };
                diesel::insert_into(organization_users::table)
                    .values(&owner_link)
                    .execute(tx_conn)?;

                Ok(org)
            })
        })
        .await??)
    }

    async fn upsert_organization(
        &self,
        github_id: i64,
        login: String,
        description: Option<String>,
    ) -> Result<Organization, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            let new_org = NewOrganization {
                github_id: Some(github_id),
                login: &login,
                description: description.as_deref(),
                github_org_name: Some(&login),
            };
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict(organizations::github_id)
                .do_update()
                .set((
                    organizations::login.eq(&login),
                    organizations::description.eq(description.as_deref()),
                ))
                .get_result::<Organization>(&mut conn)
        })
        .await??)
    }

    async fn get_organization(&self, org_id: i32) -> Result<Option<Organization>, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            organizations::table
                .find(org_id)
                .first::<Organization>(&mut conn)
                .optional()
        })
        .await??)
    }

    async fn list_organizations_for_user(&self, user_id: i32) -> Result<Vec<Organization>, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            organizations::table
                .inner_join(organization_users::table)
                .filter(organization_users::user_id.eq(user_id))
                .select(Organization::as_select())
                .order(organizations::id.asc())
                .load::<Organization>(&mut conn)
        })
        .await??)
    }

    async fn add_user_to_organization(
        &self,
        org_id: i32,
        user_id: i32,
        role: OrgRole,
    ) -> Result<OrganizationUser, Error> {
        let mut conn = self.get_conn()?;
        let role_str = role.as_str().to_string();
        Ok(web::block(move || {
            let new_link = NewOrganizationUser {
                organization_id: org_id,
                user_id,
                role: &role_str,
            };
            diesel::insert_into(organization_users::table)
                .values(&new_link)
                .on_conflict((
                    organization_users::organization_id,
                    organization_users::user_id,
                ))
                .do_update()
                .set(organization_users::role.eq(&role_str))
                .get_result::<OrganizationUser>(&mut conn)
        })
        .await??)
    }

    async fn get_user_role(&self, org_id: i32, user_id: i32) -> Result<Option<OrgRole>, Error> {
        let mut conn = self.get_conn()?;
        let role_opt: Option<String> = web::block(move || {
            organization_users::table
                .filter(organization_users::organization_id.eq(org_id))
                .filter(organization_users::user_id.eq(user_id))
                .select(organization_users::role)
                .first::<String>(&mut conn)
                .optional()
        })
        .await??;

        role_opt.as_deref().map(OrgRole::from_str).transpose()
    }

    async fn create_repository(
        &self,
        org_id: i32,
        github_id: Option<i64>,
        name: String,
        description: Option<String>,
    ) -> Result<Repository, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            let new_repo = NewRepository {
                organization_id: org_id,
                github_id,
                name: &name,
                description: description.as_deref(),
                github_org_name: "",
                topology_mode: "monorepo",
                monorepo_name: None,
                polyrepo_names_json: serde_json::json!({}),
                target_languages_json: serde_json::json!([]),
                language_options_json: serde_json::json!({}),
                release_registries_json: serde_json::json!([]),
                enable_pgp_signing: false,
                docs_config_json: serde_json::json!({"enabled": true, "target": "official"}),
            };
            diesel::insert_into(repositories::table)
                .values(&new_repo)
                .get_result::<Repository>(&mut conn)
        })
        .await??)
    }

    async fn update_repository_config(
        &self,
        repo_id: i32,
        update: UpdateRepositoryConfig,
    ) -> Result<Repository, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            diesel::update(repositories::table.find(repo_id))
                .set(&update)
                .get_result::<Repository>(&mut conn)
        })
        .await??)
    }

    async fn upsert_repository(
        &self,
        org_id: i32,
        github_id: i64,
        name: String,
        description: Option<String>,
    ) -> Result<Repository, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            let new_repo = NewRepository {
                organization_id: org_id,
                github_id: Some(github_id),
                name: &name,
                description: description.as_deref(),
                github_org_name: "",
                topology_mode: "monorepo",
                monorepo_name: None,
                polyrepo_names_json: serde_json::json!({}),
                target_languages_json: serde_json::json!([]),
                language_options_json: serde_json::json!({}),
                release_registries_json: serde_json::json!([]),
                enable_pgp_signing: false,
                docs_config_json: serde_json::json!({"enabled": true, "target": "official"}),
            };
            diesel::insert_into(repositories::table)
                .values(&new_repo)
                .on_conflict(repositories::github_id)
                .do_update()
                .set((
                    repositories::name.eq(&name),
                    repositories::description.eq(description.as_deref()),
                    repositories::organization_id.eq(org_id),
                ))
                .get_result::<Repository>(&mut conn)
        })
        .await??)
    }

    async fn get_repository(&self, repo_id: i32) -> Result<Option<Repository>, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            repositories::table
                .find(repo_id)
                .first::<Repository>(&mut conn)
                .optional()
        })
        .await??)
    }

    async fn list_repositories_for_org(&self, org_id: i32) -> Result<Vec<Repository>, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            repositories::table
                .filter(repositories::organization_id.eq(org_id))
                .order(repositories::id.asc())
                .load::<Repository>(&mut conn)
        })
        .await??)
    }

    async fn create_release(
        &self,
        repo_id: i32,
        github_id: Option<i64>,
        tag_name: String,
        name: Option<String>,
        body: Option<String>,
    ) -> Result<Release, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            let new_release = NewRelease {
                repository_id: repo_id,
                github_id,
                tag_name: &tag_name,
                name: name.as_deref(),
                body: body.as_deref(),
            };
            diesel::insert_into(releases::table)
                .values(&new_release)
                .get_result::<Release>(&mut conn)
        })
        .await??)
    }

    async fn upsert_release(
        &self,
        repo_id: i32,
        github_id: i64,
        tag_name: String,
        name: Option<String>,
        body: Option<String>,
    ) -> Result<Release, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            let new_release = NewRelease {
                repository_id: repo_id,
                github_id: Some(github_id),
                tag_name: &tag_name,
                name: name.as_deref(),
                body: body.as_deref(),
            };
            diesel::insert_into(releases::table)
                .values(&new_release)
                .on_conflict(releases::github_id)
                .do_update()
                .set((
                    releases::tag_name.eq(&tag_name),
                    releases::name.eq(name.as_deref()),
                    releases::body.eq(body.as_deref()),
                    releases::repository_id.eq(repo_id),
                ))
                .get_result::<Release>(&mut conn)
        })
        .await??)
    }

    async fn list_releases_for_repo(&self, repo_id: i32) -> Result<Vec<Release>, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            releases::table
                .filter(releases::repository_id.eq(repo_id))
                .order(releases::id.asc())
                .load::<Release>(&mut conn)
        })
        .await??)
    }

    async fn get_release(&self, release_id: i32) -> Result<Option<Release>, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            releases::table
                .find(release_id)
                .first::<Release>(&mut conn)
                .optional()
        })
        .await??)
    }

    async fn upsert_user_token(
        &self,
        user_id: i32,
        provider: TokenProvider,
        encrypted_token: String,
    ) -> Result<UserToken, Error> {
        let mut conn = self.get_conn()?;
        let provider_str = provider.as_str().to_string();
        Ok(web::block(move || {
            let new_token = NewUserToken {
                user_id,
                provider: &provider_str,
                encrypted_token: &encrypted_token,
            };
            diesel::insert_into(user_tokens::table)
                .values(&new_token)
                .on_conflict((user_tokens::user_id, user_tokens::provider))
                .do_update()
                .set((
                    user_tokens::encrypted_token.eq(&encrypted_token),
                    user_tokens::updated_at.eq(diesel::dsl::now),
                ))
                .get_result::<UserToken>(&mut conn)
        })
        .await??)
    }

    async fn list_user_tokens(&self, user_id: i32) -> Result<Vec<UserToken>, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            user_tokens::table
                .filter(user_tokens::user_id.eq(user_id))
                .order(user_tokens::provider.asc())
                .load::<UserToken>(&mut conn)
        })
        .await??)
    }

    async fn delete_user_token(&self, user_id: i32, provider: TokenProvider) -> Result<(), Error> {
        let mut conn = self.get_conn()?;
        let provider_str = provider.as_str().to_string();
        web::block(move || {
            diesel::delete(
                user_tokens::table
                    .filter(user_tokens::user_id.eq(user_id))
                    .filter(user_tokens::provider.eq(&provider_str)),
            )
            .execute(&mut conn)
        })
        .await??;
        Ok(())
    }

    async fn create_audit_log(
        &self,
        org_id: i32,
        repo_id: Option<i32>,
        user_id: i32,
        action: AuditAction,
        metadata_json: Option<serde_json::Value>,
    ) -> Result<AuditLog, Error> {
        let mut conn = self.get_conn()?;
        let action_str = action.as_str().to_string();
        Ok(web::block(move || {
            let new_log = NewAuditLog {
                org_id,
                repo_id,
                user_id,
                action: &action_str,
                metadata_json,
            };
            diesel::insert_into(audit_logs::table)
                .values(&new_log)
                .get_result::<AuditLog>(&mut conn)
        })
        .await??)
    }

    async fn list_audit_logs(
        &self,
        org_id: i32,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>, Error> {
        let mut conn = self.get_conn()?;
        Ok(web::block(move || {
            audit_logs::table
                .filter(audit_logs::org_id.eq(org_id))
                .order(audit_logs::timestamp.desc())
                .limit(limit)
                .offset(offset)
                .load::<AuditLog>(&mut conn)
        })
        .await??)
    }
}

#[cfg(test)]
mod tests {
    //! Focused integration tests for `PgRepository`.

    use super::*;
    use diesel::r2d2::{ConnectionManager, Pool};
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::time::SystemTime;
    use uuid::Uuid;

    /// Global atomic counter for generating non-colliding test GitHub IDs.
    static COUNTER: AtomicI64 = AtomicI64::new(1);

    /// Builds a test database pool without calling `std::process::exit`.
    fn get_test_pool() -> Result<DbPool, Error> {
        let default_url = "postgres://postgres:password@localhost/cdd_test".to_string();
        let database_url = std::env::var("DATABASE_URL").unwrap_or(default_url);
        let manager = ConnectionManager::<diesel::PgConnection>::new(database_url);
        Ok(Pool::builder().max_size(2).build(manager)?)
    }

    /// Generates a unique test identifier with the given `prefix`.
    fn generate_unique_name(prefix: &str) -> String {
        let raw = Uuid::new_v4().simple().to_string();
        format!("{prefix}_{}", &raw[..10])
    }

    /// Generates a unique positive `i64` GitHub ID for tests.
    fn next_github_id() -> Result<i64, Error> {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| Error::InternalError)?
            .as_nanos();
        let base = i64::try_from(nanos % 1_000_000_000).map_err(|_| Error::InternalError)?;
        Ok(base.saturating_add(COUNTER.fetch_add(1, Ordering::Relaxed)))
    }

    #[actix_web::test]
    async fn test_user_and_token_crud() -> Result<(), Error> {
        let repo = PgRepository {
            pool: get_test_pool()?,
        };
        let gh_id = next_github_id()?;
        let username = generate_unique_name("user");
        let email = format!("{username}@example.com");

        let user = repo
            .create_user(
                Some(gh_id),
                username.clone(),
                email.clone(),
                Some("hash".into()),
            )
            .await?;
        assert_eq!(user.username, username);
        assert_eq!(user.github_id, Some(gh_id));

        let by_name = repo
            .find_user_by_username(username.clone())
            .await?
            .ok_or(Error::InternalError)?;
        assert_eq!(by_name.id, user.id);

        let by_id = repo
            .find_user_by_id(user.id)
            .await?
            .ok_or(Error::InternalError)?;
        assert_eq!(by_id.email, email);

        let by_gh = repo
            .find_user_by_github_id(gh_id)
            .await?
            .ok_or(Error::InternalError)?;
        assert_eq!(by_gh.id, user.id);

        let by_email = repo
            .find_user_by_email(email.clone())
            .await?
            .ok_or(Error::InternalError)?;
        assert_eq!(by_email.id, user.id);

        let updated_email = format!("updated_{email}");
        let upserted = repo
            .upsert_user(gh_id, username.clone(), updated_email.clone())
            .await?;
        assert_eq!(upserted.id, user.id);
        assert_eq!(upserted.email, updated_email);

        let missing = repo.find_user_by_id(i32::MAX).await?;
        assert!(missing.is_none());

        repo.upsert_user_token(user.id, TokenProvider::GitHub, "enc_1".into())
            .await?;
        repo.upsert_user_token(user.id, TokenProvider::GitHub, "enc_2".into())
            .await?;

        let tokens = repo.list_user_tokens(user.id).await?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].provider, "github");
        assert_eq!(tokens[0].encrypted_token, "enc_2");

        repo.delete_user_token(user.id, TokenProvider::GitHub)
            .await?;
        assert_eq!(repo.list_user_tokens(user.id).await?, Vec::new());

        Ok(())
    }

    #[actix_web::test]
    async fn test_google_user_and_account_linking() -> Result<(), Error> {
        let repo = PgRepository {
            pool: get_test_pool()?,
        };
        let g_sub = generate_unique_name("google_sub");
        let uname = generate_unique_name("guser");
        let email = format!("{uname}@example.com");

        let g_user = repo
            .create_google_user(
                g_sub.clone(),
                uname.clone(),
                email.clone(),
                Some("https://example.com/avatar.png".into()),
            )
            .await?;
        assert_eq!(g_user.google_id.as_deref(), Some(g_sub.as_str()));
        assert_eq!(
            g_user.avatar_url.as_deref(),
            Some("https://example.com/avatar.png")
        );

        let found_g = repo
            .find_user_by_google_id(g_sub.clone())
            .await?
            .ok_or(Error::InternalError)?;
        assert_eq!(found_g.id, g_user.id);

        // Cannot unlink the only remaining auth method (Google)
        let last_err = repo
            .unlink_provider_from_user(g_user.id, AuthMethod::Google)
            .await;
        assert!(matches!(last_err, Err(Error::BadRequest(_))));

        // Cannot unlink a method that is not currently linked (Password)
        let not_linked_err = repo
            .unlink_provider_from_user(g_user.id, AuthMethod::Password)
            .await;
        assert!(matches!(not_linked_err, Err(Error::BadRequest(_))));

        // Unlinking for a non-existent user returns NotFound
        let nf_err = repo
            .unlink_provider_from_user(i32::MAX, AuthMethod::Google)
            .await;
        assert!(matches!(nf_err, Err(Error::NotFound(_))));

        // Link GitHub and Password to the user
        let gh_id = next_github_id()?;
        let with_gh = repo.link_github_to_user(g_user.id, gh_id).await?;
        assert_eq!(with_gh.github_id, Some(gh_id));

        let with_pw = repo
            .set_user_password(g_user.id, "argon2_hash".into())
            .await?;
        assert_eq!(with_pw.password_hash.as_deref(), Some("argon2_hash"));

        // Unlink Google, GitHub, and then re-link Google so we test all 3 unlink branches
        let after_unlink_google = repo
            .unlink_provider_from_user(g_user.id, AuthMethod::Google)
            .await?;
        assert!(after_unlink_google.google_id.is_none());

        let g_sub_2 = generate_unique_name("google_sub2");
        let relinked_g = repo.link_google_to_user(g_user.id, g_sub_2.clone()).await?;
        assert_eq!(relinked_g.google_id.as_deref(), Some(g_sub_2.as_str()));

        let after_unlink_gh = repo
            .unlink_provider_from_user(g_user.id, AuthMethod::GitHub)
            .await?;
        assert!(after_unlink_gh.github_id.is_none());

        let after_unlink_pw = repo
            .unlink_provider_from_user(g_user.id, AuthMethod::Password)
            .await?;
        assert!(after_unlink_pw.password_hash.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn test_org_repo_release_and_audit_crud() -> Result<(), Error> {
        let repo = PgRepository {
            pool: get_test_pool()?,
        };
        let user_name = generate_unique_name("owner");
        let user = repo
            .create_user(
                None,
                user_name.clone(),
                format!("{user_name}@example.com"),
                None,
            )
            .await?;

        // Test transactional create_organization_with_owner
        let tx_org_login = generate_unique_name("txorg");
        let tx_org = repo
            .create_organization_with_owner(user.id, tx_org_login.clone(), Some("tx desc".into()))
            .await?;
        assert_eq!(tx_org.login, tx_org_login);
        let owner_role = repo.get_user_role(tx_org.id, user.id).await?;
        assert_eq!(owner_role, Some(OrgRole::Owner));

        let user_orgs = repo.list_organizations_for_user(user.id).await?;
        assert!(user_orgs.iter().any(|o| o.id == tx_org.id));

        // Test create_organization and upsert_organization
        let gh_org = next_github_id()?;
        let org_login = generate_unique_name("org");
        let org = repo
            .create_organization(Some(gh_org), org_login.clone(), Some("desc".into()))
            .await?;
        let fetched_org = repo
            .get_organization(org.id)
            .await?
            .ok_or(Error::InternalError)?;
        assert_eq!(fetched_org.id, org.id);

        let upserted_org = repo
            .upsert_organization(gh_org, org_login, Some("updated desc".into()))
            .await?;
        assert_eq!(upserted_org.id, org.id);
        assert_eq!(upserted_org.description.as_deref(), Some("updated desc"));

        repo.add_user_to_organization(org.id, user.id, OrgRole::Admin)
            .await?;
        assert_eq!(
            repo.get_user_role(org.id, user.id).await?,
            Some(OrgRole::Admin)
        );

        // Repositories + update_repository_config
        let gh_repo = next_github_id()?;
        let repo_name = generate_unique_name("repo");
        let created_repo = repo
            .create_repository(
                org.id,
                Some(gh_repo),
                repo_name.clone(),
                Some("rdesc".into()),
            )
            .await?;
        assert_eq!(created_repo.name, repo_name);

        let updated_cfg = repo
            .update_repository_config(
                created_repo.id,
                UpdateRepositoryConfig {
                    github_org_name: Some("AcmeOrg".into()),
                    topology_mode: Some("polyrepo".into()),
                    enable_pgp_signing: Some(true),
                    target_languages_json: Some(serde_json::json!(["rust", "python"])),
                    ..UpdateRepositoryConfig::default()
                },
            )
            .await?;
        assert_eq!(updated_cfg.github_org_name, "AcmeOrg");
        assert_eq!(updated_cfg.topology_mode, "polyrepo");
        assert!(updated_cfg.enable_pgp_signing);

        let fetched_repo = repo
            .get_repository(created_repo.id)
            .await?
            .ok_or(Error::InternalError)?;
        assert_eq!(fetched_repo.id, created_repo.id);

        let upserted_repo = repo
            .upsert_repository(org.id, gh_repo, repo_name, Some("rdesc2".into()))
            .await?;
        assert_eq!(upserted_repo.id, created_repo.id);

        let org_repos = repo.list_repositories_for_org(org.id).await?;
        assert_eq!(org_repos.len(), 1);

        // Releases
        let gh_rel = next_github_id()?;
        let release = repo
            .create_release(
                created_repo.id,
                Some(gh_rel),
                "v1.0.0".into(),
                Some("R1".into()),
                Some("Notes".into()),
            )
            .await?;
        assert_eq!(release.tag_name, "v1.0.0");

        let upserted_release = repo
            .upsert_release(
                created_repo.id,
                gh_rel,
                "v1.0.0".into(),
                Some("R1-up".into()),
                Some("Notes-up".into()),
            )
            .await?;
        assert_eq!(upserted_release.id, release.id);

        let repo_releases = repo.list_releases_for_repo(created_repo.id).await?;
        assert_eq!(repo_releases.len(), 1);
        let fetched_rel = repo
            .get_release(release.id)
            .await?
            .ok_or(Error::InternalError)?;
        assert_eq!(fetched_rel.id, release.id);

        // Audit logs
        let log = repo
            .create_audit_log(
                org.id,
                Some(created_repo.id),
                user.id,
                AuditAction::RepoCreated,
                Some(serde_json::json!({"repo": created_repo.id})),
            )
            .await?;
        assert_eq!(log.action, "repo_created");

        let logs = repo.list_audit_logs(org.id, 10, 0).await?;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].action, "repo_created");

        Ok(())
    }

    #[actix_web::test]
    async fn test_invalid_pool_connection_returns_pool_error() {
        let manager =
            diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new("postgres://invalid");
        let pool = diesel::r2d2::Pool::builder()
            .connection_timeout(std::time::Duration::from_millis(1))
            .build_unchecked(manager);
        let repo_invalid = PgRepository { pool };
        let res = repo_invalid.get_conn();
        assert!(matches!(res, Err(Error::Pool(_))));
    }
}
