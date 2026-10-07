from __future__ import annotations

import json
import os
import time
from pathlib import Path

from timed_mission_agent.document_intelligence import (
    BinnedConfidenceCalibrator,
    CandidateReconciler,
    IdentityCalibrator,
    InvoiceDocumentEngine,
    expected_calibration_error,
    field_correctness,
    parse_candidates,
    percentile,
)
from timed_mission_agent.ocr import TesseractOCRProvider
from timed_mission_agent.validators import validate_invoice

ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / "benchmarks" / "f03_corpus"
REPORT = Path(os.environ.get("TMA_F03_REPORT_PATH", str(ROOT / "benchmarks" / "f03_benchmark_report.json")))

FIELD_ACCURACY_THRESHOLD = 0.97
VALIDATION_PASS_THRESHOLD = 0.95
P99_MS_THRESHOLD = 5000.0


def load_manifest() -> dict[str, object]:
    return json.loads((CORPUS / "manifest.json").read_text(encoding="utf-8"))


def main() -> None:
    manifest = load_manifest()
    cases = list(manifest["cases"])
    provider = TesseractOCRProvider()
    if not provider.available():
        raise SystemExit("F03_BENCHMARK_FAIL tesseract_unavailable")

    raw_rows: list[dict[str, object]] = []
    calibration_samples: list[tuple[float, bool]] = []
    evaluation_raw_samples: list[tuple[float, bool]] = []

    split = max(1, len(cases) // 2)
    for index, case in enumerate(cases):
        image_path = CORPUS / str(case["file"])
        result = provider.read(image_path, profile="default")
        candidates = parse_candidates(result)
        fields, confidence, _ = CandidateReconciler(IdentityCalibrator()).reconcile(candidates)
        correctness = field_correctness(fields, dict(case["expected_fields"]))
        target = calibration_samples if index < split else evaluation_raw_samples
        for field, correct in correctness.items():
            target.append((confidence.get(field, 0.0), correct))
        raw_rows.append(
            {
                "id": case["id"],
                "fields": fields,
                "confidence": confidence,
                "correctness": correctness,
                "elapsed_ms": result.elapsed_ms,
            }
        )

    calibrator = BinnedConfidenceCalibrator.fit(calibration_samples, bins=8)
    calibrated_eval_samples = [
        (calibrator.calibrate(confidence), correct)
        for confidence, correct in evaluation_raw_samples
    ]
    raw_ece = expected_calibration_error(evaluation_raw_samples, bins=8)
    calibrated_ece = expected_calibration_error(calibrated_eval_samples, bins=8)

    engine = InvoiceDocumentEngine(
        [provider],
        calibrator=calibrator,
        reread_threshold=0.96,
    )

    total_fields = 0
    correct_fields = 0
    validation_passes = 0
    latencies: list[float] = []
    reread_cases = 0
    final_cases: list[dict[str, object]] = []

    for case in cases:
        image_path = CORPUS / str(case["file"])
        started = time.perf_counter_ns()
        extraction = engine.extract(image_path)
        latency = (time.perf_counter_ns() - started) / 1_000_000
        latencies.append(latency)
        correctness = field_correctness(
            extraction.fields,
            dict(case["expected_fields"]),
        )
        correct_fields += sum(correctness.values())
        total_fields += len(correctness)
        validation = validate_invoice(extraction.fields)
        validation_passes += int(validation.ok)
        provider_runs = extraction.raw["providers"]
        if any(run["profile"] == "reread" for run in provider_runs):
            reread_cases += 1
        final_cases.append(
            {
                "id": case["id"],
                "variant": case["variant"],
                "latency_ms": latency,
                "correctness": correctness,
                "validation_ok": validation.ok,
                "validation_errors": validation.errors,
                "confidence": extraction.confidence,
                "provider_runs": provider_runs,
            }
        )

    field_accuracy = correct_fields / total_fields if total_fields else 0.0
    validation_pass_rate = validation_passes / len(cases) if cases else 0.0
    report = {
        "schema_version": 1,
        "dataset": manifest["dataset"],
        "dataset_version": manifest["version"],
        "provider": provider.provider_id,
        "provider_version": provider.version(),
        "cases": len(cases),
        "fields": total_fields,
        "field_accuracy": field_accuracy,
        "field_accuracy_threshold": FIELD_ACCURACY_THRESHOLD,
        "validation_pass_rate": validation_pass_rate,
        "validation_pass_threshold": VALIDATION_PASS_THRESHOLD,
        "latency_ms": {
            "p50": percentile(latencies, 0.50),
            "p95": percentile(latencies, 0.95),
            "p99": percentile(latencies, 0.99),
            "threshold_p99": P99_MS_THRESHOLD,
        },
        "calibration": {
            "fit_samples": len(calibration_samples),
            "evaluation_samples": len(evaluation_raw_samples),
            "raw_ece": raw_ece,
            "calibrated_ece": calibrated_ece,
            "non_regression_margin": 0.02,
        },
        "selective_reread": {
            "cases": reread_cases,
            "exercised": reread_cases > 0,
        },
        "cases_detail": final_cases,
    }
    REPORT.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    failures: list[str] = []
    if field_accuracy < FIELD_ACCURACY_THRESHOLD:
        failures.append(f"field_accuracy={field_accuracy:.4f}")
    if validation_pass_rate < VALIDATION_PASS_THRESHOLD:
        failures.append(f"validation_pass_rate={validation_pass_rate:.4f}")
    if report["latency_ms"]["p99"] > P99_MS_THRESHOLD:
        failures.append(f"p99_ms={report['latency_ms']['p99']:.2f}")
    if calibrated_ece > raw_ece + 0.02:
        failures.append(
            f"calibration_regressed raw={raw_ece:.4f} calibrated={calibrated_ece:.4f}"
        )
    if reread_cases == 0:
        failures.append("selective_reread_not_exercised")

    print(
        "F03_BENCHMARK "
        f"cases={len(cases)} fields={total_fields} "
        f"accuracy={field_accuracy:.4f} "
        f"validation={validation_pass_rate:.4f} "
        f"p50_ms={report['latency_ms']['p50']:.2f} "
        f"p95_ms={report['latency_ms']['p95']:.2f} "
        f"p99_ms={report['latency_ms']['p99']:.2f} "
        f"raw_ece={raw_ece:.4f} calibrated_ece={calibrated_ece:.4f} "
        f"reread_cases={reread_cases}"
    )
    if failures:
        raise SystemExit("F03_BENCHMARK_FAIL " + " ".join(failures))
    print("F03_BENCHMARK=PASS")


if __name__ == "__main__":
    main()
