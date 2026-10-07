from __future__ import annotations

import math
from dataclasses import dataclass
from statistics import median

from .models import MissionTask, RunResult, TaskStatus
from .validators import normalize_field


@dataclass(slots=True)
class BenchmarkReport:
    cases: int
    success_rate: float
    sla_pass_rate: float
    field_accuracy: float
    p50_ms: float
    p95_ms: float


def _percentile(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    index = max(0, min(len(ordered) - 1, math.ceil(p * len(ordered)) - 1))
    return ordered[index]


def build_report(tasks: list[MissionTask], results: list[RunResult]) -> BenchmarkReport:
    by_id = {result.task_id: result for result in results}
    latencies = [result.elapsed_ms for result in results]
    successes = sum(result.status == TaskStatus.SUCCEEDED for result in results)
    sla_passes = sum(
        by_id[task.task_id].elapsed_ms <= task.deadline_ms
        for task in tasks if task.task_id in by_id
    )

    correct = 0
    total = 0
    for task in tasks:
        if not task.expected_fields or task.task_id not in by_id:
            continue
        result = by_id[task.task_id]
        for key, expected in task.expected_fields.items():
            total += 1
            if key not in result.fields:
                continue
            try:
                actual_norm = normalize_field(key, result.fields[key])
                expected_norm = normalize_field(key, expected)
            except Exception:
                continue
            correct += actual_norm == expected_norm

    return BenchmarkReport(
        cases=len(results),
        success_rate=(successes / len(results)) if results else 0.0,
        sla_pass_rate=(sla_passes / len(results)) if results else 0.0,
        field_accuracy=(correct / total) if total else 0.0,
        p50_ms=float(median(latencies)) if latencies else 0.0,
        p95_ms=_percentile(latencies, 0.95),
    )
