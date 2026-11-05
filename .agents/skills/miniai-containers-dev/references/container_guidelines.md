# Container Security & Reproducibility Guidelines

1. **Base Image Pinning**: Use explicit tags (e.g., `rust:1.85-slim-bookworm`, `python:3.12-slim-bookworm`).
2. **Least Privilege**: Create a dedicated `miniai` user (`UID 10001`) and switch with `USER miniai`.
3. **No Embedded Secrets**: Never pass secrets via `ENV` or `ARG` in `Dockerfile`.

