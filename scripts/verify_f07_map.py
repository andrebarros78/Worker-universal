from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
concept = (ROOT / "docs" / "F07_VALIDATION_RECOVERY_MAP.md").read_text(encoding="utf-8")
matrix = (ROOT / "docs" / "F07_IMPLEMENTATION_MATRIX.md").read_text(encoding="utf-8")

expected = {f"VAL-{index:03d}" for index in range(1, 56)}
concept_ids = set(re.findall(r"VAL-\d{3}", concept))
matrix_ids: set[str] = set()

for match in re.finditer(r"VAL-(\d{3})(?:\.\.(\d{3}))?", matrix):
    start = int(match.group(1))
    end = int(match.group(2) or match.group(1))
    matrix_ids.update(f"VAL-{index:03d}" for index in range(start, end + 1))

if concept_ids != expected or matrix_ids != expected:
    raise SystemExit(
        "F07_MAP_FAIL "
        f"concept_missing={sorted(expected-concept_ids)} "
        f"matrix_missing={sorted(expected-matrix_ids)} "
        f"concept_extra={sorted(concept_ids-expected)} "
        f"matrix_extra={sorted(matrix_ids-expected)}"
    )

print("F07_MAP_OK requirements=55 mapped=55")
