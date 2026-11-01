//! Repository, Polyglot Topology Provisioning, and SDK Release management API endpoints.

use crate::api::auth_middleware::AuthenticatedUser;
use crate::db::models::{Release, Repository, UpdateRepositoryConfig};
use crate::db::repository::CddRepository;
use crate::domain::{
    AuditAction, DocsPublishConfig, GitHubOrgName, LanguageGenerationOptions, Registry, RepoName,
    RepoTopology, RepoTopologyMode, TargetLanguage, Username,
};
use crate::error::Error;
use crate::queue::{JobQueue, ReleaseSdkJob};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;
use utoipa::ToSchema;

/// Request payload for creating a new [`Repository`] inside an organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreateRepoPayload {
    /// Repository name (`1..=64` alphanumeric, `-`, `_`; optional if `github_org_name` and `topology_mode` are provided).
    #[serde(default)]
    pub name: String,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// Optional upstream GitHub organization or user login (`${GITHUB_ORG_NAME}`).
    #[serde(default)]
    pub github_org_name: Option<String>,
    /// Optional topology mode (`monorepo` or `polyrepo`, defaults to `monorepo`).
    #[serde(default)]
    pub topology_mode: Option<RepoTopologyMode>,
    /// Optional custom monorepo name override (defaults to `${GITHUB_ORG_NAME}-sdk`).
    #[serde(default)]
    pub monorepo_name: Option<String>,
    /// Optional custom polyrepo names per language (defaults to `${GITHUB_ORG_NAME}-sdk-${LANGUAGE}`).
    #[serde(default)]
    pub polyrepo_names: Option<BTreeMap<TargetLanguage, String>>,
    /// Optional selected target languages (subset or all 13).
    #[serde(default)]
    pub target_languages: Option<Vec<TargetLanguage>>,
    /// Optional per-language generator options (`to_sdk`, `to_sdk_cli`, `to_server`, `framework`, `orm`, `cli`, `mcp`).
    #[serde(default)]
    pub language_options: Option<BTreeMap<TargetLanguage, LanguageGenerationOptions>>,
    /// Optional package registries for release publishing.
    #[serde(default)]
    pub release_registries: Option<Vec<Registry>>,
    /// Optional flag to enable PGP signing of release artifacts.
    #[serde(default)]
    pub enable_pgp_signing: Option<bool>,
    /// Optional documentation hosting configuration.
    #[serde(default)]
    pub docs_config: Option<DocsPublishConfig>,
}

/// Request payload for updating an existing [`Repository`]'s topology, languages, options, registries, PGP, and docs configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UpdateRepoConfigPayload {
    /// Optional updated repository name.
    pub name: Option<String>,
    /// Optional updated description.
    pub description: Option<String>,
    /// Optional updated `${GITHUB_ORG_NAME}`.
    pub github_org_name: Option<String>,
    /// Optional updated topology mode (`monorepo` or `polyrepo`).
    pub topology_mode: Option<RepoTopologyMode>,
    /// Optional updated custom monorepo name.
    pub monorepo_name: Option<String>,
    /// Optional updated per-language custom polyrepo names.
    pub polyrepo_names: Option<BTreeMap<TargetLanguage, String>>,
    /// Optional updated target languages.
    pub target_languages: Option<Vec<TargetLanguage>>,
    /// Optional updated per-language generator options.
    pub language_options: Option<BTreeMap<TargetLanguage, LanguageGenerationOptions>>,
    /// Optional updated release registries.
    pub release_registries: Option<Vec<Registry>>,
    /// Optional updated PGP signing toggle.
    pub enable_pgp_signing: Option<bool>,
    /// Optional updated documentation hosting configuration.
    pub docs_config: Option<DocsPublishConfig>,
}

/// Request payload for `POST /orgs/{org_id}/repos/provision` to provision either a Monorepo
/// (`${GITHUB_ORG_NAME}-sdk`) or Polyrepo (`${GITHUB_ORG_NAME}-sdk-${LANGUAGE}`) set of repositories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ProvisionProjectPayload {
    /// Upstream GitHub organization or user login (`${GITHUB_ORG_NAME}`).
    pub github_org_name: String,
    /// Repository topology (`monorepo` or `polyrepo`).
    pub topology_mode: RepoTopologyMode,
    /// Optional custom monorepo name override (defaults to `${GITHUB_ORG_NAME}-sdk`).
    #[serde(default)]
    pub monorepo_name: Option<String>,
    /// Optional per-language polyrepo name overrides (each defaults to `${GITHUB_ORG_NAME}-sdk-${LANGUAGE}`).
    #[serde(default)]
    pub polyrepo_names: BTreeMap<TargetLanguage, String>,
    /// Selected target languages (must contain at least 1 of the 13 supported languages).
    pub target_languages: Vec<TargetLanguage>,
    /// Per-language generator options (`to_sdk`, `to_sdk_cli`, `to_server`, `framework`, `orm`, `cli`, `mcp`).
    #[serde(default)]
    pub language_options: BTreeMap<TargetLanguage, LanguageGenerationOptions>,
    /// Target package registries (`crates_io`, `npm`, `pypi`, etc.; defaults to language defaults if empty).
    #[serde(default)]
    pub release_registries: Vec<Registry>,
    /// Whether PGP artifact signing is enabled.
    #[serde(default)]
    pub enable_pgp_signing: bool,
    /// Documentation hosting configuration (defaults to `OfficialCddSite`).
    #[serde(default)]
    pub docs_config: DocsPublishConfig,
    /// Optional repository description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Request payload for creating a new SDK [`Release`] and enqueuing a publish job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreateReleasePayload {
    /// Non-empty Git tag name (e.g., `v1.0.0`).
    pub tag_name: String,
    /// Optional release title.
    pub name: Option<String>,
    /// Optional release notes / changelog markdown body.
    pub body: Option<String>,
    /// Optional per-release override of target languages.
    #[serde(default)]
    pub target_languages: Option<Vec<TargetLanguage>>,
    /// Optional per-release override of target package registries.
    #[serde(default)]
    pub target_registries: Option<Vec<Registry>>,
    /// Optional per-release override for PGP artifact signing.
    #[serde(default)]
    pub enable_pgp_signing: Option<bool>,
    /// Optional per-release override for documentation hosting configuration.
    #[serde(default)]
    pub docs_config: Option<DocsPublishConfig>,
}

/// Validates and converts a map of per-language polyrepo name strings into validated [`RepoName`]s.
///
/// # Arguments
///
/// * `raw_map` - Map of [`TargetLanguage`] to raw repository name string.
///
/// # Returns
///
/// Validated map of [`TargetLanguage`] to [`RepoName`].
///
/// # Errors
///
/// Returns [`Error::BadRequest`] if any repository name violates GitHub naming rules.
fn validate_polyrepo_names(
    raw_map: &BTreeMap<TargetLanguage, String>,
) -> Result<BTreeMap<TargetLanguage, RepoName>, Error> {
    let mut validated = BTreeMap::new();
    for (&lang, raw_name) in raw_map {
        validated.insert(lang, RepoName::parse(raw_name)?);
    }
    Ok(validated)
}

