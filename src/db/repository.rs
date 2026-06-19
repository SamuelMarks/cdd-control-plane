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

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::r2d2::{ConnectionManager, Pool};
    use uuid::Uuid;

    fn get_test_pool() -> DbPool {
        let database_url = std::env::var("CDD__DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost/cdd_test".to_string());
        let manager = ConnectionManager::<diesel::PgConnection>::new(database_url);
        Pool::builder()
            .max_size(2)
            .build(manager)
            .expect("Failed to create pool.")
    }

    fn generate_unique_name(prefix: &str) -> String {
        format!(
            "{}_{}",
            prefix,
            &Uuid::new_v4().to_string().replace("-", "")[..8]
        )
    }

    #[actix_web::test]
    async fn test_all_repository_methods() {
        use std::sync::atomic::{AtomicI64, Ordering};
        use std::time::SystemTime;
        static COUNTER: AtomicI64 = AtomicI64::new(0);

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as i64;

        let github_id_user: i64 = now
            .wrapping_add(COUNTER.fetch_add(1, Ordering::Relaxed))
            .abs()
            % 100000000;
        let github_id_org: i64 = now
            .wrapping_add(COUNTER.fetch_add(1, Ordering::Relaxed))
            .abs()
            % 100000000;
        let github_id_repo: i64 = now
            .wrapping_add(COUNTER.fetch_add(1, Ordering::Relaxed))
            .abs()
            % 100000000;
        let github_id_release: i64 = now
            .wrapping_add(COUNTER.fetch_add(1, Ordering::Relaxed))
            .abs()
            % 100000000;

        let pool = get_test_pool();
        let repo = PgRepository { pool: pool.clone() };

        // Test User Creation and Lookup
        let username = generate_unique_name("user");
        let email = format!("{}@example.com", username);
        let user = repo
            .create_user(
                Some(github_id_user),
                username.clone(),
                email.clone(),
                Some("hash".into()),
            )
            .await
            .expect("create_user failed");

        assert_eq!(user.username, username);
        assert_eq!(user.github_id, Some(github_id_user));

        // Test Find by Username
        let found_user = repo
            .find_user_by_username(username.clone())
            .await
            .expect("find_user_by_username failed")
            .expect("user not found");
        assert_eq!(found_user.id, user.id);

        // Test Find by ID
        let found_by_id = repo
            .find_user_by_id(user.id)
            .await
            .expect("find_user_by_id failed")
            .expect("user not found by id");
        assert_eq!(found_by_id.username, username);

        // Test Upsert User
        let upserted_user = repo
            .upsert_user(
                github_id_user,
                username.clone(),
                format!("new{}@example.com", username),
            )
            .await
            .expect("upsert_user failed");
        assert_eq!(upserted_user.id, user.id); // Should update same user

        // Test Create Organization
        let org_login = generate_unique_name("org");
        let org = repo
            .create_organization(Some(github_id_org), org_login.clone(), Some("desc".into()))
            .await
            .expect("create_organization failed");
        assert_eq!(org.login, org_login);

        // Test Get Organization
        let found_org = repo
            .get_organization(org.id)
            .await
            .expect("get_organization failed")
            .expect("org not found");
        assert_eq!(found_org.id, org.id);

        // Test Upsert Organization
        let upserted_org = repo
            .upsert_organization(github_id_org, org_login.clone(), Some("new_desc".into()))
            .await
            .expect("upsert_organization failed");
        assert_eq!(upserted_org.id, org.id);

        // Test Add User to Organization & Role
        repo.add_user_to_organization(org.id, user.id, "admin".into())
            .await
            .expect("add_user_to_organization failed");
        let role = repo
            .get_user_role(org.id, user.id)
            .await
            .expect("get_user_role failed")
            .expect("role not found");
        assert_eq!(role, "admin");

        // Test Create Repository
        let repo_name = generate_unique_name("repo");
        let repository = repo
            .create_repository(
                org.id,
                Some(github_id_repo),
                repo_name.clone(),
                Some("repo desc".into()),
            )
            .await
            .expect("create_repository failed");
        assert_eq!(repository.name, repo_name);

        // Test Get Repository
        let found_repo = repo
            .get_repository(repository.id)
            .await
            .expect("get_repository failed")
            .expect("repository not found");
        assert_eq!(found_repo.id, repository.id);

        // Test Upsert Repository
        let upserted_repo = repo
            .upsert_repository(
                org.id,
                github_id_repo,
                repo_name.clone(),
                Some("new repo desc".into()),
            )
            .await
            .expect("upsert_repository failed");
        assert_eq!(upserted_repo.id, repository.id);

        // Test Create Release
        let release = repo
            .create_release(
                repository.id,
                Some(github_id_release),
                "v1.0.0".into(),
                Some("Release 1".into()),
                Some("Body".into()),
            )
            .await
            .expect("create_release failed");
        assert_eq!(release.tag_name, "v1.0.0");

        // Test Upsert Release
        let upserted_release = repo
            .upsert_release(
                repository.id,
                github_id_release,
                "v1.0.0".into(),
                Some("Release 1 updated".into()),
                Some("New body".into()),
            )
            .await
            .expect("upsert_release failed");
        assert_eq!(upserted_release.id, release.id);

        // Test Upsert User Token
        repo.upsert_user_token(user.id, "github".into(), "enc_token".into())
            .await
            .expect("upsert_user_token failed");

        let tokens = repo
            .list_user_tokens(user.id)
            .await
            .expect("list_user_tokens failed");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].provider, "github");

        // Test Delete User Token
        repo.delete_user_token(user.id, "github".into())
            .await
            .expect("delete_user_token failed");
        let tokens_after = repo
            .list_user_tokens(user.id)
            .await
            .expect("list_user_tokens failed");
        assert!(tokens_after.is_empty());

        // Test Create Audit Log
        let log = repo
            .create_audit_log(
                org.id,
                Some(repository.id),
                user.id,
                "test_action".into(),
                Some(serde_json::json!({"key": "val"})),
            )
            .await
            .expect("create_audit_log failed");
        assert_eq!(log.action, "test_action");

        // Test List Audit Logs
        let logs = repo
            .list_audit_logs(org.id, 10, 0)
            .await
            .expect("list_audit_logs failed");
        assert!(!logs.is_empty());
        assert_eq!(logs[0].action, "test_action");

        // Also test missing entities
        let missing_user = repo
            .find_user_by_id(9999999)
            .await
            .expect("find_user_by_id failed on missing");
        assert!(missing_user.is_none());

        // Bad connection test
        // To get 100% coverage, we need to test connection errors.
        // But get_conn is hard to make fail unless pool is closed or exhausted, which is hard.
    }
}
