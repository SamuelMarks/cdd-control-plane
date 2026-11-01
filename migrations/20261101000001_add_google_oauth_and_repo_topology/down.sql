-- Revert Google OAuth, avatar URL, GitHub org name, and repository topology/generator/registry/docs columns.

ALTER TABLE repositories
    DROP COLUMN IF EXISTS docs_config_json,
    DROP COLUMN IF EXISTS enable_pgp_signing,
    DROP COLUMN IF EXISTS release_registries_json,
    DROP COLUMN IF EXISTS language_options_json,
    DROP COLUMN IF EXISTS target_languages_json,
    DROP COLUMN IF EXISTS polyrepo_names_json,
    DROP COLUMN IF EXISTS monorepo_name,
    DROP COLUMN IF EXISTS topology_mode,
    DROP COLUMN IF EXISTS github_org_name;

ALTER TABLE organizations
    DROP COLUMN IF EXISTS github_org_name;

ALTER TABLE users
    DROP COLUMN IF EXISTS avatar_url,
    DROP COLUMN IF EXISTS google_id;