/// Validates a map of per-language [`LanguageGenerationOptions`] against each language's allowed frameworks and ORMs.
///
/// # Arguments
///
/// * `options_map` - Map of [`TargetLanguage`] to [`LanguageGenerationOptions`].
///
/// # Returns
///
/// `Ok(())` if all language options are valid.
///
/// # Errors
///
/// Returns [`Error::BadRequest`] if any language option has an invalid framework or ORM.
fn validate_language_options(
    options_map: &BTreeMap<TargetLanguage, LanguageGenerationOptions>,
) -> Result<(), Error> {
    for (&lang, opts) in options_map {
        opts.validate_for(lang)?;
    }
    Ok(())
}

/// Builds and validates an [`UpdateRepositoryConfig`] changeset from an [`UpdateRepoConfigPayload`].
///
/// # Arguments
///
/// * `payload` - Incoming [`UpdateRepoConfigPayload`].
///
/// # Returns
///
/// Validated [`UpdateRepositoryConfig`] changeset.
///
/// # Errors
///
/// Returns [`Error::BadRequest`] if any field fails domain validation.
fn build_update_repository_config(
    payload: &UpdateRepoConfigPayload,
) -> Result<UpdateRepositoryConfig, Error> {
    let mut update = UpdateRepositoryConfig::default();
    if let Some(raw_name) = payload.name.as_deref() {
        update.name = Some(RepoName::parse(raw_name)?.as_str().to_string());
    }
    if let Some(desc) = payload.description.clone() {
        update.description = Some(desc);
    }
    if let Some(raw_gh_org) = payload.github_org_name.as_deref() {
        update.github_org_name = Some(GitHubOrgName::parse(raw_gh_org)?.as_str().to_string());
    }
    if let Some(mode) = payload.topology_mode {
        update.topology_mode = Some(mode.as_str().to_string());
    }
    if let Some(raw_mono) = payload.monorepo_name.as_deref() {
        update.monorepo_name = Some(RepoName::parse(raw_mono)?.as_str().to_string());
    }
    if let Some(poly_map) = payload.polyrepo_names.as_ref() {
        let validated = validate_polyrepo_names(poly_map)?;
        update.polyrepo_names_json = Some(serde_json::to_value(validated)?);
    }
    if let Some(langs) = payload.target_languages.as_ref() {
        update.target_languages_json = Some(serde_json::to_value(langs)?);
    }
    if let Some(opts) = payload.language_options.as_ref() {
        validate_language_options(opts)?;
        update.language_options_json = Some(serde_json::to_value(opts)?);
    }
    if let Some(regs) = payload.release_registries.as_ref() {
        update.release_registries_json = Some(serde_json::to_value(regs)?);
    }
    if let Some(pgp) = payload.enable_pgp_signing {
        update.enable_pgp_signing = Some(pgp);
    }
    if let Some(docs) = payload.docs_config.as_ref() {
        docs.validate()?;
        update.docs_config_json = Some(serde_json::to_value(docs)?);
    }
    Ok(update)
}

/// Creates a new [`Repository`] inside `org_id` if the caller holds `owner` or `admin` privileges.
///
/// Supports both basic repository creation and full topology/language/registry/PGP/docs configuration.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Organization primary key ID.
/// * `payload` - JSON [`CreateRepoPayload`].
/// * `repo` - Database repository handle.
///
/// # Returns
///
/// `201 Created` with the persisted [`Repository`].
///
/// # Errors
///
/// Returns [`Error::BadRequest`] on validation failure or [`Error::Forbidden`] if the caller lacks write access.
#[utoipa::path(
    post,
    path = "/orgs/{org_id}/repos",
    params(
        ("org_id" = i32, Path, description = "Organization ID")
    ),
    request_body = CreateRepoPayload,
    responses(
        (status = 201, description = "Repository created", body = Repository),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn create_repo(
    user: AuthenticatedUser,
    path: web::Path<i32>,
    payload: web::Json<CreateRepoPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let org_id = path.into_inner();

    let resolved_name = if payload.name.trim().is_empty() {
        if let Some(raw_gh_org) = payload.github_org_name.as_deref() {
            let gh_org = GitHubOrgName::parse(raw_gh_org)?;
            let custom_mono = payload
                .monorepo_name
                .as_deref()
                .map(RepoName::parse)
                .transpose()?;
            let topology = RepoTopology::Monorepo {
                repo_name: custom_mono,
            };
            topology.resolve_monorepo_name(&gh_org).as_str().to_string()
        } else {
            return Err(Error::BadRequest("Repository name cannot be empty".into()));
        }
    } else {
        Username::parse(&payload.name)?.as_str().to_string()
    };

    let has_extended_config = payload.github_org_name.is_some()
        || payload.topology_mode.is_some()
        || payload.monorepo_name.is_some()
        || payload.polyrepo_names.is_some()
        || payload.target_languages.is_some()
        || payload.language_options.is_some()
        || payload.release_registries.is_some()
        || payload.enable_pgp_signing.is_some()
        || payload.docs_config.is_some();

    let update_cfg = build_update_repository_config(&UpdateRepoConfigPayload {
        name: None,
        description: None,
        github_org_name: payload.github_org_name.clone(),
        topology_mode: payload.topology_mode,
        monorepo_name: payload.monorepo_name.clone(),
        polyrepo_names: payload.polyrepo_names.clone(),
        target_languages: payload.target_languages.clone(),
        language_options: payload.language_options.clone(),
        release_registries: payload.release_registries.clone(),
        enable_pgp_signing: payload.enable_pgp_signing,
        docs_config: payload.docs_config.clone(),
    })?;

    let Some(role) = repo.get_user_role(org_id, user.user_id).await? else {
        return Err(Error::Forbidden(
            "User is not a member of this organization".into(),
        ));
    };
    if !role.can_write_repos() {
        return Err(Error::Forbidden(
            "Insufficient permissions to create a repository".into(),
        ));
    }

    let mut repository = repo
        .create_repository(org_id, None, resolved_name, payload.description.clone())
        .await?;

    if has_extended_config {
        repository = repo
            .update_repository_config(repository.id, update_cfg)
            .await?;
    }

    repo.create_audit_log(
        org_id,
        Some(repository.id),
        user.user_id,
        AuditAction::RepoCreated,
        Some(serde_json::json!({ "name": repository.name })),
    )
    .await?;

    Ok(HttpResponse::Created().json(repository))
}

/// Updates an existing [`Repository`]'s topology, target languages, generator options,
/// registries, PGP signing, and documentation configuration.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Tuple of `(org_id, repo_id)`.
/// * `payload` - JSON [`UpdateRepoConfigPayload`].
/// * `repo` - Database repository handle.
///
/// # Returns
///
/// `200 OK` with updated [`Repository`].
///
/// # Errors
///
/// Returns [`Error::BadRequest`] on invalid options, [`Error::Forbidden`] if unauthorized, or [`Error::NotFound`] if missing.
#[utoipa::path(
    patch,
    path = "/orgs/{org_id}/repos/{repo_id}",
    params(
        ("org_id" = i32, Path, description = "Organization ID"),
        ("repo_id" = i32, Path, description = "Repository ID")
    ),
    request_body = UpdateRepoConfigPayload,
    responses(
        (status = 200, description = "Repository configuration updated", body = Repository),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found")
    )
)]
pub async fn update_repo_config(
    user: AuthenticatedUser,
    path: web::Path<(i32, i32)>,
    payload: web::Json<UpdateRepoConfigPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let (org_id, repo_id) = path.into_inner();
    let update = build_update_repository_config(&payload)?;

    let Some(role) = repo.get_user_role(org_id, user.user_id).await? else {
        return Err(Error::Forbidden(
            "User is not a member of this organization".into(),
        ));
    };
    if !role.can_write_repos() {
        return Err(Error::Forbidden(
            "Insufficient permissions to update repository configuration".into(),
        ));
    }

    let Some(existing) = repo.get_repository(repo_id).await? else {
        return Err(Error::NotFound("Repository not found".into()));
    };
    if existing.organization_id != org_id {
        return Err(Error::NotFound("Repository not found".into()));
    }

    let updated = repo.update_repository_config(repo_id, update).await?;

    repo.create_audit_log(
        org_id,
        Some(repo_id),
        user.user_id,
        AuditAction::RepoConfigUpdated,
        Some(serde_json::json!({ "repo_id": repo_id, "name": updated.name })),
    )
    .await?;

    Ok(HttpResponse::Ok().json(updated))
}

