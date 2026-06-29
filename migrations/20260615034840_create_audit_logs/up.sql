CREATE TABLE audit_logs (
    id SERIAL PRIMARY KEY,
    org_id INTEGER NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    repo_id INTEGER REFERENCES repositories(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action VARCHAR NOT NULL,
    metadata_json JSONB,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW()
);
