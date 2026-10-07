from __future__ import annotations

import time
from dataclasses import dataclass

from .extractors import Extractor
from .ledger import EvidenceLedger
from .models import AttemptResult, MissionTask, RunResult, TaskStatus
from .scoring import aggregate_confidence
from .validators import validate_invoice


@dataclass(slots=True)
class RunnerConfig:
    max_attempts: int = 3
    min_confidence: float = 0.90
    required_fields: tuple[str, ...] = ("cnpj", "numero", "data_emissao", "total")


class TimedMissionRunner:
    def __init__(self, extractor: Extractor, ledger: EvidenceLedger, config: RunnerConfig | None = None) -> None:
        self.extractor = extractor
        self.ledger = ledger
        self.config = config or RunnerConfig()

    def run(self, task: MissionTask) -> RunResult:
        if task.deadline_ms <= 0:
            raise ValueError("deadline_ms must be positive")
        started = time.perf_counter_ns()
        attempts: list[AttemptResult] = []
        self.ledger.record(task.mission_id, task.task_id, "task_started", {"deadline_ms": task.deadline_ms})

        for attempt_no in range(1, self.config.max_attempts + 1):
            elapsed = (time.perf_counter_ns() - started) / 1_000_000
            if elapsed >= task.deadline_ms:
                return self._finish(task, TaskStatus.DEADLINE_EXCEEDED, started, attempts, errors=["deadline_before_attempt"])

            try:
                extraction = self.extractor.extract(task.payload)
                validation = validate_invoice(extraction.fields)
                confidence = aggregate_confidence(extraction.confidence, self.config.required_fields)
                elapsed = (time.perf_counter_ns() - started) / 1_000_000
                attempts.append(
                    AttemptResult(attempt_no, elapsed, extraction, validation, confidence)
                )
                self.ledger.record(
                    task.mission_id,
                    task.task_id,
                    "attempt",
                    {
                        "attempt": attempt_no,
                        "elapsed_ms": elapsed,
                        "validation_ok": validation.ok,
                        "validation_errors": validation.errors,
                        "confidence": confidence,
                    },
                )
                elapsed = (time.perf_counter_ns() - started) / 1_000_000
                if elapsed > task.deadline_ms:
                    return self._finish(
                        task, TaskStatus.DEADLINE_EXCEEDED, started, attempts,
                        fields=extraction.fields, confidence=confidence,
                        errors=["deadline_after_attempt"],
                    )
                if validation.ok and confidence >= self.config.min_confidence:
                    return self._finish(
                        task, TaskStatus.SUCCEEDED, started, attempts,
                        fields=extraction.fields, confidence=confidence,
                    )
            except Exception as exc:
                elapsed = (time.perf_counter_ns() - started) / 1_000_000
                attempts.append(
                    AttemptResult(
                        attempt=attempt_no,
                        elapsed_ms=elapsed,
                        extraction=None,
                        validation=None,
                        confidence=0.0,
                        error=f"{type(exc).__name__}:{exc}",
                    )
                )
                self.ledger.record(
                    task.mission_id, task.task_id, "attempt_error",
                    {"attempt": attempt_no, "elapsed_ms": elapsed, "error_type": type(exc).__name__},
                )

        last = attempts[-1] if attempts else None
        fields = last.extraction.fields if last and last.extraction else {}
        confidence = last.confidence if last else 0.0
        errors: list[str] = []
        if last and last.error:
            errors.append(last.error)
        if last and last.validation:
            errors.extend(last.validation.errors)
        if last and last.confidence < self.config.min_confidence:
            errors.append("confidence_below_threshold")
        return self._finish(task, TaskStatus.FAILED, started, attempts, fields, confidence, errors)

    def _finish(
        self,
        task: MissionTask,
        status: TaskStatus,
        started_ns: int,
        attempts: list[AttemptResult],
        fields: dict[str, object] | None = None,
        confidence: float = 0.0,
        errors: list[str] | None = None,
    ) -> RunResult:
        elapsed = (time.perf_counter_ns() - started_ns) / 1_000_000
        result = RunResult(
            task_id=task.task_id,
            mission_id=task.mission_id,
            status=status,
            elapsed_ms=elapsed,
            attempts=attempts,
            fields=dict(fields or {}),
            confidence=confidence,
            errors=list(errors or []),
        )
        self.ledger.record(
            task.mission_id,
            task.task_id,
            "task_finished",
            {"status": status.value, "elapsed_ms": elapsed, "attempts": len(attempts), "errors": result.errors},
        )
        return result
