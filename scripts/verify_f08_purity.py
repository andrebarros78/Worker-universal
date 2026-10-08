from __future__ import annotations

import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
QUAL = ROOT / "qualification-rust"

cargo = tomllib.loads((QUAL / "Cargo.toml").read_text(encoding="utf-8"))
dependencies = set(cargo.get("dependencies", {}))
expected = {"sha2", "tma-foundation", "tma-validation"}
if dependencies != expected:
    raise SystemExit(f"F08_PURITY_FAIL dependencies={sorted(dependencies)}")

forbidden = (
    "openai",
    "anthropic",
    "gemini",
    "openrouter",
    "playwright",
    "tesseract",
    "selenium",
    "99freelas",
    "homeocta",
    "mercadolivre",
    "http://",
    "https://",
)
violations: list[str] = []
for path in sorted((QUAL / "src").rglob("*.rs")):
    text = path.read_text(encoding="utf-8").lower()
    for token in forbidden:
        if token in text:
            violations.append(f"{path.relative_to(ROOT)}:{token}")
if violations:
    raise SystemExit("F08_PURITY_FAIL=" + ",".join(violations))

harness = (QUAL / "src" / "harness.rs").read_text(encoding="utf-8")
if "ValidationReport" not in harness or "report.is_proven()" not in harness:
    raise SystemExit("F08_PURITY_FAIL missing_f07_validation_binding")

registry = (ROOT / "foundation-rust" / "src" / "registry.rs").read_text(encoding="utf-8")
if "promotion skip rejected" not in registry:
    raise SystemExit("F08_PURITY_FAIL sequential_promotion_not_enforced")

# Any production code outside foundation/qualification that mutates promotion is a bypass.
bypass: list[str] = []
for src_root in [
    ROOT / "core-rust" / "src",
    ROOT / "planner-rust" / "src",
    ROOT / "validation-rust" / "src",
]:
    if not src_root.exists():
        continue
    for path in src_root.rglob("*.rs"):
        if ".set_promotion(" in path.read_text(encoding="utf-8"):
            bypass.append(str(path.relative_to(ROOT)))
if bypass:
    raise SystemExit("F08_PURITY_FAIL promotion_bypass=" + ",".join(bypass))

print(
    "F08_PURITY_OK provider_sdks=0 platform_apis=0 "
    "f07_validation_bound=1 promotion_bypass=0 sequential_registry_gate=1"
)
