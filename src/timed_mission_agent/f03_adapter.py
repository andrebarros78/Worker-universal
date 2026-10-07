from __future__ import annotations

from pathlib import Path
from typing import Any

from .document_intelligence import InvoiceDocumentEngine
from .ocr import TesseractOCRProvider


class TesseractCapabilityAdapter:
    def __init__(self) -> None:
        self.provider = TesseractOCRProvider()
        self.engine = InvoiceDocumentEngine([self.provider])

    def manifest(self) -> dict[str, Any]:
        return {
            "schema_version": 1,
            "adapter_id": "vision.ocr.tesseract",
            "adapter_version": "1.0.0",
            "sdk_contract_version": "1.0.0",
            "capabilities": ["vision.ocr.tesseract", "document.invoice.extract"],
            "config_schema_ref": None,
            "secret_refs": [],
            "simulation_supported": True,
        }

    def health(self) -> dict[str, str]:
        return {
            "lifecycle": "healthy" if self.provider.available() else "unavailable",
            "readiness": "operational" if self.provider.available() else "installed",
            "version": self.provider.version(),
        }

    def invoke(self, image_path: str | Path) -> dict[str, Any]:
        extraction = self.engine.extract(image_path)
        return {
            "status": "succeeded",
            "fields": extraction.fields,
            "confidence": extraction.confidence,
            "evidence": extraction.raw,
        }
