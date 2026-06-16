#![cfg(not(tarpaulin_include))]
//!n//! Database repository module.n

use crate::db::models::*;
use crate::db::schema::*;
use actix_web::web;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::result::Error;
#[cfg(test)]
use mockall::automock;

/// Database connection pool type alias
pub type DbPool = r2d2::Pool<ConnectionManager<diesel::PgConnection>>;

/// Database repository trait for handling CRUD operations.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait CddRepository: Send + Sync {
    /// Find a user by username
    async fn find_user_by_username(&self, username: String) -> Result<Option<User>, Error>;
    /// Find a user by id
    async fn find_user_by_id(&self, id: i32) -> Result<Option<User>, Error>;
    /// Create a new user
    async fn create_user(
        &self,
        github_id: Option<i64>,
        username: String,
        email: String,
        password_hash: Option<String>,
    ) -> Result<User, Error>;

    /// Upsert a user based on GitHub ID
    async fn upsert_user(
        &self,
        github_id: i64,
        username: String,
        email: String,
    ) -> Result<User, Error>;

    /// Create an organization
    async fn create_organization(
        &self,
        github_id: Option<i64>,
        login: String,
        description: Option<String>,
    ) -> Result<Organization, Error>;

    /// Upsert an organization based on GitHub ID
    async fn upsert_organization(
        &self,
        github_id: i64,
        login: String,
        description: Option<String>,
    ) -> Result<Organization, Error>;

    /// Get an organization
    async fn get_organization(&self, org_id: i32) -> Result<Option<Organization>, Error>;

    /// Link user to organization
    async fn add_user_to_organization(
        &self,
        org_id: i32,
        user_id: i32,
        role: String,
    ) -> Result<OrganizationUser, Error>;

    /// Check user role in organization
    async fn get_user_role(&self, org_id: i32, user_id: i32) -> Result<Option<String>, Error>;

    /// Create a repository
    async fn create_repository(
        &self,
        org_id: i32,
        github_id: Option<i64>,
        name: String,
        description: Option<String>,
    ) -> Result<Repository, Error>;

    /// Upsert a repository based on GitHub ID
    async fn upsert_repository(
        &self,
        org_id: i32,
        github_id: i64,
        name: String,
        description: Option<String>,
    ) -> Result<Repository, Error>;

    /// Get a repository
    async fn get_repository(&self, repo_id: i32) -> Result<Option<Repository>, Error>;

    /// Create a release
    async fn create_release(
        &self,
        repo_id: i32,
        github_id: Option<i64>,
        tag_name: String,
        name: Option<String>,
        body: Option<String>,
    ) -> Result<Release, Error>;

    /// Upsert a release based on GitHub ID
    async fn upsert_release(
        &self,
        repo_id: i32,
        github_id: i64,
        tag_name: String,
        name: Option<String>,
        body: Option<String>,
    ) -> Result<Release, Error>;

    /// Upsert a user token
    async fn upsert_user_token(
        &self,
        user_id: i32,
        provider: String,
        encrypted_token: String,
    ) -> Result<UserToken, Error>;

    /// List a user's tokens
    async fn list_user_tokens(&self, user_id: i32) -> Result<Vec<UserToken>, Error>;

    /// Delete a user token
    async fn delete_user_token(&self, user_id: i32, provider: String) -> Result<(), Error>;

    /// Create an audit log entry
    async fn create_audit_log(
        &self,
        org_id: i32,
        repo_id: Option<i32>,
        user_id: i32,
        action: String,
        metadata_json: Option<serde_json::Value>,
    ) -> Result<AuditLog, Error>;

    /// List audit logs for an organization
    async fn list_audit_logs(
        &self,
        org_id: i32,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>, Error>;
}

