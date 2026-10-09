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
assert state["terminal_phase"]["terminal_label"] == "MISSION_PROVEN"
current_id = state["current_phase"]["id"]
terminal_id = state["terminal_phase"]["id"]
next_obj = state.get("next_phase")
if current_id == terminal_id:
    # F13 is terminal; there is no invented F14 or automatic MISSION_PROVEN.
    if next_obj is not None:
        raise SystemExit("CONTINUITY_TERMINAL_NEXT_MUST_BE_NULL")
    next_id = "NONE"
else:
    if not isinstance(next_obj, dict) or next_obj.get("status") != "PLANNED":
        raise SystemExit("CONTINUITY_NEXT_PHASE_INVALID")
    next_id = next_obj["id"]
    if current_id == next_id:
        raise SystemExit("CONTINUITY_DUPLICATE_CURRENT_NEXT")

ledger = (ROOT / state["phase_ledger"]).read_text(encoding="utf-8")
rows = ledger.splitlines()
current_rows = [row for row in rows if row.startswith(f"| {current_id} |")]
if len(current_rows) != 1 or "| IN_PROGRESS |" not in current_rows[0]:
    raise SystemExit("CONTINUITY_LEDGER_CURRENT_PHASE_MISMATCH")
if next_id != "NONE":
    next_rows = [row for row in rows if row.startswith(f"| {next_id} |")]
    if len(next_rows) != 1 or "| PLANNED |" not in next_rows[0]:
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
