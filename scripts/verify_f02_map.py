from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
concept = (ROOT / "docs" / "F02_CAPABILITY_MAP.md").read_text(encoding="utf-8")
matrix = (ROOT / "docs" / "F02_IMPLEMENTATION_MATRIX.md").read_text(encoding="utf-8")

expected = {f"CAP-{index:03d}" for index in range(1, 46)}
concept_ids = set(re.findall(r"CAP-\d{3}", concept))
matrix_ids = set()

for match in re.finditer(r"CAP-(\d{3})(?:\.\.(\d{3}))?", matrix):
    start = int(match.group(1))
    end = int(match.group(2) or match.group(1))
    matrix_ids.update(f"CAP-{index:03d}" for index in range(start, end + 1))

missing_concept = sorted(expected - concept_ids)
missing_matrix = sorted(expected - matrix_ids)
extra_concept = sorted(concept_ids - expected)
extra_matrix = sorted(matrix_ids - expected)

if missing_concept or missing_matrix or extra_concept or extra_matrix:
    raise SystemExit(
        "F02_MAP_FAIL "
        f"missing_concept={missing_concept} "
        f"missing_matrix={missing_matrix} "
        f"extra_concept={extra_concept} "
        f"extra_matrix={extra_matrix}"
    )

print("F02_MAP_OK requirements=45 mapped=45")