/// Resolves the package registries for a provisioned repository target, falling back to
/// language defaults when no explicit registries were specified in the request.
///
/// # Arguments
///
/// * `explicit` - Explicitly requested [`Registry`] slice.
/// * `languages` - Target [`TargetLanguage`] slice for the repository.
///
/// # Returns
///
/// Resolved vector of [`Registry`] items.
fn resolve_target_registries(explicit: &[Registry], languages: &[TargetLanguage]) -> Vec<Registry> {
    if !explicit.is_empty() {
        return explicit.to_vec();
    }
    let mut default_regs = Vec::new();
    for &lang in languages {
        for &reg in Registry::default_for_language(lang) {
            if !default_regs.contains(&reg) {
                default_regs.push(reg);
            }
        }
    }
    default_regs
}

/// Provisions either a single Monorepo (`${GITHUB_ORG_NAME}-sdk` or custom) or individual
/// per-language Polyrepos (`${GITHUB_ORG_NAME}-sdk-${LANGUAGE}` or custom) for the selected target languages.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Organization primary key ID.
/// * `payload` - JSON [`ProvisionProjectPayload`].
/// * `repo` - Database repository handle.
///
/// # Returns
///
/// `201 Created` with a JSON array of provisioned [`Repository`] records.
///
/// # Errors
///
/// Returns [`Error::BadRequest`] on invalid topology/languages/options or [`Error::Forbidden`] if unauthorized.
#[utoipa::path(
    post,
    path = "/orgs/{org_id}/repos/provision",
    params(
        ("org_id" = i32, Path, description = "Organization ID")
    ),
    request_body = ProvisionProjectPayload,
    responses(
        (status = 201, description = "Project repositories provisioned", body = [Repository]),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn provision_project_repos(
    user: AuthenticatedUser,
    path: web::Path<i32>,
    payload: web::Json<ProvisionProjectPayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let org_id = path.into_inner();
    if payload.target_languages.is_empty() {
        return Err(Error::BadRequest(
            "At least one target language must be selected".into(),
        ));
    }

    let gh_org = GitHubOrgName::parse(&payload.github_org_name)?;
    validate_language_options(&payload.language_options)?;
    payload.docs_config.validate()?;

    let validated_poly = validate_polyrepo_names(&payload.polyrepo_names)?;
    let custom_mono = payload
        .monorepo_name
        .as_deref()
        .map(RepoName::parse)
        .transpose()?;

    let topology = match payload.topology_mode {
        RepoTopologyMode::Monorepo => RepoTopology::Monorepo {
            repo_name: custom_mono.clone(),
        },
        RepoTopologyMode::Polyrepo => RepoTopology::Polyrepo {
            repo_names: validated_poly.clone(),
        },
    };

    let Some(role) = repo.get_user_role(org_id, user.user_id).await? else {
        return Err(Error::Forbidden(
            "User is not a member of this organization".into(),
        ));
    };
    if !role.can_write_repos() {
        return Err(Error::Forbidden(
            "Insufficient permissions to provision repositories".into(),
        ));
    }

    let targets = topology.expand_targets(&gh_org, &payload.target_languages);
    let mut created_repos = Vec::with_capacity(targets.len());

    for target in targets {
        let mut lang_opts_for_target = BTreeMap::new();
        for &lang in &target.languages {
            let opts = payload
                .language_options
                .get(&lang)
                .cloned()
                .unwrap_or_default();
            lang_opts_for_target.insert(lang, opts);
        }

        let registries_for_target =
            resolve_target_registries(&payload.release_registries, &target.languages);

        let base_repo = repo
            .create_repository(
                org_id,
                None,
                target.repo_name.as_str().to_string(),
                payload.description.clone(),
            )
            .await?;

        let monorepo_name_opt = custom_mono.as_ref().map(|m| m.as_str().to_string());

        let configured_repo = repo
            .update_repository_config(
                base_repo.id,
                UpdateRepositoryConfig {
                    name: None,
                    description: None,
                    github_org_name: Some(gh_org.as_str().to_string()),
                    topology_mode: Some(target.topology_mode.as_str().to_string()),
                    monorepo_name: monorepo_name_opt,
                    polyrepo_names_json: Some(serde_json::to_value(&validated_poly)?),
                    target_languages_json: Some(serde_json::to_value(&target.languages)?),
                    language_options_json: Some(serde_json::to_value(&lang_opts_for_target)?),
                    release_registries_json: Some(serde_json::to_value(&registries_for_target)?),
                    enable_pgp_signing: Some(payload.enable_pgp_signing),
                    docs_config_json: Some(serde_json::to_value(&payload.docs_config)?),
                },
            )
            .await?;

        created_repos.push(configured_repo);
    }

    let first_repo_id = Some(created_repos[0].id);

    repo.create_audit_log(
        org_id,
        first_repo_id,
        user.user_id,
        AuditAction::ProjectProvisioned,
        Some(serde_json::json!({
            "github_org_name": gh_org.as_str(),
            "topology_mode": payload.topology_mode.as_str(),
            "count": created_repos.len(),
        })),
    )
    .await?;

    Ok(HttpResponse::Created().json(created_repos))
}

/// Lists all repositories in `org_id` if the caller is a member of the organization.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Organization primary key ID.
/// * `repo` - Database repository handle.
///
/// # Returns
///
/// `200 OK` with a JSON array of [`Repository`] records.
///
/// # Errors
///
/// Returns [`Error::Forbidden`] if the caller is not a member of `org_id`.
#[utoipa::path(
    get,
    path = "/orgs/{org_id}/repos",
    params(
        ("org_id" = i32, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "List of repositories", body = [Repository]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn list_repos(
    user: AuthenticatedUser,
    path: web::Path<i32>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let org_id = path.into_inner();
    let Some(role) = repo.get_user_role(org_id, user.user_id).await? else {
        return Err(Error::Forbidden(
            "User is not a member of this organization".into(),
        ));
    };
    debug_assert!(role.can_read());

    let repos = repo.list_repositories_for_org(org_id).await?;
    Ok(HttpResponse::Ok().json(repos))
}

/// Retrieves a single [`Repository`] by `(org_id, repo_id)` if the caller is an organization member.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Tuple of `(org_id, repo_id)`.
/// * `repo` - Database repository handle.
///
/// # Returns
///
/// `200 OK` with [`Repository`].
///
/// # Errors
///
/// Returns [`Error::Forbidden`] if not an organization member or [`Error::NotFound`] if the repository does not exist.
#[utoipa::path(
    get,
    path = "/orgs/{org_id}/repos/{repo_id}",
    params(
        ("org_id" = i32, Path, description = "Organization ID"),
        ("repo_id" = i32, Path, description = "Repository ID")
    ),
    responses(
        (status = 200, description = "Repository details", body = Repository),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found")
    )
)]
pub async fn get_repo(
    user: AuthenticatedUser,
    path: web::Path<(i32, i32)>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let (org_id, repo_id) = path.into_inner();
    let Some(role) = repo.get_user_role(org_id, user.user_id).await? else {
        return Err(Error::Forbidden(
            "User is not a member of this organization".into(),
        ));
    };
    debug_assert!(role.can_read());

    match repo.get_repository(repo_id).await? {
        Some(repository) if repository.organization_id == org_id => {
            Ok(HttpResponse::Ok().json(repository))
        }
        _ => Err(Error::NotFound("Repository not found".into())),
    }
}

/// Creates a new SDK [`Release`], records an audit log, and enqueues an enriched [`ReleaseSdkJob`]
/// if a [`JobQueue`] is configured.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Tuple of `(org_id, repo_id)`.
/// * `payload` - JSON [`CreateReleasePayload`].
/// * `repo` - Database repository handle.
/// * `queue` - Optional background job queue handle.
///
/// # Returns
///
/// `201 Created` with [`Release`].
///
/// # Errors
///
/// Returns [`Error::BadRequest`] on invalid tag or docs config, [`Error::Forbidden`] if unauthorized, or [`Error::NotFound`].
#[utoipa::path(
    post,
    path = "/orgs/{org_id}/repos/{repo_id}/releases",
    params(
        ("org_id" = i32, Path, description = "Organization ID"),
        ("repo_id" = i32, Path, description = "Repository ID")
    ),
    request_body = CreateReleasePayload,
    responses(
        (status = 201, description = "Release created and queued", body = Release),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found")
    )
)]
pub async fn create_release(
    user: AuthenticatedUser,
    path: web::Path<(i32, i32)>,
    payload: web::Json<CreateReleasePayload>,
    repo: web::Data<Arc<dyn CddRepository>>,
    queue: Option<web::Data<Arc<dyn JobQueue>>>,
) -> Result<HttpResponse, Error> {
    let (org_id, repo_id) = path.into_inner();
    let tag_trimmed = payload.tag_name.trim();
    if tag_trimmed.is_empty() {
        return Err(Error::BadRequest("Release tag_name cannot be empty".into()));
    }
    if let Some(docs) = payload.docs_config.as_ref() {
        docs.validate()?;
    }

    let Some(role) = repo.get_user_role(org_id, user.user_id).await? else {
        return Err(Error::Forbidden(
            "User is not a member of this organization".into(),
        ));
    };
    if !role.can_write_repos() {
        return Err(Error::Forbidden(
            "Insufficient permissions to create a release".into(),
        ));
    }

    let Some(repository) = repo.get_repository(repo_id).await? else {
        return Err(Error::NotFound("Repository not found".into()));
    };
    if repository.organization_id != org_id {
        return Err(Error::NotFound("Repository not found".into()));
    }

    let release = repo
        .create_release(
            repo_id,
            None,
            tag_trimmed.to_string(),
            payload.name.clone(),
            payload.body.clone(),
        )
        .await?;

    repo.create_audit_log(
        org_id,
        Some(repo_id),
        user.user_id,
        AuditAction::ReleaseCreated,
        Some(serde_json::json!({ "tag_name": release.tag_name })),
    )
    .await?;

    if let Some(q) = queue {
        let topology_mode = RepoTopologyMode::from_str(&repository.topology_mode)
            .unwrap_or(RepoTopologyMode::Monorepo);

        let target_languages: Vec<TargetLanguage> =
            if let Some(override_langs) = payload.target_languages.clone() {
                override_langs
            } else {
                serde_json::from_value(repository.target_languages_json.clone()).unwrap_or_default()
            };

        let target_registries: Vec<Registry> = if let Some(override_regs) =
            payload.target_registries.clone()
        {
            override_regs
        } else {
            serde_json::from_value(repository.release_registries_json.clone()).unwrap_or_default()
        };

        let enable_pgp_signing = payload
            .enable_pgp_signing
            .unwrap_or(repository.enable_pgp_signing);

        let docs_config: DocsPublishConfig =
            if let Some(override_docs) = payload.docs_config.clone() {
                override_docs
            } else {
                serde_json::from_value(repository.docs_config_json.clone()).unwrap_or_default()
            };

        let job = ReleaseSdkJob {
            release_id: release.id,
            org_id,
            repo_id,
            tag_name: release.tag_name.clone(),
            github_org_name: repository.github_org_name.clone(),
            repo_name: repository.name.clone(),
            topology_mode,
            target_languages,
            target_registries,
            enable_pgp_signing,
            docs_config,
        };
        q.enqueue_release_sdk(&job).await?;
    }

    Ok(HttpResponse::Created().json(release))
}

