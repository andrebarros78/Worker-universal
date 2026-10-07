from __future__ import annotations

import math
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Protocol

from .models import Extraction
from .ocr import OCRProvider, OCRResult
from .validators import normalize_field, validate_invoice
from .vision import ImageIngestor


FIELD_LABELS = {
    "cnpj": "cnpj",
    "numero": "numero",
    "número": "numero",
    "nota": "numero",
    "data": "data_emissao",
    "data emissao": "data_emissao",
    "data emissão": "data_emissao",
    "data_emissao": "data_emissao",
    "subtotal": "subtotal",
    "desconto": "desconto",
    "acrescimos": "acrescimos",
    "acréscimos": "acrescimos",
    "total": "total",
}

REQUIRED_INVOICE_FIELDS = (
    "cnpj",
    "numero",
    "data_emissao",
    "subtotal",
    "desconto",
    "acrescimos",
    "total",
)


@dataclass(frozen=True, slots=True)
class FieldCandidate:
    field: str
    value: str
    confidence: float
    provider: str
    profile: str


class ConfidenceCalibrator(Protocol):
    def calibrate(self, confidence: float) -> float: ...


@dataclass(frozen=True, slots=True)
class IdentityCalibrator:
    def calibrate(self, confidence: float) -> float:
        return max(0.0, min(1.0, float(confidence)))


@dataclass(frozen=True, slots=True)
class BinnedConfidenceCalibrator:
    edges: tuple[float, ...]
    values: tuple[float, ...]

    @classmethod
    def fit(
        cls,
        samples: Iterable[tuple[float, bool]],
        bins: int = 10,
        prior_strength: float = 2.0,
    ) -> "BinnedConfidenceCalibrator":
        if bins < 2:
            raise ValueError("bins must be >= 2")
        rows = [(max(0.0, min(1.0, float(c))), bool(ok)) for c, ok in samples]
        edges = tuple(index / bins for index in range(bins + 1))
        values: list[float] = []
        global_rate = (
            sum(1 for _, ok in rows if ok) / len(rows)
            if rows
            else 0.5
        )
        previous = 0.0
        for index in range(bins):
            low, high = edges[index], edges[index + 1]
            in_bin = [
                ok
                for confidence, ok in rows
                if (
                    low <= confidence <= high
                    if index == bins - 1
                    else low <= confidence < high
                )
            ]
            positives = sum(in_bin)
            value = (positives + prior_strength * global_rate) / (
                len(in_bin) + prior_strength
            )
            value = max(previous, value)
            previous = value
            values.append(max(0.0, min(1.0, value)))
        return cls(edges=edges, values=tuple(values))

    def calibrate(self, confidence: float) -> float:
        value = max(0.0, min(1.0, float(confidence)))
        for index in range(len(self.values)):
            if value <= self.edges[index + 1] or index == len(self.values) - 1:
                return self.values[index]
        return self.values[-1]


def expected_calibration_error(
    samples: Iterable[tuple[float, bool]],
    bins: int = 10,
) -> float:
    rows = [(max(0.0, min(1.0, float(c))), bool(ok)) for c, ok in samples]
    if not rows:
        return 0.0
    total = len(rows)
    error = 0.0
    for index in range(bins):
        low, high = index / bins, (index + 1) / bins
        bucket = [
            (confidence, ok)
            for confidence, ok in rows
            if (
                low <= confidence <= high
                if index == bins - 1
                else low <= confidence < high
            )
        ]
        if not bucket:
            continue
        mean_conf = sum(confidence for confidence, _ in bucket) / len(bucket)
        accuracy = sum(1 for _, ok in bucket if ok) / len(bucket)
        error += (len(bucket) / total) * abs(mean_conf - accuracy)
    return error


def parse_candidates(result: OCRResult) -> list[FieldCandidate]:
    candidates: list[FieldCandidate] = []
    for line in result.lines:
        match = re.match(r"\s*([^:=]+)\s*[:=]\s*(.*?)\s*$", line.text)
        if not match:
            continue
        raw_key, value = match.groups()
        key = FIELD_LABELS.get(raw_key.strip().lower())
        if not key or not value.strip():
            continue
        candidates.append(
            FieldCandidate(
                field=key,
                value=value.strip(),
                confidence=line.confidence,
                provider=result.provider,
                profile=result.profile,
            )
        )
    return candidates


