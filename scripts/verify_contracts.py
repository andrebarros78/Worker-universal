from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MISSION = json.loads((ROOT / "contracts" / "mission-envelope.schema.json").read_text(encoding="utf-8"))
RESULT = json.loads((ROOT / "contracts" / "result-envelope.schema.json").read_text(encoding="utf-8"))

assert MISSION["properties"]["schema_version"]["const"] == 1
assert RESULT["properties"]["schema_version"]["const"] == 1
assert MISSION["properties"]["deadline_ms"]["minimum"] == 1
statuses = set(RESULT["properties"]["status"]["enum"])
required_statuses = {"succeeded", "failed", "deadline_exceeded"}
assert required_statuses <= statuses
print("CONTRACTS_OK schema_version=1 statuses=" + ",".join(sorted(statuses)))