/// Lists all SDK releases for a repository.
///
/// # Arguments
///
/// * `user` - Authenticated caller identity.
/// * `path` - Tuple of `(org_id, repo_id)`.
/// * `repo` - Database repository handle.
///
/// # Returns
///
/// `200 OK` with a JSON array of [`Release`] records.
///
/// # Errors
///
/// Returns [`Error::Forbidden`] if unauthorized or [`Error::NotFound`] if the repository does not exist.
#[utoipa::path(
    get,
    path = "/orgs/{org_id}/repos/{repo_id}/releases",
    params(
        ("org_id" = i32, Path, description = "Organization ID"),
        ("repo_id" = i32, Path, description = "Repository ID")
    ),
    responses(
        (status = 200, description = "List of releases", body = [Release]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found")
    )
)]
pub async fn list_releases(
    user: AuthenticatedUser,
    path: web::Path<(i32, i32)>,
    repo: web::Data<Arc<dyn CddRepository>>,
) -> Result<HttpResponse, Error> {
    let (org_id, repo_id) = path.into_inner();
    let Some(role) = repo.get_user_role(org_id, user.user_id).await? else {
        return Err(Error::Forbidden(
            "User is not a member of this organization".into(),
        ));
    };
    debug_assert!(role.can_read());

    let Some(repository) = repo.get_repository(repo_id).await? else {
        return Err(Error::NotFound("Repository not found".into()));
    };
    if repository.organization_id != org_id {
        return Err(Error::NotFound("Repository not found".into()));
    }

    let releases = repo.list_releases_for_repo(repo_id).await?;
    Ok(HttpResponse::Ok().json(releases))
}

