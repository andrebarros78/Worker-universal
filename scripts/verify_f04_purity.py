from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "web-worker" / "src"

forbidden = {
    "robotjs",
    "@nut-tree/nut-js",
    "pyautogui",
    "autohotkey",
    "sendinput",
    "setcursorpos",
    "puppeteer-extra-plugin-stealth",
    "fingerprint spoof",
    "captcha bypass",
    "stealth plugin",
}

violations: list[str] = []
for path in sorted(SRC.rglob("*.ts")):
    text = path.read_text(encoding="utf-8").lower()
    for token in forbidden:
        if token in text:
            violations.append(f"{path.relative_to(ROOT)}:{token}")

if violations:
    raise SystemExit("F04_PURITY_FAIL=" + ",".join(violations))

runtime = (SRC / "browser_runtime.ts").read_text(encoding="utf-8")
if 'throw new Error("headless_required")' not in runtime:
    raise SystemExit("F04_PURITY_FAIL=headless_not_enforced")
if "chromium.launchServer" not in runtime:
    raise SystemExit("F04_PURITY_FAIL=browser_process_not_isolated")

package = json.loads((ROOT / "web-worker" / "package.json").read_text(encoding="utf-8"))
dependencies = package.get("dependencies", {})
if dependencies != {"playwright": "1.64.0"}:
    raise SystemExit(f"F04_PURITY_FAIL=unexpected_dependencies:{dependencies}")

fixture = (ROOT / "web-worker" / "test" / "fixture_server.ts").read_text(encoding="utf-8")
if "127.0.0.1" not in fixture:
    raise SystemExit("F04_PURITY_FAIL=fixture_not_loopback")

print(
    "F04_PURITY_OK "
    f"source_files={len(list(SRC.rglob('*.ts')))} "
    "headless=enforced native_input_dependencies=0 external_test_network=0"
)
