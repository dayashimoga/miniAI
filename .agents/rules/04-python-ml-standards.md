# Rule 04: Python & PyTorch ML Tooling Standards

## Scope
Applies to `ml/` and `scripts/`.

## Mandates
1. **Environment Management**:
   - Use `uv` (`pyproject.toml`) for reproducible virtual environments and dependency resolution.
   - Avoid polluting the host global Python installation.
2. **CPU-First & Deterministic Execution**:
   - Never assume CUDA or discrete GPU availability (`torch.cuda.is_available()` must always fall back cleanly to `cpu`).
   - Seed random number generators (`random`, `numpy`, `torch`) explicitly for reproducible training, distillation, and evaluation runs.
3. **Type Safety & Documentation**:
   - Annotate public functions, dataclasses, and return types using standard Python type hints.
4. **Verification**:
   - Every commit touching Python code must pass `uv run pytest` with >= 90% coverage on core modules (`ml/src/miniai_ml`).

