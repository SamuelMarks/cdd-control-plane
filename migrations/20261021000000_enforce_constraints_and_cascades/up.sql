-- Add ON DELETE CASCADE and uniqueness constraints to core relational tables.

ALTER TABLE organization_users
    DROP CONSTRAINT IF EXISTS organization_users_organization_id_fkey,
    ADD CONSTRAINT organization_users_organization_id_fkey
        FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    DROP CONSTRAINT IF EXISTS organization_users_user_id_fkey,
    ADD CONSTRAINT organization_users_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE repositories
    DROP CONSTRAINT IF EXISTS repositories_organization_id_fkey,
    ADD CONSTRAINT repositories_organization_id_fkey
        FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE;

CREATE UNIQUE INDEX IF NOT EXISTS idx_repositories_org_name
    ON repositories (organization_id, name);

ALTER TABLE releases
    DROP CONSTRAINT IF EXISTS releases_repository_id_fkey,
    ADD CONSTRAINT releases_repository_id_fkey
        FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE;

CREATE UNIQUE INDEX IF NOT EXISTS idx_releases_repo_tag
    ON releases (repository_id, tag_name);
