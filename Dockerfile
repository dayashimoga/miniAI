# syntax=docker/dockerfile:1
FROM rust:1.85-slim-bookworm AS rust-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    git \
    pkg-config \
    python3 \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for secure execution
RUN useradd --create-home --uid 10001 --shell /bin/bash miniai
WORKDIR /workspace
RUN chown miniai:miniai /workspace

USER miniai
# Install uv for isolated Python/ML environment management
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/home/miniai/.local/bin:${PATH}"

COPY --chown=miniai:miniai Cargo.toml ./
COPY --chown=miniai:miniai crates ./crates
COPY --chown=miniai:miniai ml ./ml
COPY --chown=miniai:miniai scripts ./scripts
COPY --chown=miniai:miniai .agents ./.agents
COPY --chown=miniai:miniai docs ./docs
COPY --chown=miniai:miniai *.md ./

RUN cargo test --workspace
RUN cd ml && uv run pytest --cov=miniai_ml --cov-fail-under=90

CMD ["cargo", "test", "--workspace"]

