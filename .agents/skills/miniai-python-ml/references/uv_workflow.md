# Reproducible Python ML Workflow with `uv`

## Running Tests & Coverage
```powershell
Push-Location g:\miniAI\ml
uv run pytest --cov=miniai_ml --cov-report=term-missing --cov-fail-under=90
Pop-Location
```

## Deterministic Seeding Pattern
Always initialize deterministic seeds before dataset splitting, training, or evaluation:
```python
import random

def seed_everything(seed: int = 42) -> int:
    random.seed(seed)
    return seed
```

