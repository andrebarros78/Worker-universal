from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
core = ROOT / "core-rust" / "src"

forbidden = {
    "openai",
    "anthropic",
    "gemini",
    "playwright",
    "selenium",
    "homeocta",
    "99freelas",
    "chromium",
    "chrome",
    "mcp",
}

violations: list[str] = []
for path in sorted(core.rglob("*.rs")):
    text = path.read_text(encoding="utf-8").lower()
    for token in sorted(forbidden):
        if token in text:
            violations.append(f"{path.relative_to(ROOT)}:{token}")

if violations:
    raise SystemExit("CORE_PURITY_FAIL=" + ",".join(violations))

print(f"CORE_PURITY_OK files={len(list(core.rglob('*.rs')))} forbidden_tokens={len(forbidden)}")
