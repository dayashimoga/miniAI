"""Unit and property tests for local dataset curation, validation, and splitting."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from miniai_ml.dataset import (
    DatasetManifest,
    DatasetRecord,
    compute_file_sha256,
    deduplicate_records,
    load_dataset_file,
    save_splits_and_manifest,
    split_dataset,
    validate_record,
    verify_dataset_manifest,
)


def test_validate_record_valid_variants() -> None:
    # Standard format
    rec1 = validate_record({"id": "rec-01", "input_text": "Hello", "target_text": "World"})
    assert rec1.id == "rec-01"
    assert rec1.input_text == "Hello"
    assert rec1.target_text == "World"
    assert rec1.metadata == {}

    # Prompt / Response variant
    rec2 = validate_record({"prompt": "Tell me a joke", "response": "Why did the robot cross the road?"})
    assert rec2.id.startswith("rec-")
    assert rec2.input_text == "Tell me a joke"
    assert rec2.target_text == "Why did the robot cross the road?"

    # Instruction / Output variant with metadata
    rec3 = validate_record(
        {
            "instruction": "Translate to French",
            "output": "Bonjour",
            "metadata": {"domain": "translation"},
        }
    )
    assert rec3.input_text == "Translate to French"
    assert rec3.target_text == "Bonjour"
    assert rec3.metadata == {"domain": "translation"}


def test_validate_record_invalid_inputs() -> None:
    # Not a dict
    with pytest.raises(ValueError, match="Expected dict"):
        validate_record(["not", "a", "dict"])  # type: ignore

    # Missing input_text
    with pytest.raises(ValueError, match="'input_text' must be a non-empty string"):
        validate_record({"target_text": "Response only"})

    # Empty input_text
    with pytest.raises(ValueError, match="'input_text' must be a non-empty string"):
        validate_record({"input_text": "   ", "target_text": "Response"})

    # Missing target_text
    with pytest.raises(ValueError, match="'target_text' must be a non-empty string"):
        validate_record({"input_text": "Prompt only"})

    # Invalid metadata
    with pytest.raises(ValueError, match="'metadata' must be a dict"):
        validate_record({"input_text": "Prompt", "target_text": "Response", "metadata": "invalid"})


def test_dataset_record_roundtrip() -> None:
    rec = DatasetRecord(
        id="test-123",
        input_text="Question",
        target_text="Answer",
        metadata={"source": "faq"},
    )
    d = rec.to_dict()
    assert d["id"] == "test-123"
    assert d["input_text"] == "Question"
    assert d["target_text"] == "Answer"

    restored = DatasetRecord.from_dict(d)
    assert restored == rec


def test_load_dataset_file_jsonl(tmp_path: Path) -> None:
    jsonl_path = tmp_path / "data.jsonl"
    lines = [
        json.dumps({"input_text": "Q1", "target_text": "A1"}),
        json.dumps({"input_text": "Q2", "target_text": "A2"}),
        "",  # Empty line to verify blank lines are ignored
        json.dumps({"input_text": "Q3", "target_text": "A3"}),
    ]
    jsonl_path.write_text("\n".join(lines), encoding="utf-8")

    records = load_dataset_file(jsonl_path)
    assert len(records) == 3
    assert records[0].input_text == "Q1"
    assert records[2].target_text == "A3"


def test_load_dataset_file_csv(tmp_path: Path) -> None:
    csv_path = tmp_path / "data.csv"
    csv_content = "prompt,response\nWhat is AI?,Artificial Intelligence\nExplain ML,Machine Learning\n"
    csv_path.write_text(csv_content, encoding="utf-8")

    records = load_dataset_file(csv_path)
    assert len(records) == 2
    assert records[0].input_text == "What is AI?"
    assert records[1].target_text == "Machine Learning"


def test_load_dataset_file_errors(tmp_path: Path) -> None:
    # File not found
    with pytest.raises(FileNotFoundError):
        load_dataset_file(tmp_path / "nonexistent.jsonl")

    # Unsupported format
    txt_path = tmp_path / "data.txt"
    txt_path.write_text("Hello", encoding="utf-8")
    with pytest.raises(ValueError, match="Unsupported dataset format"):
        load_dataset_file(txt_path)

    # Empty file
    empty_path = tmp_path / "empty.jsonl"
    empty_path.write_text("", encoding="utf-8")
    with pytest.raises(ValueError, match="0 valid records"):
        load_dataset_file(empty_path)

    # Corrupt JSON line
    corrupt_path = tmp_path / "corrupt.jsonl"
    corrupt_path.write_text(
        '{"input_text": "Q1", "target_text": "A1"}\n{not valid json}\n', encoding="utf-8"
    )
    with pytest.raises(ValueError, match="Invalid JSON at line 2"):
        load_dataset_file(corrupt_path)

    # Empty / headerless CSV
    empty_csv = tmp_path / "empty.csv"
    empty_csv.write_text("", encoding="utf-8")
    with pytest.raises(ValueError, match="Empty or headerless CSV"):
        load_dataset_file(empty_csv)


def test_deduplicate_records() -> None:
    records = [
        DatasetRecord(id="1", input_text="Hello", target_text="World"),
        DatasetRecord(id="2", input_text="hello", target_text="world"),  # duplicate
        DatasetRecord(id="3", input_text="Unique", target_text="Answer"),
        DatasetRecord(id="4", input_text="Hello", target_text="World"),  # duplicate
    ]
    unique, dropped = deduplicate_records(records)
    assert len(unique) == 2
    assert dropped == 2
    assert unique[0].input_text == "Hello"
    assert unique[1].input_text == "Unique"


def test_split_dataset_deterministic() -> None:
    records = [
        DatasetRecord(id=str(i), input_text=f"Q{i}", target_text=f"A{i}")
        for i in range(100)
    ]

    # Split with seed 42
    splits1 = split_dataset(records, train_ratio=0.8, val_ratio=0.1, test_ratio=0.1, seed=42)
    assert len(splits1["train"]) == 80
    assert len(splits1["val"]) == 10
    assert len(splits1["test"]) == 10

    # Repeat with same seed gives identical order
    splits2 = split_dataset(records, train_ratio=0.8, val_ratio=0.1, test_ratio=0.1, seed=42)
    assert [r.id for r in splits1["train"]] == [r.id for r in splits2["train"]]

    # Different seed gives different order
    splits3 = split_dataset(records, train_ratio=0.8, val_ratio=0.1, test_ratio=0.1, seed=99)
    assert [r.id for r in splits1["train"]] != [r.id for r in splits3["train"]]

    # Single item with small train_ratio still guarantees train gets 1 record
    single_split = split_dataset([records[0]], train_ratio=0.1, val_ratio=0.45, test_ratio=0.45)
    assert len(single_split["train"]) == 1


def test_split_dataset_validation_errors() -> None:
    records = [DatasetRecord(id="1", input_text="Q", target_text="A")]

    # Empty list
    with pytest.raises(ValueError, match="empty record list"):
        split_dataset([])

    # Ratios not summing to 1.0
    with pytest.raises(ValueError, match="sum to 1.0"):
        split_dataset(records, train_ratio=0.7, val_ratio=0.1, test_ratio=0.1)

    # Negative ratio
    with pytest.raises(ValueError, match="non-negative"):
        split_dataset(records, train_ratio=1.2, val_ratio=-0.2, test_ratio=0.0)


def test_save_splits_and_manifest_and_verify(tmp_path: Path) -> None:
    splits = {
        "train": [
            DatasetRecord(id="t1", input_text="Train Q1", target_text="Train A1"),
            DatasetRecord(id="t2", input_text="Train Q2", target_text="Train A2"),
        ],
        "val": [
            DatasetRecord(id="v1", input_text="Val Q1", target_text="Val A1"),
        ],
        "test": [
            DatasetRecord(id="te1", input_text="Test Q1", target_text="Test A1"),
        ],
    }

    out_dir = tmp_path / "processed_dataset"
    manifest = save_splits_and_manifest(
        splits=splits,
        output_dir=out_dir,
        dataset_id="test-dataset-001",
        source_format="jsonl",
        duplicates_removed=5,
    )

    assert manifest.dataset_id == "test-dataset-001"
    assert manifest.total_records == 9  # 4 unique + 5 dropped
    assert manifest.unique_records == 4
    assert manifest.duplicates_removed == 5
    assert manifest.split_counts == {"train": 2, "val": 1, "test": 1}
    assert len(manifest.split_checksums) == 3
    assert len(manifest.overall_sha256) == 64

    # Verify manifest file exists on disk
    manifest_path = out_dir / "manifest.json"
    assert manifest_path.is_file()
    assert (out_dir / "train.jsonl").is_file()
    assert (out_dir / "val.jsonl").is_file()
    assert (out_dir / "test.jsonl").is_file()

    # Integrity verification
    assert verify_dataset_manifest(out_dir) is True

    # Tampering with a split file causes verification to fail
    with open(out_dir / "train.jsonl", "a", encoding="utf-8") as f:
        f.write('{"tampered": true}\n')

    with pytest.raises(ValueError, match="Checksum mismatch"):
        verify_dataset_manifest(out_dir)


def test_verify_manifest_missing_files(tmp_path: Path) -> None:
    # Missing manifest.json
    with pytest.raises(FileNotFoundError, match="Dataset manifest not found"):
        verify_dataset_manifest(tmp_path)

    # Missing split file
    manifest = DatasetManifest(
        dataset_id="d1",
        source_format="jsonl",
        total_records=1,
        unique_records=1,
        duplicates_removed=0,
        split_counts={"train": 1},
        split_checksums={"train": "abc123"},
        overall_sha256="hash",
        created_at_utc="now",
    )
    (tmp_path / "manifest.json").write_text(manifest.to_json(), encoding="utf-8")

    with pytest.raises(FileNotFoundError, match="Missing split file"):
        verify_dataset_manifest(tmp_path)
