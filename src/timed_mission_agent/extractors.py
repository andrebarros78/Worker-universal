from __future__ import annotations

import re
import time
from dataclasses import dataclass
from typing import Protocol

from .models import Extraction


class Extractor(Protocol):
    def extract(self, payload: object) -> Extraction:
        ...


_LABELS = {
    "cnpj": "cnpj",
    "numero": "numero",
    "número": "numero",
    "nota": "numero",
    "data": "data_emissao",
    "data_emissao": "data_emissao",
    "data emissão": "data_emissao",
    "subtotal": "subtotal",
    "desconto": "desconto",
    "acrescimos": "acrescimos",
    "acréscimos": "acrescimos",
    "total": "total",
}


@dataclass(slots=True)
class TextInvoiceExtractor:
    default_confidence: float = 0.98

    def extract(self, payload: object) -> Extraction:
        if not isinstance(payload, str):
            raise TypeError("TextInvoiceExtractor expects string payload")
        fields: dict[str, str] = {}
        confidence: dict[str, float] = {}
        for line in payload.splitlines():
            match = re.match(r"\s*([^:=]+)\s*[:=]\s*(.*?)\s*$", line)
            if not match:
                continue
            raw_key, value = match.groups()
            key = _LABELS.get(raw_key.strip().lower())
            if key:
                fields[key] = value.strip()
                confidence[key] = self.default_confidence
        if not fields:
            raise ValueError("no recognizable invoice fields")
        return Extraction(fields=fields, confidence=confidence, raw=payload)


@dataclass(slots=True)
class StaticExtractor:
    extraction: Extraction
    delay_ms: int = 0
    fail_times: int = 0
    _calls: int = 0

    def extract(self, payload: object) -> Extraction:
        self._calls += 1
        if self.delay_ms:
            time.sleep(self.delay_ms / 1000)
        if self._calls <= self.fail_times:
            raise RuntimeError("synthetic extractor failure")
        return self.extraction