/// Configures the `/orgs/{org_id}/repos` routes on the provided [`web::ServiceConfig`].
///
/// # Arguments
///
/// * `cfg` - Actix service configuration builder.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/orgs/{org_id}/repos")
            .route("", web::post().to(create_repo))
            .route("", web::get().to(list_repos))
            .route("/provision", web::post().to(provision_project_repos))
            .route("/{repo_id}", web::get().to(get_repo))
            .route("/{repo_id}", web::patch().to(update_repo_config))
            .route("/{repo_id}/releases", web::post().to(create_release))
            .route("/{repo_id}/releases", web::get().to(list_releases)),
    );
}

#[cfg(test)]
mod tests {
    //! Unit tests for repository, topology provisioning, configuration updates, and SDK release endpoints.

    use super::*;
    use crate::api::auth_middleware::{generate_test_token, test_app_config};
    use crate::db::models::AuditLog;
    use crate::db::repository::MockCddRepository;
    use crate::domain::{DocHostTarget, GeneratorTarget, OrgRole};
    use crate::queue::MockJobQueue;
    use actix_web::{body::to_bytes, http::StatusCode, test, App};

    fn sample_repo(id: i32, org_id: i32, name: &str) -> Repository {
        Repository {
            id,
            organization_id: org_id,
            github_id: None,
            name: name.to_string(),
            description: None,
            github_org_name: "AcmeOrg".to_string(),
            topology_mode: "monorepo".to_string(),
            monorepo_name: None,
            polyrepo_names_json: serde_json::json!({}),
            target_languages_json: serde_json::json!(["rust", "python"]),
            language_options_json: serde_json::json!({}),
            release_registries_json: serde_json::json!(["crates_io", "pypi"]),
            enable_pgp_signing: true,
            docs_config_json: serde_json::json!({"enabled": true, "target": "official"}),
        }
    }

    #[actix_web::test]
    async fn test_create_and_list_repos_rbac() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();

