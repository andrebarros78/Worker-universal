"""Independently verify F13 synthetic rehearsal artifacts on disk."""
import hashlib,json,sqlite3,sys
from pathlib import Path

if len(sys.argv)!=3:
    raise SystemExit("usage: verify_f13_artifact.py <rehearsal-dir> <fixture>")
root=Path(sys.argv[1])
source=Path(sys.argv[2]).read_bytes()
output=(root/"result.txt").read_bytes()
expected=source.upper()
if output!=expected:raise SystemExit("F13_EVIDENCE_FAIL transformation_mismatch")
digest=hashlib.sha256(output).hexdigest()
with sqlite3.connect(f"file:{(root/'missions.sqlite3').as_posix()}?mode=ro",uri=True) as c:
    row=c.execute("SELECT state,terminal_result_hash,fence_seq FROM missions WHERE mission_id='f13-local-rehearsal'").fetchone()
    events=c.execute("SELECT COUNT(*) FROM mission_events WHERE mission_id='f13-local-rehearsal'").fetchone()[0]
    snapshots=c.execute("SELECT COUNT(*) FROM mission_snapshots WHERE mission_id='f13-local-rehearsal'").fetchone()[0]
    validated=c.execute("SELECT COUNT(*) FROM validation_receipts WHERE mission_id='f13-local-rehearsal' AND accepted=1").fetchone()[0]
    db_ok=c.execute("PRAGMA integrity_check").fetchone()[0]
if row!=( "succeeded",digest,2) or events<10 or snapshots<3 or validated!=1 or db_ok!="ok":
    raise SystemExit(f"F13_EVIDENCE_FAIL F05_state_or_receipt_invalid {row}")
with sqlite3.connect(f"file:{(root/'cost.sqlite3').as_posix()}?mode=ro",uri=True) as c:
    income=c.execute("SELECT kind,COALESCE(SUM(amount_cents),0) FROM economic_events GROUP BY kind").fetchall()
    ec_ok=c.execute("PRAGMA integrity_check").fetchone()[0]
if income!=[("cost",1)] or ec_ok!="ok":
    raise SystemExit("F13_EVIDENCE_FAIL economic_real_revenue_fabricated")
print(f"F13_ARTIFACT_VERIFY=PASS digest={digest} f05_events={events} snapshots={snapshots} independent_validation=1 fencing=2 synthetic_cost_cents=1 recorded_revenue_cents=0")
