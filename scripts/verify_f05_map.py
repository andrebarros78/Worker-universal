from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
concept = (ROOT / "docs" / "F05_DURABLE_RUNTIME_MAP.md").read_text(encoding="utf-8")
matrix = (ROOT / "docs" / "F05_IMPLEMENTATION_MATRIX.md").read_text(encoding="utf-8")

expected = {f"DUR-{index:03d}" for index in range(1, 46)}
concept_ids = set(re.findall(r"DUR-\d{3}", concept))
matrix_ids: set[str] = set()

for match in re.finditer(r"DUR-(\d{3})(?:\.\.(\d{3}))?", matrix):
    start = int(match.group(1))
    end = int(match.group(2) or match.group(1))
    matrix_ids.update(f"DUR-{index:03d}" for index in range(start, end + 1))

if concept_ids != expected or matrix_ids != expected:
    raise SystemExit(
        "F05_MAP_FAIL "
        f"concept_missing={sorted(expected-concept_ids)} "
        f"matrix_missing={sorted(expected-matrix_ids)} "
        f"concept_extra={sorted(concept_ids-expected)} "
        f"matrix_extra={sorted(matrix_ids-expected)}"
    )

print("F05_MAP_OK requirements=45 mapped=45")