impl PgRepository {
    /// Helper to get a database connection
    pub fn get_conn(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, diesel::result::Error>
    {
        self.pool.get().map_err(|_| diesel::result::Error::NotFound)
    }
}

/// Postgres implementation of CddRepository
pub struct PgRepository {
    /// The database connection pool
    pub pool: DbPool,
}

#[async_trait]
impl CddRepository for PgRepository {
    async fn find_user_by_username(&self, username: String) -> Result<Option<User>, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            users::table
                .filter(users::username.eq(username))
                .first::<User>(&mut conn)
                .optional()
        })
        .await
        .map_err(|_| diesel::result::Error::NotFound)?
    }

    async fn find_user_by_id(&self, id: i32) -> Result<Option<User>, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || users::table.find(id).first::<User>(&mut conn).optional())
            .await
            .map_err(|_| Error::NotFound)?
    }

    async fn create_user(
        &self,
        github_id: Option<i64>,
        username: String,
        email: String,
        password_hash: Option<String>,
    ) -> Result<User, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            let new_user = NewUser {
                github_id,
                username: &username,
                email: &email,
                password_hash: password_hash.as_deref(),
            };
            diesel::insert_into(users::table)
                .values(&new_user)
                .get_result::<User>(&mut conn)
        })
        .await
        .map_err(|_| Error::NotFound)?
    }

    async fn upsert_user(
        &self,
        github_id: i64,
        username: String,
        email: String,
    ) -> Result<User, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            let new_user = NewUser {
                github_id: Some(github_id),
                username: &username,
                email: &email,
                password_hash: None,
            };
            diesel::insert_into(users::table)
                .values(&new_user)
                .on_conflict(users::github_id)
                .do_update()
                .set((users::username.eq(&username), users::email.eq(&email)))
                .get_result::<User>(&mut conn)
        })
        .await
        .map_err(|_| Error::NotFound)?
    }

    async fn create_organization(
        &self,
        github_id: Option<i64>,
        login: String,
        description: Option<String>,
    ) -> Result<Organization, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            let new_org = NewOrganization {
                github_id,
                login: &login,
                description: description.as_deref(),
            };
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .get_result::<Organization>(&mut conn)
        })
        .await
        .map_err(|_| Error::NotFound)?
    }

    async fn upsert_organization(
        &self,
        github_id: i64,
        login: String,
        description: Option<String>,
    ) -> Result<Organization, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            let new_org = NewOrganization {
                github_id: Some(github_id),
                login: &login,
                description: description.as_deref(),
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
        .await
        .map_err(|_| Error::NotFound)?
    }

    async fn get_organization(&self, org_id: i32) -> Result<Option<Organization>, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            organizations::table
                .find(org_id)
                .first::<Organization>(&mut conn)
                .optional()
        })
        .await
        .map_err(|_| Error::NotFound)?
    }

    async fn add_user_to_organization(
        &self,
        org_id: i32,
        user_id: i32,
        role: String,
    ) -> Result<OrganizationUser, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            let new_link = NewOrganizationUser {
                organization_id: org_id,
                user_id,
                role: &role,
            };
            diesel::insert_into(organization_users::table)
                .values(&new_link)
                .on_conflict((
                    organization_users::organization_id,
                    organization_users::user_id,
                ))
                .do_update()
                .set(organization_users::role.eq(&role))
                .get_result::<OrganizationUser>(&mut conn)
        })
        .await
        .map_err(|_| Error::NotFound)?
    }

    async fn get_user_role(&self, org_id: i32, user_id: i32) -> Result<Option<String>, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            organization_users::table
                .filter(organization_users::organization_id.eq(org_id))
                .filter(organization_users::user_id.eq(user_id))
                .select(organization_users::role)
                .first::<String>(&mut conn)
                .optional()
        })
        .await
        .map_err(|_| Error::NotFound)?
    }

    async fn create_repository(
        &self,
        org_id: i32,
        github_id: Option<i64>,
        name: String,
        description: Option<String>,
    ) -> Result<Repository, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            let new_repo = NewRepository {
                organization_id: org_id,
                github_id,
                name: &name,
                description: description.as_deref(),
            };
            diesel::insert_into(repositories::table)
                .values(&new_repo)
                .get_result::<Repository>(&mut conn)
        })
        .await
        .map_err(|_| Error::NotFound)?
    }

    async fn upsert_repository(
        &self,
        org_id: i32,
        github_id: i64,
        name: String,
        description: Option<String>,
    ) -> Result<Repository, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            let new_repo = NewRepository {
                organization_id: org_id,
                github_id: Some(github_id),
                name: &name,
                description: description.as_deref(),
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
        .await
        .map_err(|_| Error::NotFound)?
    }

    async fn get_repository(&self, repo_id: i32) -> Result<Option<Repository>, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            repositories::table
                .find(repo_id)
                .first::<Repository>(&mut conn)
                .optional()
        })
        .await
        .map_err(|_| Error::NotFound)?
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
        web::block(move || {
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
        .await
        .map_err(|_| Error::NotFound)?
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
        web::block(move || {
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
        .await
        .map_err(|_| Error::NotFound)?
    }

    async fn upsert_user_token(
        &self,
        user_id: i32,
        provider: String,
        encrypted_token: String,
    ) -> Result<UserToken, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            let new_token = NewUserToken {
                user_id,
                provider: &provider,
                encrypted_token: &encrypted_token,
            };
            diesel::insert_into(user_tokens::table)
                .values(&new_token)
                .on_conflict((user_tokens::user_id, user_tokens::provider))
                .do_update()
                .set(user_tokens::encrypted_token.eq(&encrypted_token))
                .get_result::<UserToken>(&mut conn)
        })
        .await
        .map_err(|_| Error::NotFound)?
    }

    async fn list_user_tokens(&self, user_id: i32) -> Result<Vec<UserToken>, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            user_tokens::table
                .filter(user_tokens::user_id.eq(user_id))
                .load::<UserToken>(&mut conn)
        })
        .await
        .map_err(|_| Error::NotFound)?
    }

    async fn delete_user_token(&self, user_id: i32, provider: String) -> Result<(), Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            diesel::delete(
                user_tokens::table
                    .filter(user_tokens::user_id.eq(user_id))
                    .filter(user_tokens::provider.eq(&provider)),
            )
            .execute(&mut conn)
        })
        .await
        .map_err(|_| Error::NotFound)??;
        Ok(())
    }

    async fn create_audit_log(
        &self,
        org_id: i32,
        repo_id: Option<i32>,
        user_id: i32,
        action: String,
        metadata_json: Option<serde_json::Value>,
    ) -> Result<AuditLog, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            let new_log = NewAuditLog {
                org_id,
                repo_id,
                user_id,
                action: &action,
                metadata_json,
            };
            diesel::insert_into(audit_logs::table)
                .values(&new_log)
                .get_result::<AuditLog>(&mut conn)
        })
        .await
        .map_err(|_| Error::NotFound)?
    }

    async fn list_audit_logs(
        &self,
        org_id: i32,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>, Error> {
        let mut conn = self.get_conn()?;
        web::block(move || {
            audit_logs::table
                .filter(audit_logs::org_id.eq(org_id))
                .order(audit_logs::timestamp.desc())
                .limit(limit)
                .offset(offset)
                .load::<AuditLog>(&mut conn)
        })
        .await
        .map_err(|_| Error::NotFound)?
    }
}
