from __future__ import annotations

import re
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VALIDATION = ROOT / "validation-rust"
cargo = tomllib.loads((VALIDATION / "Cargo.toml").read_text(encoding="utf-8"))
dependencies = set(cargo.get("dependencies", {}))
expected = {"sha2", "tma-foundation", "tma-core", "tma-planner"}
if dependencies != expected:
    raise SystemExit(f"F07_PURITY_FAIL dependencies={sorted(dependencies)}")

forbidden = (
    "openai",
    "anthropic",
    "gemini",
    "openrouter",
    "playwright",
    "tesseract",
    "selenium",
    "pyautogui",
    "robotjs",
)
violations: list[str] = []
for path in sorted((VALIDATION / "src").rglob("*.rs")):
    text = path.read_text(encoding="utf-8").lower()
    for token in forbidden:
        if token in text:
            violations.append(f"{path.relative_to(ROOT)}:{token}")
if violations:
    raise SystemExit("F07_PURITY_FAIL=" + ",".join(violations))

recovery = (VALIDATION / "src" / "recovery.rs").read_text(encoding="utf-8")
foundation = (ROOT / "foundation-rust" / "src" / "model.rs").read_text(encoding="utf-8")
classes = re.search(r"pub enum ErrorClass \{(.*?)\}", foundation, re.S)
if not classes:
    raise SystemExit("F07_PURITY_FAIL error_class_enum_missing")
variants = [
    line.strip().rstrip(",")
    for line in classes.group(1).splitlines()
    if line.strip()
]
missing = [variant for variant in variants if f"ErrorClass::{variant}" not in recovery]
if missing:
    raise SystemExit("F07_PURITY_FAIL recovery_missing=" + ",".join(missing))

core = (ROOT / "core-rust" / "src" / "durable.rs").read_text(encoding="utf-8")
for token in (
    "validation_receipts",
    "validation_receipt_required",
    "validator must be independent from lease owner",
):
    if token not in core:
        raise SystemExit("F07_PURITY_FAIL core_gate_missing=" + token)

print(
    "F07_PURITY_OK "
    f"error_classes={len(variants)} provider_sdks=0 "
    "success_gate=independent_validation_receipt"
)
