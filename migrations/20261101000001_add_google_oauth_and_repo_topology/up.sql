-- Add Google OAuth, avatar URL, GitHub org name, and repository topology/generator/registry/docs columns.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS google_id VARCHAR UNIQUE NULL,
    ADD COLUMN IF NOT EXISTS avatar_url VARCHAR NULL;

ALTER TABLE organizations
    ADD COLUMN IF NOT EXISTS github_org_name VARCHAR NULL;

ALTER TABLE repositories
    ADD COLUMN IF NOT EXISTS github_org_name VARCHAR NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS topology_mode VARCHAR NOT NULL DEFAULT 'monorepo',
    ADD COLUMN IF NOT EXISTS monorepo_name VARCHAR NULL,
    ADD COLUMN IF NOT EXISTS polyrepo_names_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS target_languages_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS language_options_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS release_registries_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS enable_pgp_signing BOOL NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS docs_config_json JSONB NOT NULL DEFAULT '{"enabled":true,"target":"official"}'::jsonb;
