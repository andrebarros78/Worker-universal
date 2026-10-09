"""Fail-closed F13 production-acceptance gate.

Synthetic receipts and engineering fixtures do not satisfy real platform proof.
A genuine production receipt validator must be implemented and independently
approved before this gate may ever return success.
"""
from pathlib import Path
import json
import sys
root=Path(__file__).resolve().parents[1]
state=json.loads((root/"state/EXECUTION_STATE.json").read_text(encoding="utf-8"))
if state["current_phase"]["id"]!="F13":
    print("F13_FINAL_ACCEPTANCE=DENIED INVALID_CANONICAL_PHASE")
    sys.exit(3)
policy=(root/"platform-adapters/src/policy.ts").read_text(encoding="utf-8")
if 'F09_POLICY_LIVE_NOT_AUTHORIZED' not in policy:
    print("F13_FINAL_ACCEPTANCE=DENIED ADAPTER_POLICY_CHANGED_REVIEW_REQUIRED")
    sys.exit(3)
if state.get("f13_candidate",{}).get("mission_proven") is True:
    print("F13_FINAL_ACCEPTANCE=DENIED UNVERIFIED_TERMINAL_ASSERTION")
    sys.exit(3)
print("F13_FINAL_ACCEPTANCE=BLOCKED")
print("REQUIRED: authorized qualified external adapter, real completion evidence, F07 validation, verified economics, productive recovery and independent F13 sign-off")
print("MISSION_PROVEN=PENDING")
sys.exit(3)
