from __future__ import annotations


def aggregate_confidence(confidence: dict[str, float], required_fields: tuple[str, ...]) -> float:
    if not required_fields:
        return 1.0
    values = [float(confidence.get(field, 0.0)) for field in required_fields]
    return sum(values) / len(values)
