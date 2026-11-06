"""Local dataset curation, schema validation, deduplication, and deterministic splitting.

Conforms to the requirements of Stage 4 (miniai-training-distillation and miniai-python-ml).
"""

from __future__ import annotations

import csv
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import random
from typing import Any


@dataclass(frozen=True)
class DatasetRecord:
    """A validated training/evaluation record."""

    id: str
    input_text: str
    target_text: str
    metadata: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        """Converts record to a dictionary."""
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> DatasetRecord:
        """Constructs and validates a record from dictionary."""
        return validate_record(data)


@dataclass(frozen=True)
class DatasetManifest:
    """Cryptographic provenance manifest for a curated and partitioned dataset."""

    dataset_id: str
    source_format: str
    total_records: int
    unique_records: int
    duplicates_removed: int
    split_counts: dict[str, int]
    split_checksums: dict[str, str]
    overall_sha256: str
    created_at_utc: str

    def to_dict(self) -> dict[str, Any]:
        """Converts manifest to a dictionary."""
        return asdict(self)

    def to_json(self, indent: int = 2) -> str:
        """Serializes manifest to JSON string."""
        return json.dumps(self.to_dict(), indent=indent)


def validate_record(record: dict[str, Any], record_idx: int | None = None) -> DatasetRecord:
    """Validates raw record dictionary ensuring required fields and non-empty content."""
    if not isinstance(record, dict):
        prefix = f"Record at index {record_idx}: " if record_idx is not None else ""
        raise ValueError(f"{prefix}Expected dict, got {type(record).__name__}")

    # Resolve input text across common schemas
    input_text = (
        record.get("input_text")
        or record.get("input")
        or record.get("prompt")
        or record.get("instruction")
    )
    if input_text is None or not isinstance(input_text, str) or not input_text.strip():
        prefix = f"Record {record_idx}: " if record_idx is not None else ""
        raise ValueError(f"{prefix}'input_text' must be a non-empty string")

    # Resolve target text across common schemas
    target_text = (
        record.get("target_text")
        or record.get("target")
        or record.get("response")
        or record.get("output")
        or record.get("completion")
    )
    if target_text is None or not isinstance(target_text, str) or not target_text.strip():
        prefix = f"Record {record_idx}: " if record_idx is not None else ""
        raise ValueError(f"{prefix}'target_text' must be a non-empty string")

    metadata = record.get("metadata", {})
    if not isinstance(metadata, dict):
        prefix = f"Record {record_idx}: " if record_idx is not None else ""
        raise ValueError(f"{prefix}'metadata' must be a dict")

    # Generate or sanitize ID
    raw_id = record.get("id")
    if raw_id and isinstance(raw_id, (str, int)):
        record_id = str(raw_id).strip()
    else:
        norm = f"{input_text.strip()}::{target_text.strip()}"
        record_id = f"rec-{hashlib.sha256(norm.encode('utf-8')).hexdigest()[:12]}"

    return DatasetRecord(
        id=record_id,
        input_text=input_text.strip(),
        target_text=target_text.strip(),
        metadata=metadata,
    )


def load_dataset_file(file_path: Path) -> list[DatasetRecord]:
    """Loads and validates a dataset from a .jsonl, .json, or .csv file."""
    path = Path(file_path)
    if not path.is_file():
        raise FileNotFoundError(f"Dataset file not found: {path}")

    suffix = path.suffix.lower()
    records: list[DatasetRecord] = []

    if suffix in {".jsonl", ".json"}:
        with open(path, "r", encoding="utf-8") as f:
            for idx, line in enumerate(f):
                clean = line.strip()
                if not clean:
                    continue
                try:
                    raw = json.loads(clean)
                except json.JSONDecodeError as e:
                    raise ValueError(f"Invalid JSON at line {idx + 1} in {path.name}: {e}") from e
                records.append(validate_record(raw, record_idx=idx))

    elif suffix == ".csv":
        with open(path, "r", encoding="utf-8", newline="") as f:
            reader = csv.DictReader(f)
            if not reader.fieldnames:
                raise ValueError(f"Empty or headerless CSV file: {path.name}")
            for idx, row in enumerate(reader):
                records.append(validate_record(row, record_idx=idx))

    else:
        raise ValueError(f"Unsupported dataset format '{suffix}'. Supported: .jsonl, .csv")

    if not records:
        raise ValueError(f"Dataset file contains 0 valid records: {path.name}")

    return records


