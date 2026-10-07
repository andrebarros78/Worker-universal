from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
engine = (ROOT / "src" / "timed_mission_agent" / "document_intelligence.py").read_text(encoding="utf-8").lower()
ocr = (ROOT / "src" / "timed_mission_agent" / "ocr.py").read_text(encoding="utf-8").lower()

for forbidden in ("openai", "anthropic", "gemini", "google.cloud", "azure.ai"):
    if forbidden in engine:
        raise SystemExit("F03_PURITY_FAIL engine_vendor=" + forbidden)

if "tesseractocrprovider" in engine or "tesseract.exe" in engine:
    raise SystemExit("F03_PURITY_FAIL document_engine_depends_on_tesseract")

required_engine = ("ocrprovider", "candidate", "validate_invoice")
for token in required_engine:
    if token not in engine:
        raise SystemExit("F03_PURITY_FAIL missing_engine_surface=" + token)

if "class ocrprovider" not in ocr or "class remotevisionprovider" not in ocr:
    raise SystemExit("F03_PURITY_FAIL provider_protocols")

print("F03_PURITY_OK document_engine=provider_neutral remote_extension=present")
