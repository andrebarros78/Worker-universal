from pathlib import Path
import json
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
state=json.loads((ROOT/'state/EXECUTION_STATE.json').read_text(encoding='utf-8'))
if state['current_phase']['id']!='F13' or state['current_phase']['status']!='IN_PROGRESS':
    raise SystemExit('F13_SPLIT_FAIL_INVALID_CURRENT_PHASE')
if state.get('next_phase') is not None:
    raise SystemExit('F13_SPLIT_FAIL_TERMINAL_NEXT_NOT_NONE')
if state['terminal_phase']['terminal_label']!='MISSION_PROVEN':
    raise SystemExit('F13_SPLIT_FAIL_TERMINAL_LABEL')
separation=state.get('f13_separation',{})
if separation.get('engineering_candidate',{}).get('status')!='COMPLETE':
    raise SystemExit('F13_SPLIT_FAIL_ENGINEERING_NOT_COMPLETE')
if separation.get('productive_homologation',{}).get('status')!='PENDING':
    raise SystemExit('F13_SPLIT_FAIL_HOMOLOGATION_NOT_PENDING')
if separation.get('productive_homologation',{}).get('mission_proven') is not False:
    raise SystemExit('F13_SPLIT_FAIL_MISSION_PROVEN_NOT_FALSE')
if 'F13_ENGINEERING_VERIFY=PASS' not in separation.get('engineering_candidate',{}).get('evidence',[]):
    raise SystemExit('F13_SPLIT_FAIL_ENGINEERING_EVIDENCE_MISSING')
if 'F13_FINAL_ACCEPTANCE=BLOCKED' not in separation.get('productive_homologation',{}).get('blocking_gate',''):
    raise SystemExit('F13_SPLIT_FAIL_BLOCKING_GATE_MISSING')
required_docs=[
 'docs/F13_ENGINEERING_VS_PRODUCTIVE_HOMOLOGATION.md',
 'docs/F13_ROLE_SEPARATION.md',
 'docs/F13_ACCEPTANCE_BLOCKERS.md',
 'docs/PROOF_F13_ENGINEERING_CANDIDATE.md',
 'docs/PROOF_F13_ROLE_SEPARATION.md',
]
for rel in required_docs:
    if not (ROOT/rel).is_file():
        raise SystemExit('F13_SPLIT_FAIL_MISSING_DOC:'+rel)
# Acceptance gate must remain fail-closed until productive proof exists.
result=subprocess.run([str(ROOT/'.venv/Scripts/python.exe'),str(ROOT/'scripts/verify_f13_acceptance.py')],capture_output=True,text=True,timeout=20)
if result.returncode!=3 or 'F13_FINAL_ACCEPTANCE=BLOCKED' not in result.stdout:
    raise SystemExit('F13_SPLIT_FAIL_ACCEPTANCE_NOT_BLOCKED')
print('F13_ENGINEERING_HOMOLOGATION_SPLIT=PASS engineering=COMPLETE productive_homologation=PENDING mission_proven=false')
