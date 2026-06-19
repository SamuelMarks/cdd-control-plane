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

## Platform Architecture

The `cdd` ecosystem is powered by a distributed microservice architecture:

| Repository                                                              | Role        | Description                                                                                        | CI Status                                                                                                                                                               |
| ----------------------------------------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`cdd-web-ui`](https://github.com/SamuelMarks/cdd-web-ui)               | Frontend    | The central control plane dashboard and UI for managing organizations, repositories, and releases. | [![CI](https://github.com/SamuelMarks/cdd-web-ui/actions/workflows/ci.yml/badge.svg)](https://github.com/SamuelMarks/cdd-web-ui/actions/workflows/ci.yml)               |
| [`cdd-control-plane`](https://github.com/SamuelMarks/cdd-control-plane) | Backend API | Manages Database, Auth, RBAC, organizations, and secrets.                                          | [![CI](https://github.com/SamuelMarks/cdd-control-plane/actions/workflows/ci.yml/badge.svg)](https://github.com/SamuelMarks/cdd-control-plane/actions/workflows/ci.yml) |
| [`cdd-engine`](https://github.com/SamuelMarks/cdd-engine)               | Generator   | Core code generation, WASI orchestration, and AST transformations.                                 | [![CI](https://github.com/SamuelMarks/cdd-engine/actions/workflows/ci.yml/badge.svg)](https://github.com/SamuelMarks/cdd-engine/actions/workflows/ci.yml)               |
| [`cdd-gateway`](https://github.com/SamuelMarks/cdd-gateway)             | Ingress     | API Gateway, reverse proxy, and routing.                                                           | [![CI](https://github.com/SamuelMarks/cdd-gateway/actions/workflows/ci.yml/badge.svg)](https://github.com/SamuelMarks/cdd-gateway/actions/workflows/ci.yml)             |
| [`cdd-publisher`](https://github.com/SamuelMarks/cdd-publisher)         | Worker      | Background worker for secure SDK releases to package registries.                                   | [![CI](https://github.com/SamuelMarks/cdd-publisher/actions/workflows/ci.yml/badge.svg)](https://github.com/SamuelMarks/cdd-publisher/actions/workflows/ci.yml)         |
| [`cdd-storage`](https://github.com/SamuelMarks/cdd-storage)             | Storage     | High-performance blob storage for JSON schemas and SDK zip artifacts.                              | [![CI](https://github.com/SamuelMarks/cdd-storage/actions/workflows/ci.yml/badge.svg)](https://github.com/SamuelMarks/cdd-storage/actions/workflows/ci.yml)             |
| [`cdd-docs-ui`](https://github.com/SamuelMarks/cdd-docs-ui)             | Frontend    | Dynamic API documentation viewer rendered for published endpoints.                                 | [![CI](https://github.com/SamuelMarks/cdd-docs-ui/actions/workflows/ci.yml/badge.svg)](https://github.com/SamuelMarks/cdd-docs-ui/actions/workflows/ci.yml)             |

## License
Dual-licensed under Apache 2.0 and MIT.