        // Org 1: non-member -> 403
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 1)
            .times(2)
            .returning(|_, _| Ok(None));

        // Org 2: member (read-only) -> 403 on POST, 200 on GET
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 2)
            .times(2)
            .returning(|_, _| Ok(Some(OrgRole::Member)));
        mock_repo
            .expect_list_repositories_for_org()
            .withf(|org_id| *org_id == 2)
            .times(1)
            .returning(|_| Ok(vec![sample_repo(5, 2, "existing_repo")]));

        // Org 3: admin -> 201 on POST + audit log (both basic and extended config)
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 3)
            .times(2)
            .returning(|_, _| Ok(Some(OrgRole::Admin)));
        mock_repo
            .expect_create_repository()
            .withf(|org_id, gh_id, name, _| *org_id == 3 && gh_id.is_none() && name == "sdk_repo")
            .times(1)
            .returning(|org_id, _, name, _| Ok(sample_repo(9, org_id, &name)));

        mock_repo
            .expect_create_repository()
            .withf(|org_id, _, name, _| *org_id == 3 && name == "SamuelMarks-sdk")
            .times(1)
            .returning(|org_id, _, name, _| Ok(sample_repo(10, org_id, &name)));
        mock_repo
            .expect_update_repository_config()
            .withf(|rid, _| *rid == 10)
            .times(1)
            .returning(|rid, _| Ok(sample_repo(rid, 3, "SamuelMarks-sdk")));

        mock_repo
            .expect_create_audit_log()
            .withf(|org_id, _, uid, action, _| {
                *org_id == 3 && *uid == 1 && *action == AuditAction::RepoCreated
            })
            .times(2)
            .returning(|org_id, repo_id, user_id, action, metadata_json| {
                Ok(AuditLog {
                    id: 1,
                    org_id,
                    repo_id,
                    user_id,
                    action: action.as_str().to_string(),
                    metadata_json,
                    timestamp: chrono::Utc::now().naive_utc(),
                })
            });

        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let token = generate_test_token()?;

        // Empty name without github_org_name -> 400
        let req_empty_name = test::TestRequest::post()
            .uri("/orgs/3/repos")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateRepoPayload {
                name: String::new(),
                description: None,
                github_org_name: None,
                topology_mode: None,
                monorepo_name: None,
                polyrepo_names: None,
                target_languages: None,
                language_options: None,
                release_registries: None,
                enable_pgp_signing: None,
                docs_config: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_empty_name).await.status(),
            StatusCode::BAD_REQUEST
        );

        let req_non_member = test::TestRequest::post()
            .uri("/orgs/1/repos")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateRepoPayload {
                name: "sdk_repo".into(),
                description: None,
                github_org_name: None,
                topology_mode: None,
                monorepo_name: None,
                polyrepo_names: None,
                target_languages: None,
                language_options: None,
                release_registries: None,
                enable_pgp_signing: None,
                docs_config: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_non_member).await.status(),
            StatusCode::FORBIDDEN
        );

        let req_list_non_member = test::TestRequest::get()
            .uri("/orgs/1/repos")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_list_non_member).await.status(),
            StatusCode::FORBIDDEN
        );

        let req_member_post = test::TestRequest::post()
            .uri("/orgs/2/repos")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateRepoPayload {
                name: "sdk_repo".into(),
                description: None,
                github_org_name: None,
                topology_mode: None,
                monorepo_name: None,
                polyrepo_names: None,
                target_languages: None,
                language_options: None,
                release_registries: None,
                enable_pgp_signing: None,
                docs_config: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_member_post).await.status(),
            StatusCode::FORBIDDEN
        );

        let req_member_list = test::TestRequest::get()
            .uri("/orgs/2/repos")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_list = test::call_service(&app, req_member_list).await;
        assert_eq!(resp_list.status(), StatusCode::OK);
        let list_bytes = to_bytes(resp_list.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let repos: Vec<Repository> = serde_json::from_slice(&list_bytes)?;
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "existing_repo");

        let req_admin_post = test::TestRequest::post()
            .uri("/orgs/3/repos")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateRepoPayload {
                name: "sdk_repo".into(),
                description: Some("SDK".into()),
                github_org_name: None,
                topology_mode: None,
                monorepo_name: None,
                polyrepo_names: None,
                target_languages: None,
                language_options: None,
                release_registries: None,
                enable_pgp_signing: None,
                docs_config: None,
            })
            .to_request();
        let resp_created = test::call_service(&app, req_admin_post).await;
        assert_eq!(resp_created.status(), StatusCode::CREATED);
        let created_bytes = to_bytes(resp_created.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let created_repo: Repository = serde_json::from_slice(&created_bytes)?;
        assert_eq!(created_repo.id, 9);
        assert_eq!(created_repo.name, "sdk_repo");

        // Create repo with default ${GITHUB_ORG_NAME}-sdk and full extended configuration
        let mut lang_opts = BTreeMap::new();
        lang_opts.insert(
            TargetLanguage::Ts,
            LanguageGenerationOptions {
                target: GeneratorTarget::ToSdk,
                cli: true,
                mcp: true,
                framework: Some("angular".into()),
                orm: Some("prisma".into()),
                ..LanguageGenerationOptions::default()
            },
        );
        let mut poly_names = BTreeMap::new();
        poly_names.insert(TargetLanguage::Ts, "SamuelMarks-sdk-ts".into());

        let req_ext_post = test::TestRequest::post()
            .uri("/orgs/3/repos")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateRepoPayload {
                name: String::new(),
                description: Some("Monorepo SDK".into()),
                github_org_name: Some("SamuelMarks".into()),
                topology_mode: Some(RepoTopologyMode::Monorepo),
                monorepo_name: Some("SamuelMarks-sdk".into()),
                polyrepo_names: Some(poly_names),
                target_languages: Some(vec![TargetLanguage::Ts]),
                language_options: Some(lang_opts),
                release_registries: Some(vec![Registry::Npm]),
                enable_pgp_signing: Some(true),
                docs_config: Some(DocsPublishConfig::default()),
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_ext_post).await.status(),
            StatusCode::CREATED
        );

        Ok(())
    }

    #[actix_web::test]
    async fn test_update_repo_config_and_provision_project_repos() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();

        // PATCH /orgs/1/repos/10 -> non-member 403
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 1)
            .times(2)
            .returning(|_, _| Ok(None));

        // PATCH & provision on org 2 -> member (read-only) 403
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 2)
            .times(2)
            .returning(|_, _| Ok(Some(OrgRole::Member)));

        // Org 3 -> owner: PATCH 404 (missing), PATCH 404 (wrong org), PATCH 200 OK
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 3)
            .times(5)
            .returning(|_, _| Ok(Some(OrgRole::Owner)));

        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 404)
            .times(1)
            .returning(|_| Ok(None));
        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 405)
            .times(1)
            .returning(|_| Ok(Some(sample_repo(405, 999, "wrong_org"))));
        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 10)
            .times(1)
            .returning(|_| Ok(Some(sample_repo(10, 3, "AcmeOrg-sdk"))));

        mock_repo
            .expect_update_repository_config()
            .withf(|rid, _| *rid == 10)
            .times(1)
            .returning(|rid, _| Ok(sample_repo(rid, 3, "AcmeOrg-custom-sdk")));

        mock_repo
            .expect_create_audit_log()
            .withf(|org_id, repo_id, _, action, _| {
                *org_id == 3 && *repo_id == Some(10) && *action == AuditAction::RepoConfigUpdated
            })
            .times(1)
            .returning(|org_id, repo_id, user_id, action, metadata_json| {
                Ok(AuditLog {
                    id: 1,
                    org_id,
                    repo_id,
                    user_id,
                    action: action.as_str().to_string(),
                    metadata_json,
                    timestamp: chrono::Utc::now().naive_utc(),
                })
            });

        // Provision Monorepo (1 repo: AcmeOrg-sdk) and Polyrepo (2 repos: AcmeOrg-sdk-rust, acme-custom-py)
        mock_repo
            .expect_create_repository()
            .times(3)
            .returning(|org_id, _, name, _| Ok(sample_repo(50, org_id, &name)));
        mock_repo
            .expect_update_repository_config()
            .withf(|rid, _| *rid == 50)
            .times(3)
            .returning(|rid, upd| {
                let mut r = sample_repo(rid, 3, "provisioned");
                if let Some(m) = upd.topology_mode {
                    r.topology_mode = m;
                }
                Ok(r)
            });
        mock_repo
            .expect_create_audit_log()
            .withf(|org_id, _, _, action, _| {
                *org_id == 3 && *action == AuditAction::ProjectProvisioned
            })
            .times(2)
            .returning(|org_id, repo_id, user_id, action, metadata_json| {
                Ok(AuditLog {
                    id: 2,
                    org_id,
                    repo_id,
                    user_id,
                    action: action.as_str().to_string(),
                    metadata_json,
                    timestamp: chrono::Utc::now().naive_utc(),
                })
            });

        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let token = generate_test_token()?;

        // PATCH non-member & member -> 403
        let req_p1 = test::TestRequest::patch()
            .uri("/orgs/1/repos/10")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(UpdateRepoConfigPayload::default())
            .to_request();
        assert_eq!(
            test::call_service(&app, req_p1).await.status(),
            StatusCode::FORBIDDEN
        );

        let req_p2 = test::TestRequest::patch()
            .uri("/orgs/2/repos/10")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(UpdateRepoConfigPayload::default())
            .to_request();
        assert_eq!(
            test::call_service(&app, req_p2).await.status(),
            StatusCode::FORBIDDEN
        );

        // PATCH 404 missing and wrong org
        let req_p404 = test::TestRequest::patch()
            .uri("/orgs/3/repos/404")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(UpdateRepoConfigPayload::default())
            .to_request();
        assert_eq!(
            test::call_service(&app, req_p404).await.status(),
            StatusCode::NOT_FOUND
        );

        let req_p405 = test::TestRequest::patch()
            .uri("/orgs/3/repos/405")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(UpdateRepoConfigPayload::default())
            .to_request();
        assert_eq!(
            test::call_service(&app, req_p405).await.status(),
            StatusCode::NOT_FOUND
        );

        // PATCH 200 OK with full fields
        let mut poly_map = BTreeMap::new();
        poly_map.insert(TargetLanguage::Rust, "AcmeOrg-sdk-rust".into());
        let mut opts_map = BTreeMap::new();
        opts_map.insert(
            TargetLanguage::Rust,
            LanguageGenerationOptions {
                target: GeneratorTarget::ToServer,
                framework: Some("actix".into()),
                orm: Some("diesel".into()),
                cli: true,
                mcp: true,
                ..LanguageGenerationOptions::default()
            },
        );
        let req_patch_ok = test::TestRequest::patch()
            .uri("/orgs/3/repos/10")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(UpdateRepoConfigPayload {
                name: Some("AcmeOrg-custom-sdk".into()),
                description: Some("Updated".into()),
                github_org_name: Some("AcmeOrg".into()),
                topology_mode: Some(RepoTopologyMode::Monorepo),
                monorepo_name: Some("AcmeOrg-custom-sdk".into()),
                polyrepo_names: Some(poly_map),
                target_languages: Some(vec![TargetLanguage::Rust]),
                language_options: Some(opts_map),
                release_registries: Some(vec![Registry::CratesIo]),
                enable_pgp_signing: Some(true),
                docs_config: Some(DocsPublishConfig {
                    enabled: true,
                    target: DocHostTarget::GithubPages,
                    custom_url: None,
                    github_pages_branch: Some("gh-pages".into()),
                }),
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_patch_ok).await.status(),
            StatusCode::OK
        );

        // Provision validation failure: empty target_languages -> 400
        let req_prov_empty = test::TestRequest::post()
            .uri("/orgs/3/repos/provision")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(ProvisionProjectPayload {
                github_org_name: "AcmeOrg".into(),
                topology_mode: RepoTopologyMode::Monorepo,
                monorepo_name: None,
                polyrepo_names: BTreeMap::new(),
                target_languages: vec![],
                language_options: BTreeMap::new(),
                release_registries: vec![],
                enable_pgp_signing: false,
                docs_config: DocsPublishConfig::default(),
                description: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_prov_empty).await.status(),
            StatusCode::BAD_REQUEST
        );

        // Provision RBAC: non-member & member -> 403
        let valid_prov_payload = ProvisionProjectPayload {
            github_org_name: "AcmeOrg".into(),
            topology_mode: RepoTopologyMode::Monorepo,
            monorepo_name: Some("AcmeOrg-custom-mono".into()),
            polyrepo_names: BTreeMap::new(),
            target_languages: vec![TargetLanguage::Rust, TargetLanguage::Python],
            language_options: BTreeMap::new(),
            release_registries: vec![],
            enable_pgp_signing: true,
            docs_config: DocsPublishConfig::default(),
            description: Some("Provisioned".into()),
        };

        let req_prov_nm = test::TestRequest::post()
            .uri("/orgs/1/repos/provision")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(valid_prov_payload.clone())
            .to_request();
        assert_eq!(
            test::call_service(&app, req_prov_nm).await.status(),
            StatusCode::FORBIDDEN
        );

        let req_prov_mem = test::TestRequest::post()
            .uri("/orgs/2/repos/provision")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(valid_prov_payload.clone())
            .to_request();
        assert_eq!(
            test::call_service(&app, req_prov_mem).await.status(),
            StatusCode::FORBIDDEN
        );

        // Provision Monorepo -> 201 Created (1 repository)
        let req_prov_mono = test::TestRequest::post()
            .uri("/orgs/3/repos/provision")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(valid_prov_payload)
            .to_request();
        let resp_mono = test::call_service(&app, req_prov_mono).await;
        assert_eq!(resp_mono.status(), StatusCode::CREATED);
        let mono_bytes = to_bytes(resp_mono.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let mono_repos: Vec<Repository> = serde_json::from_slice(&mono_bytes)?;
        assert_eq!(mono_repos.len(), 1);

        // Provision Polyrepo -> 201 Created (2 repositories)
        let mut custom_poly = BTreeMap::new();
        custom_poly.insert(TargetLanguage::Python, "acme-custom-py".into());
        let req_prov_poly = test::TestRequest::post()
            .uri("/orgs/3/repos/provision")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(ProvisionProjectPayload {
                github_org_name: "AcmeOrg".into(),
                topology_mode: RepoTopologyMode::Polyrepo,
                monorepo_name: None,
                polyrepo_names: custom_poly,
                target_languages: vec![TargetLanguage::Rust, TargetLanguage::Python],
                language_options: BTreeMap::new(),
                release_registries: vec![Registry::CratesIo, Registry::PyPi],
                enable_pgp_signing: true,
                docs_config: DocsPublishConfig::default(),
                description: None,
            })
            .to_request();
        let resp_poly = test::call_service(&app, req_prov_poly).await;
        assert_eq!(resp_poly.status(), StatusCode::CREATED);
        let poly_bytes = to_bytes(resp_poly.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let poly_repos: Vec<Repository> = serde_json::from_slice(&poly_bytes)?;
        assert_eq!(poly_repos.len(), 2);

        Ok(())
    }

    #[actix_web::test]
    async fn test_get_repo_branches() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 1)
            .times(1)
            .returning(|_, _| Ok(None));

        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 2)
            .times(3)
            .returning(|_, _| Ok(Some(OrgRole::Member)));

        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 10)
            .times(1)
            .returning(|_| Ok(Some(sample_repo(10, 2, "valid_repo"))));

        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 11)
            .times(1)
            .returning(|_| Ok(Some(sample_repo(11, 99, "other_org_repo"))));

        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 12)
            .times(1)
            .returning(|_| Ok(None));

        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .configure(configure),
        )
        .await;

        let token = generate_test_token()?;

        let req_unauth = test::TestRequest::get()
            .uri("/orgs/1/repos/10")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_unauth).await.status(),
            StatusCode::FORBIDDEN
        );

        let req_ok = test::TestRequest::get()
            .uri("/orgs/2/repos/10")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_ok = test::call_service(&app, req_ok).await;
        assert_eq!(resp_ok.status(), StatusCode::OK);
        let body = to_bytes(resp_ok.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let repo_item: Repository = serde_json::from_slice(&body)?;
        assert_eq!(repo_item.name, "valid_repo");

        let req_mismatch = test::TestRequest::get()
            .uri("/orgs/2/repos/11")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_mismatch).await.status(),
            StatusCode::NOT_FOUND
        );

        let req_nf = test::TestRequest::get()
            .uri("/orgs/2/repos/12")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_nf).await.status(),
            StatusCode::NOT_FOUND
        );

        Ok(())
    }

    #[actix_web::test]
    async fn test_create_and_list_releases_with_job_queue() -> Result<(), Error> {
        let mut mock_repo = MockCddRepository::new();
        let mut mock_queue = MockJobQueue::new();

        // Org 1: non-member -> 403
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 1)
            .times(2)
            .returning(|_, _| Ok(None));

        // Org 2: member -> 403 on POST release, 200 on GET releases
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 2)
            .times(4)
            .returning(|_, _| Ok(Some(OrgRole::Member)));

        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 50)
            .times(1)
            .returning(|_| Ok(Some(sample_repo(50, 2, "r50"))));
        mock_repo
            .expect_list_releases_for_repo()
            .withf(|rid| *rid == 50)
            .times(1)
            .returning(|_| {
                Ok(vec![Release {
                    id: 100,
                    repository_id: 50,
                    github_id: None,
                    tag_name: "v1.0.0".into(),
                    name: Some("Initial".into()),
                    body: None,
                }])
            });

        // Repo mismatch & missing on list_releases
        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 51)
            .times(1)
            .returning(|_| Ok(Some(sample_repo(51, 999, "r51"))));
        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 52)
            .times(1)
            .returning(|_| Ok(None));

        // Org 3: owner -> 201 on POST release + queue job (default + override)
        mock_repo
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 3)
            .times(4)
            .returning(|_, _| Ok(Some(OrgRole::Owner)));

        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 60)
            .times(2)
            .returning(|_| Ok(Some(sample_repo(60, 3, "r60"))));
        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 61)
            .times(1)
            .returning(|_| Ok(Some(sample_repo(61, 999, "r61"))));
        mock_repo
            .expect_get_repository()
            .withf(|rid| *rid == 62)
            .times(1)
            .returning(|_| Ok(None));

        mock_repo
            .expect_create_release()
            .withf(|rid, _, tag, _, _| *rid == 60 && (tag == "v2.0.0" || tag == "v2.1.0"))
            .times(2)
            .returning(|repository_id, github_id, tag_name, name, body| {
                Ok(Release {
                    id: 200,
                    repository_id,
                    github_id,
                    tag_name,
                    name,
                    body,
                })
            });

        mock_repo
            .expect_create_audit_log()
            .withf(|org_id, repo_id, _, action, _| {
                *org_id == 3 && *repo_id == Some(60) && *action == AuditAction::ReleaseCreated
            })
            .times(2)
            .returning(|org_id, repo_id, user_id, action, metadata_json| {
                Ok(AuditLog {
                    id: 1,
                    org_id,
                    repo_id,
                    user_id,
                    action: action.as_str().to_string(),
                    metadata_json,
                    timestamp: chrono::Utc::now().naive_utc(),
                })
            });

        mock_queue
            .expect_enqueue_release_sdk()
            .withf(|job| {
                job.release_id == 200
                    && job.org_id == 3
                    && job.repo_id == 60
                    && job.github_org_name == "AcmeOrg"
                    && job.repo_name == "r60"
                    && job.enable_pgp_signing
            })
            .times(2)
            .returning(|_| Ok(()));

        let app = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(Arc::new(mock_repo) as Arc<dyn CddRepository>))
                .app_data(web::Data::new(Arc::new(mock_queue) as Arc<dyn JobQueue>))
                .configure(configure),
        )
        .await;

        let token = generate_test_token()?;

        // Empty tag_name -> 400 Bad Request
        let req_empty_tag = test::TestRequest::post()
            .uri("/orgs/3/repos/60/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "   ".into(),
                name: None,
                body: None,
                target_languages: None,
                target_registries: None,
                enable_pgp_signing: None,
                docs_config: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_empty_tag).await.status(),
            StatusCode::BAD_REQUEST
        );

        // Non-member POST & GET -> 403
        let req_nm_post = test::TestRequest::post()
            .uri("/orgs/1/repos/60/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "v1.0.0".into(),
                name: None,
                body: None,
                target_languages: None,
                target_registries: None,
                enable_pgp_signing: None,
                docs_config: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_nm_post).await.status(),
            StatusCode::FORBIDDEN
        );

        let req_nm_get = test::TestRequest::get()
            .uri("/orgs/1/repos/60/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_nm_get).await.status(),
            StatusCode::FORBIDDEN
        );

        // Read-only member POST -> 403
        let req_mem_post = test::TestRequest::post()
            .uri("/orgs/2/repos/50/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "v1.0.0".into(),
                name: None,
                body: None,
                target_languages: None,
                target_registries: None,
                enable_pgp_signing: None,
                docs_config: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_mem_post).await.status(),
            StatusCode::FORBIDDEN
        );

        // Member GET releases -> 200 OK
        let req_mem_get = test::TestRequest::get()
            .uri("/orgs/2/repos/50/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp_mem_get = test::call_service(&app, req_mem_get).await;
        assert_eq!(resp_mem_get.status(), StatusCode::OK);
        let rel_bytes = to_bytes(resp_mem_get.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let releases: Vec<Release> = serde_json::from_slice(&rel_bytes)?;
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].tag_name, "v1.0.0");

        // Member GET releases mismatch & missing -> 404
        let req_get_mismatch = test::TestRequest::get()
            .uri("/orgs/2/repos/51/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_get_mismatch).await.status(),
            StatusCode::NOT_FOUND
        );
        let req_get_missing = test::TestRequest::get()
            .uri("/orgs/2/repos/52/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        assert_eq!(
            test::call_service(&app, req_get_missing).await.status(),
            StatusCode::NOT_FOUND
        );

        // Owner POST release mismatch & missing -> 404
        let req_post_mismatch = test::TestRequest::post()
            .uri("/orgs/3/repos/61/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "v2.0.0".into(),
                name: None,
                body: None,
                target_languages: None,
                target_registries: None,
                enable_pgp_signing: None,
                docs_config: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_post_mismatch).await.status(),
            StatusCode::NOT_FOUND
        );
        let req_post_missing = test::TestRequest::post()
            .uri("/orgs/3/repos/62/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "v2.0.0".into(),
                name: None,
                body: None,
                target_languages: None,
                target_registries: None,
                enable_pgp_signing: None,
                docs_config: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_post_missing).await.status(),
            StatusCode::NOT_FOUND
        );

        // Owner POST release valid (using repository defaults) -> 201 Created + queued
        let req_owner_post = test::TestRequest::post()
            .uri("/orgs/3/repos/60/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "v2.0.0".into(),
                name: Some("Major".into()),
                body: Some("Notes".into()),
                target_languages: None,
                target_registries: None,
                enable_pgp_signing: None,
                docs_config: None,
            })
            .to_request();
        let resp_owner = test::call_service(&app, req_owner_post).await;
        assert_eq!(resp_owner.status(), StatusCode::CREATED);
        let created_bytes = to_bytes(resp_owner.into_body())
            .await
            .map_err(|_| Error::InternalError)?;
        let created_rel: Release = serde_json::from_slice(&created_bytes)?;
        assert_eq!(created_rel.id, 200);
        assert_eq!(created_rel.tag_name, "v2.0.0");

        // Owner POST release with per-release overrides -> 201 Created + queued
        let req_override_post = test::TestRequest::post()
            .uri("/orgs/3/repos/60/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "v2.1.0".into(),
                name: Some("Minor".into()),
                body: None,
                target_languages: Some(vec![TargetLanguage::Rust]),
                target_registries: Some(vec![Registry::CratesIo]),
                enable_pgp_signing: Some(true),
                docs_config: Some(DocsPublishConfig::default()),
            })
            .to_request();
        assert_eq!(
            test::call_service(&app, req_override_post).await.status(),
            StatusCode::CREATED
        );

        // Also test create_release when JobQueue app_data is not registered (queue == None)
        let mut mock_repo_no_q = MockCddRepository::new();
        mock_repo_no_q
            .expect_get_user_role()
            .withf(|org_id, _| *org_id == 3)
            .times(1)
            .returning(|_, _| Ok(Some(OrgRole::Owner)));
        mock_repo_no_q
            .expect_get_repository()
            .withf(|rid| *rid == 60)
            .times(1)
            .returning(|_| Ok(Some(sample_repo(60, 3, "r60"))));
        mock_repo_no_q.expect_create_release().times(1).returning(
            |repository_id, github_id, tag_name, name, body| {
                Ok(Release {
                    id: 201,
                    repository_id,
                    github_id,
                    tag_name,
                    name,
                    body,
                })
            },
        );
        mock_repo_no_q.expect_create_audit_log().times(1).returning(
            |org_id, repo_id, user_id, action, metadata_json| {
                Ok(AuditLog {
                    id: 2,
                    org_id,
                    repo_id,
                    user_id,
                    action: action.as_str().to_string(),
                    metadata_json,
                    timestamp: chrono::Utc::now().naive_utc(),
                })
            },
        );
        let app_no_q = test::init_service(
            App::new()
                .app_data(test_app_config()?)
                .app_data(web::Data::new(
                    Arc::new(mock_repo_no_q) as Arc<dyn CddRepository>
                ))
                .configure(configure),
        )
        .await;
        let req_no_q = test::TestRequest::post()
            .uri("/orgs/3/repos/60/releases")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(CreateReleasePayload {
                tag_name: "v3.0.0".into(),
                name: None,
                body: None,
                target_languages: None,
                target_registries: None,
                enable_pgp_signing: None,
                docs_config: None,
            })
            .to_request();
        assert_eq!(
            test::call_service(&app_no_q, req_no_q).await.status(),
            StatusCode::CREATED
        );

        Ok(())
    }
}
