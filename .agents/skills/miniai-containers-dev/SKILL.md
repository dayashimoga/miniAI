---
name: miniai-containers-dev
description: >-
  Guides containerized development, Dockerfile authoring, Docker Compose orchestration, and VS Code
  Devcontainer setup for MiniAI. Activate when configuring reproducible build/test containers
  or isolated execution environments.
---

# MiniAI Containerized Development Skill

Use this skill when maintaining `Dockerfile`, `docker-compose.yml`, or `.devcontainer/devcontainer.json`.

## Container Best Practices
1. **Multi-Stage Builds**: Separate Rust/Python/Flutter build stages from minimal runtime images.
2. **Non-Root Execution**: Run container workloads as an unprivileged user (`miniai`) by default.
3. **Offline/Local Volume Isolation**: Mount model and dataset caches explicitly with read-only permissions where appropriate.

## Workflow Steps
1. Consult [`references/container_guidelines.md`](./references/container_guidelines.md).
2. Update `Dockerfile`, `docker-compose.yml`, and `.devcontainer/devcontainer.json`.
3. Validate syntax and configuration consistency via `scripts/verify_stage0.py`.

## Verification
- Confirm all container definitions specify pinned base images, non-root users, and health checks.

