//! Database models.


use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::db::schema::*;

/// User model representing an authenticated user or synced GitHub user.
#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
)]
#[diesel(table_name = users)]
/// User model.
pub struct User {
    /// Field id

    pub id: i32,
    /// Field github_id

    pub github_id: Option<i64>,
    /// Field username

    pub username: String,
    /// Field email

    pub email: String,
    #[serde(skip_serializing)]
    /// Field password_hash

    pub password_hash: Option<String>,
}

/// NewUser model.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = users)]
/// NewUser model.
pub struct NewUser<'a> {
    /// Field github_id

    pub github_id: Option<i64>,
    /// Field username

    pub username: &'a str,
    /// Field email

    pub email: &'a str,
    /// Field password_hash

    pub password_hash: Option<&'a str>,
}

/// UpdateUser model.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = users)]
/// UpdateUser model.
pub struct UpdateUser<'a> {
    /// Field github_id

    pub github_id: Option<i64>,
    /// Field username

    pub username: Option<&'a str>,
    /// Field email

    pub email: Option<&'a str>,
    /// Field password_hash

    pub password_hash: Option<&'a str>,
}

#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
)]
#[diesel(table_name = organizations)]
/// Organization model.
pub struct Organization {
    /// Field id

    pub id: i32,
    /// Field github_id

    pub github_id: Option<i64>,
    /// Field login

    pub login: String,
    /// Field description

    pub description: Option<String>,
}

/// NewOrganization model.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = organizations)]
/// NewOrganization model.
pub struct NewOrganization<'a> {
    /// Field github_id

    pub github_id: Option<i64>,
    /// Field login

    pub login: &'a str,
    /// Field description

    pub description: Option<&'a str>,
}

/// UpdateOrganization model.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = organizations)]
/// UpdateOrganization model.
pub struct UpdateOrganization<'a> {
    /// Field github_id

    pub github_id: Option<i64>,
    /// Field login

    pub login: Option<&'a str>,
    /// Field description

    pub description: Option<Option<&'a str>>,
}

#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
)]
#[diesel(table_name = organization_users)]
#[diesel(primary_key(organization_id, user_id))]
/// OrganizationUser model.
pub struct OrganizationUser {
    /// Field organization_id

    pub organization_id: i32,
    /// Field user_id

    pub user_id: i32,
    /// Field role

    pub role: String,
}

/// NewOrganizationUser model.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = organization_users)]
/// NewOrganizationUser model.
pub struct NewOrganizationUser<'a> {
    /// Field organization_id

    pub organization_id: i32,
    /// Field user_id

    pub user_id: i32,
    /// Field role

    pub role: &'a str,
}

/// UpdateOrganizationUser model.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = organization_users)]
/// UpdateOrganizationUser model.
pub struct UpdateOrganizationUser<'a> {
    /// Field role

    pub role: Option<&'a str>,
}

#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
)]
#[diesel(table_name = repositories)]
/// Repository model.
pub struct Repository {
    /// Field id

    pub id: i32,
    /// Field organization_id

    pub organization_id: i32,
    /// Field github_id

    pub github_id: Option<i64>,
    /// Field name

    pub name: String,
    /// Field description

    pub description: Option<String>,
}

/// NewRepository model.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = repositories)]
/// NewRepository model.
pub struct NewRepository<'a> {
    /// Field organization_id

    pub organization_id: i32,
    /// Field github_id

    pub github_id: Option<i64>,
    /// Field name

    pub name: &'a str,
    /// Field description

    pub description: Option<&'a str>,
}

/// UpdateRepository model.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = repositories)]
/// UpdateRepository model.
pub struct UpdateRepository<'a> {
    /// Field organization_id

    pub organization_id: Option<i32>,
    /// Field github_id

    pub github_id: Option<i64>,
    /// Field name

    pub name: Option<&'a str>,
    /// Field description

    pub description: Option<Option<&'a str>>,
}

#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
)]
#[diesel(table_name = releases)]
/// Release model.
pub struct Release {
    /// Field id

    pub id: i32,
    /// Field repository_id

    pub repository_id: i32,
    /// Field github_id

    pub github_id: Option<i64>,
    /// Field tag_name

    pub tag_name: String,
    /// Field name

    pub name: Option<String>,
    /// Field body

    pub body: Option<String>,
}

/// NewRelease model.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = releases)]
/// NewRelease model.
pub struct NewRelease<'a> {
    /// Field repository_id

    pub repository_id: i32,
    /// Field github_id

    pub github_id: Option<i64>,
    /// Field tag_name

    pub tag_name: &'a str,
    /// Field name

    pub name: Option<&'a str>,
    /// Field body

    pub body: Option<&'a str>,
}

/// UpdateRelease model.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = releases)]
/// UpdateRelease model.
pub struct UpdateRelease<'a> {
    /// Field repository_id

    pub repository_id: Option<i32>,
    /// Field github_id

    pub github_id: Option<i64>,
    /// Field tag_name

    pub tag_name: Option<&'a str>,
    /// Field name

    pub name: Option<Option<&'a str>>,
    /// Field body

    pub body: Option<Option<&'a str>>,
}

#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
)]
#[diesel(table_name = user_tokens)]
/// UserToken model.
pub struct UserToken {
    /// Field id

    pub id: i32,
    /// Field user_id

    pub user_id: i32,
    /// Field provider

    pub provider: String,
    /// Field encrypted_token

    pub encrypted_token: String,
    /// Field created_at

    pub created_at: chrono::NaiveDateTime,
    /// Field updated_at

    pub updated_at: chrono::NaiveDateTime,
}

/// NewUserToken model.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_tokens)]
/// NewUserToken model.
pub struct NewUserToken<'a> {
    /// Field user_id

    pub user_id: i32,
    /// Field provider

    pub provider: &'a str,
    /// Field encrypted_token

    pub encrypted_token: &'a str,
}

/// UpdateUserToken model.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = user_tokens)]
/// UpdateUserToken model.
pub struct UpdateUserToken<'a> {
    /// Field encrypted_token

    pub encrypted_token: Option<&'a str>,
}

#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    Serialize,
    Deserialize,
    ToSchema,
    PartialEq,
)]
#[diesel(table_name = audit_logs)]
/// AuditLog model.
pub struct AuditLog {
    /// Field id

    pub id: i32,
    /// Field org_id

    pub org_id: i32,
    /// Field repo_id

    pub repo_id: Option<i32>,
    /// Field user_id

    pub user_id: i32,
    /// Field action

    pub action: String,
    /// Field metadata_json

    pub metadata_json: Option<serde_json::Value>,
    /// Field timestamp

    pub timestamp: chrono::NaiveDateTime,
}

/// NewAuditLog model.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = audit_logs)]
/// NewAuditLog model.
pub struct NewAuditLog<'a> {
    /// Field org_id

    pub org_id: i32,
    /// Field repo_id

    pub repo_id: Option<i32>,
    /// Field user_id

    pub user_id: i32,
    /// Field action

    pub action: &'a str,
    /// Field metadata_json

    pub metadata_json: Option<serde_json::Value>,
}
