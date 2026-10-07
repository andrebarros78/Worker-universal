from __future__ import annotations

import json
import os
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / "benchmarks" / "f03_corpus"
MANIFEST = CORPUS / "manifest.json"
REPORT = Path(os.environ.get("TMA_F03_REPORT_PATH", str(ROOT / "benchmarks" / "f03_benchmark_report.json")))

manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
report = json.loads(REPORT.read_text(encoding="utf-8"))

if manifest.get("version") != "1.0.0":
    raise SystemExit("F03_ARTIFACT_FAIL corpus_version")
cases = manifest.get("cases", [])
if len(cases) != 16:
    raise SystemExit(f"F03_ARTIFACT_FAIL cases={len(cases)}")

variants = {case["variant"] for case in cases}
expected_variants = {"clean", "low_contrast", "mild_blur", "compressed"}
if variants != expected_variants:
    raise SystemExit(f"F03_ARTIFACT_FAIL variants={sorted(variants)}")

for case in cases:
    image = CORPUS / case["file"]
    if not image.is_file() or image.stat().st_size <= 0:
        raise SystemExit(f"F03_ARTIFACT_FAIL image={case['file']}")
    required = {"cnpj", "numero", "data_emissao", "subtotal", "desconto", "acrescimos", "total"}
    if set(case["expected_fields"]) != required:
        raise SystemExit(f"F03_ARTIFACT_FAIL fields={case['id']}")

if report.get("dataset_version") != manifest["version"]:
    raise SystemExit("F03_ARTIFACT_FAIL report_dataset_version")
if report.get("field_accuracy", 0.0) < report.get("field_accuracy_threshold", 1.0):
    raise SystemExit("F03_ARTIFACT_FAIL field_accuracy")
if report.get("validation_pass_rate", 0.0) < report.get("validation_pass_threshold", 1.0):
    raise SystemExit("F03_ARTIFACT_FAIL validation")
if report["latency_ms"]["p99"] > report["latency_ms"]["threshold_p99"]:
    raise SystemExit("F03_ARTIFACT_FAIL p99")
if not report["selective_reread"]["exercised"]:
    raise SystemExit("F03_ARTIFACT_FAIL reread")
cal = report["calibration"]
if cal["calibrated_ece"] > cal["raw_ece"] + cal["non_regression_margin"]:
    raise SystemExit("F03_ARTIFACT_FAIL calibration")

concept = (ROOT / "docs" / "F03_VISION_OCR_MAP.md").read_text(encoding="utf-8")
matrix = (ROOT / "docs" / "F03_IMPLEMENTATION_MATRIX.md").read_text(encoding="utf-8")
expected = {f"VIS-{index:03d}" for index in range(1, 36)}
concept_ids = set(re.findall(r"VIS-\d{3}", concept))
matrix_ids: set[str] = set()
for match in re.finditer(r"VIS-(\d{3})(?:\.\.(\d{3}))?", matrix):
    start = int(match.group(1))
    end = int(match.group(2) or match.group(1))
    matrix_ids.update(f"VIS-{i:03d}" for i in range(start, end + 1))
if concept_ids != expected or matrix_ids != expected:
    raise SystemExit(
        f"F03_ARTIFACT_FAIL map concept_missing={sorted(expected-concept_ids)} "
        f"matrix_missing={sorted(expected-matrix_ids)}"
    )

print(
    "F03_ARTIFACTS_OK "
    f"requirements=35 cases={len(cases)} variants={len(variants)} "
    f"accuracy={report['field_accuracy']:.4f} "
    f"validation={report['validation_pass_rate']:.4f}"
)
