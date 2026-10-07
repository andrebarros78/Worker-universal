from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

required = [
    "00_START_HERE.md",
    "AGENTS.md",
    "PROJECT_DNA.md",
    "docs/PROJECT_CONCEPT.md",
    "docs/EXECUTIVE_PROJECT.md",
    "docs/EXECUTION_PHASES.md",
    "docs/PHASE_LEDGER.md",
    "docs/CANONICAL_STATE.md",
    "state/EXECUTION_STATE.json",
]

missing = [rel for rel in required if not (ROOT / rel).is_file()]
if missing:
    raise SystemExit("CONTINUITY_MISSING=" + ",".join(missing))

state = json.loads((ROOT / "state/EXECUTION_STATE.json").read_text(encoding="utf-8"))

assert state["schema_version"] == 1
assert Path(state["canonical_root"]) == ROOT
assert state["current_phase"]["status"] == "IN_PROGRESS"
assert state["next_phase"]["status"] == "PLANNED"
assert state["current_phase"]["id"] != state["next_phase"]["id"]
assert state["terminal_phase"]["terminal_label"] == "MISSION_PROVEN"

ledger = (ROOT / state["phase_ledger"]).read_text(encoding="utf-8")
current_id = state["current_phase"]["id"]
next_id = state["next_phase"]["id"]

if f"| {current_id} |" not in ledger or "IN_PROGRESS" not in ledger:
    raise SystemExit("CONTINUITY_LEDGER_CURRENT_PHASE_MISMATCH")
if f"| {next_id} |" not in ledger or "PLANNED" not in ledger:
    raise SystemExit("CONTINUITY_LEDGER_NEXT_PHASE_MISMATCH")

start = (ROOT / "00_START_HERE.md").read_text(encoding="utf-8")
for rel in (
    "PROJECT_DNA.md",
    "docs/PROJECT_CONCEPT.md",
    "docs/EXECUTIVE_PROJECT.md",
    "state/EXECUTION_STATE.json",
    "docs/CANONICAL_STATE.md",
    "docs/PHASE_LEDGER.md",
):
    if rel not in start:
        raise SystemExit(f"CONTINUITY_ENTRYPOINT_MISSING_REFERENCE={rel}")

print(
    "CONTINUITY_OK "
    f"current={current_id} "
    f"next={next_id} "
    f"terminal={state['terminal_phase']['id']} "
    f"root={ROOT}"
)
