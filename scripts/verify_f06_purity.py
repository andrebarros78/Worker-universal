from __future__ import annotations

import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PLANNER = ROOT / "planner-rust"
cargo = tomllib.loads((PLANNER / "Cargo.toml").read_text(encoding="utf-8"))
dependencies = set(cargo.get("dependencies", {}))

expected = {"sha2", "tma-foundation", "tma-core"}
if dependencies != expected:
    raise SystemExit(f"F06_PURITY_FAIL dependencies={sorted(dependencies)}")

forbidden = (
    "openai",
    "anthropic",
    "gemini",
    "openrouter",
    "playwright",
    "tesseract",
    "selenium",
    "redis",
    "rabbitmq",
    "kafka",
)
violations: list[str] = []
for path in sorted((PLANNER / "src").rglob("*.rs")):
    text = path.read_text(encoding="utf-8").lower()
    for token in forbidden:
        if token in text:
            violations.append(f"{path.relative_to(ROOT)}:{token}")

if violations:
    raise SystemExit("F06_PURITY_FAIL=" + ",".join(violations))

router = (PLANNER / "src" / "router.rs").read_text(encoding="utf-8")
if "CapabilityRegistry" not in router or "PolicyEngine" not in router:
    raise SystemExit("F06_PURITY_FAIL router_not_using_foundation")
bridge = (PLANNER / "src" / "durable_bridge.rs").read_text(encoding="utf-8")
if "DurableStore" not in bridge or "save_checkpoint" not in bridge:
    raise SystemExit("F06_PURITY_FAIL durable_bridge_missing")

print(
    "F06_PURITY_OK "
    "provider_sdks=0 browser_ocr_duplication=0 "
    "capability_authority=tma-foundation durable_authority=tma-core"
)
