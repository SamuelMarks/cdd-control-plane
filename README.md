# cdd-control-plane

> The central API control plane, database interface, and multi-tenant management service for the `cdd` ecosystem.

[![CI](https://github.com/SamuelMarks/cdd-control-plane/actions/workflows/ci.yml/badge.svg)](https://github.com/SamuelMarks/cdd-control-plane/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg)](https://opensource.org/licenses/Apache-2.0)

`cdd-control-plane` is the source-of-truth backend for managing users, organizations, repositories, and releases. Extracted from the monolithic `cdd-ctl`, this repository handles Role-Based Access Control (RBAC), database persistence (`diesel`), and authentication, enabling multi-tenant workflows and dashboard telemetry for the `cdd-web-ui`.

## Features
- **Identity & RBAC:** Multi-tenant organization and repository management.
- **Authentication:** Validates JWTs, handles OAuth integrations (e.g., GitHub), and manages user sessions.
- **State Management:** PostgreSQL backed schema managing `Orgs`, `Users`, `Repos`, and `AuditLogs`.
- **Secret Vault:** Securely stores registry and webhook credentials using Libsodium.

## License
Dual-licensed under Apache 2.0 and MIT.