def deduplicate_records(records: list[DatasetRecord]) -> tuple[list[DatasetRecord], int]:
    """Deduplicates records based on normalized input and target text."""
    seen: set[tuple[str, str]] = set()
    unique_records: list[DatasetRecord] = []
    duplicates_count = 0

    for rec in records:
        key = (rec.input_text.lower(), rec.target_text.lower())
        if key in seen:
            duplicates_count += 1
        else:
            seen.add(key)
            unique_records.append(rec)

    return unique_records, duplicates_count


def split_dataset(
    records: list[DatasetRecord],
    train_ratio: float = 0.8,
    val_ratio: float = 0.1,
    test_ratio: float = 0.1,
    seed: int = 42,
) -> dict[str, list[DatasetRecord]]:
    """Deterministically partitions records into train, val, and test splits."""
    if not records:
        raise ValueError("Cannot split an empty record list")

    total_ratio = train_ratio + val_ratio + test_ratio
    if abs(total_ratio - 1.0) > 1e-6:
        raise ValueError(f"Split ratios must sum to 1.0 (got {total_ratio:.4f})")

    if train_ratio <= 0.0 or val_ratio < 0.0 or test_ratio < 0.0:
        raise ValueError("Ratios must be non-negative, and train_ratio must be positive")

    shuffled = list(records)
    rng = random.Random(seed)
    rng.shuffle(shuffled)

    n_total = len(shuffled)
    n_train = int(n_total * train_ratio)
    n_val = int(n_total * val_ratio)

    # Ensure train gets at least 1 record if possible
    if n_train == 0 and n_total > 0:
        n_train = 1

    train_split = shuffled[:n_train]
    val_split = shuffled[n_train : n_train + n_val]
    test_split = shuffled[n_train + n_val :]

    return {
        "train": train_split,
        "val": val_split,
        "test": test_split,
    }


def compute_file_sha256(path: Path) -> str:
    """Computes SHA-256 digest of a file in streaming chunks."""
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()


def save_splits_and_manifest(
    splits: dict[str, list[DatasetRecord]],
    output_dir: Path,
    dataset_id: str,
    source_format: str = "jsonl",
    duplicates_removed: int = 0,
) -> DatasetManifest:
    """Saves splits as JSONL files and writes a cryptographically verified manifest.json."""
    out_path = Path(output_dir)
    out_path.mkdir(parents=True, exist_ok=True)

    split_counts: dict[str, int] = {}
    split_checksums: dict[str, str] = {}
    total_unique = 0

    for split_name in ("train", "val", "test"):
        records = splits.get(split_name, [])
        split_file = out_path / f"{split_name}.jsonl"

        with open(split_file, "w", encoding="utf-8") as f:
            for rec in records:
                f.write(json.dumps(rec.to_dict(), ensure_ascii=False) + "\n")

        digest = compute_file_sha256(split_file)
        split_counts[split_name] = len(records)
        split_checksums[split_name] = digest
        total_unique += len(records)

    # Compute overall dataset fingerprint (hash of sorted split hashes)
    overall_hasher = hashlib.sha256()
    for name in sorted(split_checksums.keys()):
        overall_hasher.update(f"{name}:{split_checksums[name]}".encode("utf-8"))
    overall_sha256 = overall_hasher.hexdigest()

    now_utc = datetime.now(timezone.utc).isoformat()
    manifest = DatasetManifest(
        dataset_id=dataset_id,
        source_format=source_format,
        total_records=total_unique + duplicates_removed,
        unique_records=total_unique,
        duplicates_removed=duplicates_removed,
        split_counts=split_counts,
        split_checksums=split_checksums,
        overall_sha256=overall_sha256,
        created_at_utc=now_utc,
    )

    manifest_file = out_path / "manifest.json"
    manifest_file.write_text(manifest.to_json(), encoding="utf-8")

    return manifest


def verify_dataset_manifest(dataset_dir: Path) -> bool:
    """Verifies that all split files in a dataset directory match manifest.json."""
    dir_path = Path(dataset_dir)
    manifest_path = dir_path / "manifest.json"
    if not manifest_path.is_file():
        raise FileNotFoundError(f"Dataset manifest not found: {manifest_path}")

    data = json.loads(manifest_path.read_text(encoding="utf-8"))
    split_checksums = data.get("split_checksums", {})

    for split_name, expected_sha256 in split_checksums.items():
        split_file = dir_path / f"{split_name}.jsonl"
        if not split_file.is_file():
            raise FileNotFoundError(f"Missing split file: {split_file}")
        actual_sha256 = compute_file_sha256(split_file)
        if actual_sha256 != expected_sha256:
            raise ValueError(
                f"Checksum mismatch for split '{split_name}': expected {expected_sha256}, got {actual_sha256}"
            )

    return True