class CandidateReconciler:
    def __init__(self, calibrator: ConfidenceCalibrator | None = None) -> None:
        self.calibrator = calibrator or IdentityCalibrator()

    def reconcile(
        self,
        candidates: Iterable[FieldCandidate],
    ) -> tuple[dict[str, str], dict[str, float], dict[str, list[dict[str, object]]]]:
        grouped: dict[str, dict[str, list[FieldCandidate]]] = {}
        display_values: dict[tuple[str, str], str] = {}

        for candidate in candidates:
            try:
                normalized = str(normalize_field(candidate.field, candidate.value))
            except Exception:
                normalized = candidate.value.strip()
            grouped.setdefault(candidate.field, {}).setdefault(normalized, []).append(candidate)
            display_values[(candidate.field, normalized)] = candidate.value

        fields: dict[str, str] = {}
        confidence: dict[str, float] = {}
        evidence: dict[str, list[dict[str, object]]] = {}

        for field, values in grouped.items():
            ranked: list[tuple[float, int, str, list[FieldCandidate]]] = []
            for normalized, items in values.items():
                calibrated = [self.calibrator.calibrate(item.confidence) for item in items]
                providers = len({item.provider for item in items})
                profiles = len({item.profile for item in items})
                support = sum(calibrated) + 0.03 * max(0, providers - 1) + 0.01 * max(0, profiles - 1)
                ranked.append((support, providers, normalized, items))

            ranked.sort(key=lambda item: (-item[0], -item[1], item[2]))
            support, _, normalized, winners = ranked[0]
            fields[field] = display_values[(field, normalized)]
            confidence[field] = max(
                0.0,
                min(
                    1.0,
                    sum(self.calibrator.calibrate(item.confidence) for item in winners)
                    / len(winners),
                ),
            )
            evidence[field] = [
                {
                    "value": item.value,
                    "confidence": item.confidence,
                    "provider": item.provider,
                    "profile": item.profile,
                }
                for item in winners
            ]
        return fields, confidence, evidence


class InvoiceDocumentEngine:
    def __init__(
        self,
        providers: list[OCRProvider],
        calibrator: ConfidenceCalibrator | None = None,
        reread_threshold: float = 0.82,
    ) -> None:
        if not providers:
            raise ValueError("at least one OCR provider is required")
        self.providers = providers
        self.reconciler = CandidateReconciler(calibrator)
        self.reread_threshold = reread_threshold
        self.ingestor = ImageIngestor()

    def extract(self, payload: object) -> Extraction:
        asset = self.ingestor.ingest(payload)
        candidates: list[FieldCandidate] = []
        provider_runs: list[dict[str, object]] = []

        for provider in self.providers:
            if not provider.available():
                continue
            result = provider.read(asset.path, profile="default")
            candidates.extend(parse_candidates(result))
            provider_runs.append(
                {
                    "provider": result.provider,
                    "profile": result.profile,
                    "elapsed_ms": result.elapsed_ms,
                }
            )

        if not candidates:
            raise RuntimeError("no OCR provider produced recognizable invoice fields")

        fields, confidence, evidence = self.reconciler.reconcile(candidates)
        confidence_uncertain = {
            field
            for field in REQUIRED_INVOICE_FIELDS
            if confidence.get(field, 0.0) < self.reread_threshold
        }
        initial_validation = validate_invoice(fields)
        validation_uncertain = _fields_from_validation_errors(initial_validation.errors)
        uncertain = confidence_uncertain | validation_uncertain

        if uncertain:
            reread_candidates_all: list[FieldCandidate] = []
            for provider in self.providers:
                if not provider.available():
                    continue
                result = provider.read(asset.path, profile="reread")
                reread_candidates = [
                    candidate
                    for candidate in parse_candidates(result)
                    if candidate.field in uncertain
                ]
                reread_candidates_all.extend(reread_candidates)
                provider_runs.append(
                    {
                        "provider": result.provider,
                        "profile": result.profile,
                        "elapsed_ms": result.elapsed_ms,
                        "fields": sorted(uncertain),
                    }
                )

            replacement_fields = {
                field
                for field in validation_uncertain
                if any(candidate.field == field for candidate in reread_candidates_all)
            }
            if replacement_fields:
                candidates = [
                    candidate
                    for candidate in candidates
                    if candidate.field not in replacement_fields
                ]
            candidates.extend(reread_candidates_all)
            fields, confidence, evidence = self.reconciler.reconcile(candidates)

        validation = validate_invoice(fields)
        if not validation.ok:
            for error in validation.errors:
                if ":" not in error:
                    continue
                _, field = error.split(":", 1)
                if field in confidence:
                    confidence[field] = min(confidence[field], 0.49)

        return Extraction(
            fields=fields,
            confidence=confidence,
            raw={
                "image": {
                    "path": str(asset.path),
                    "sha256": asset.sha256,
                    "width": asset.width,
                    "height": asset.height,
                },
                "providers": provider_runs,
                "evidence": evidence,
                "validation": {
                    "ok": validation.ok,
                    "errors": validation.errors,
                },
            },
        )


def _fields_from_validation_errors(errors: Iterable[str]) -> set[str]:
    fields: set[str] = set()
    for error in errors:
        if ":" not in error:
            continue
        kind, field = error.split(":", 1)
        if kind in {"missing", "invalid"} and field in REQUIRED_INVOICE_FIELDS:
            fields.add(field)
        elif kind == "mismatch" and field == "total":
            fields.update({"subtotal", "desconto", "acrescimos", "total"})
    return fields


def field_correctness(
    fields: dict[str, object],
    expected: dict[str, object],
) -> dict[str, bool]:
    result: dict[str, bool] = {}
    for key, expected_value in expected.items():
        if key not in fields:
            result[key] = False
            continue
        try:
            result[key] = normalize_field(key, fields[key]) == normalize_field(
                key, expected_value
            )
        except Exception:
            result[key] = False
    return result


def percentile(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    index = max(0, min(len(ordered) - 1, math.ceil(p * len(ordered)) - 1))
    return float(ordered[index])
