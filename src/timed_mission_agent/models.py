from __future__ import annotations

from dataclasses import dataclass, field
from enum import StrEnum
from typing import Any


class TaskStatus(StrEnum):
    PENDING = "pending"
    RUNNING = "running"
    SUCCEEDED = "succeeded"
    FAILED = "failed"
    DEADLINE_EXCEEDED = "deadline_exceeded"


@dataclass(slots=True)
class MissionTask:
    task_id: str
    mission_id: str
    payload: Any
    deadline_ms: int
    expected_fields: dict[str, Any] | None = None


@dataclass(slots=True)
class Extraction:
    fields: dict[str, Any]
    confidence: dict[str, float] = field(default_factory=dict)
    raw: Any = None


@dataclass(slots=True)
class ValidationResult:
    ok: bool
    errors: list[str] = field(default_factory=list)


@dataclass(slots=True)
class AttemptResult:
    attempt: int
    elapsed_ms: float
    extraction: Extraction | None
    validation: ValidationResult | None
    confidence: float
    error: str | None = None


@dataclass(slots=True)
class RunResult:
    task_id: str
    mission_id: str
    status: TaskStatus
    elapsed_ms: float
    attempts: list[AttemptResult]
    fields: dict[str, Any] = field(default_factory=dict)
    confidence: float = 0.0
    errors: list[str] = field(default_factory=list)
