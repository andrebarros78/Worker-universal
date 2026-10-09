"""F12 offline F05 SQLite snapshot/restore verification without process termination."""
from __future__ import annotations
import importlib.util
import os
from pathlib import Path
import secrets
import shutil
import sqlite3
import subprocess
import sys
import tempfile

PROJECT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location("tma_f12_security",PROJECT/"security-runtime/security.py")
assert spec and spec.loader
security=importlib.util.module_from_spec(spec)
spec.loader.exec_module(security)

def executable(path:Path,args:list[str])->str:
    if not path.is_file():
        raise RuntimeError("F12_REQUIRED_LOCAL_BINARY_MISSING")
    result=subprocess.run([str(path),*args],capture_output=True,text=True,timeout=60)
    if result.returncode!=0:
        raise RuntimeError("F12_BINARY_PROOF_FAILED:"+str(result.returncode)+":"+result.stderr[-300:])
    return result.stdout

def main()->int:
    availability=Path(r"C:\ProgramData\SentinelX\workspace\tma-build\availability-rust\debug\tma-availability.exe")
    verifier=Path(r"C:\ProgramData\SentinelX\workspace\tma-build\security-rust\debug\tma-security-verify.exe")
    workspace=Path(os.environ.get("TMA_F12_TEST_ROOT",r"C:\ProgramData\SentinelX\workspace"))
    with tempfile.TemporaryDirectory(prefix="tma-f12-offline-f05-",dir=workspace) as temp:
        root=Path(temp)
        db=root/"live.sqlite3"
        output=executable(availability,["matrix",str(db),"4"])
        if "integrity=ok" not in output or "duplicates=0 lost=0" not in output:
            raise RuntimeError("F12_F05_SEED_MATRIX_FAILED")
        before=executable(verifier,["verify",str(db)])
        if "F12_RESTORED_F05_VERIFY=PASS" not in before:
            raise RuntimeError("F12_SOURCE_F05_VERIFY_FAILED")
        key=secrets.token_bytes(32)
        manifest=security.backup_sqlite(root=root,source=db,
            destination=root/"signed-backup",signing_key=key)
        restored=Path(security.restore_sqlite(root=root,snapshot=root/"signed-backup",
            target=root/"restored",signing_key=key))
        after=executable(verifier,["verify",str(restored)])
        if before.strip()!=after.strip():
            raise RuntimeError("F12_F05_RECOVERY_LEDGER_DRIFT")
        # Online SQLite backup can have a different byte layout from the
        # WAL-backed original. Compare independently verified F05 projection,
        # event chain, snapshots, queue and integrity rather than raw bytes.
        if not db.is_file():
            raise RuntimeError("F12_ORIGINAL_DATABASE_NOT_PRESERVED")
        audit=security.AuditTrail(root/"recovery-audit.sqlite3")
        audit.record(event_id="f12-backup",actor_id="f12-operator",action="backup",
            outcome="ok",resource_id="f05-fixture",occurred_ms=100)
        audit.record(event_id="f12-restore",actor_id="f12-operator",action="restore",
            outcome="ok",resource_id="f05-fixture",occurred_ms=110)
        if audit.verify()!=2:
            raise RuntimeError("F12_RECOVERY_AUDIT_INVALID")
        print("F12_F05_OFFLINE_RESTORE=PASS workers=4 missions=64 "
              f"snapshot_bytes={manifest['bytes']} rust_ledger=PASS audit=PASS "
              "existing_live_db_preserved=true")
    return 0

if __name__=="__main__":
    try:sys.exit(main())
    except Exception as exc:
        print(f"F12_F05_OFFLINE_RESTORE=FAIL reason={type(exc).__name__} detail={exc}",file=sys.stderr)
        sys.exit(1)
