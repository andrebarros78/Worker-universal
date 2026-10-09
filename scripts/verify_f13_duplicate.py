"""Verify fail-closed duplicate-output rejection without a shell stderr exception."""
import subprocess,sys
from pathlib import Path
if len(sys.argv)!=4:raise SystemExit(2)
exe,destination,source=map(Path,sys.argv[1:])
if not destination.is_dir() or not (destination/"result.txt").is_file():
    raise SystemExit("F13_DUPLICATE_TARGET=FAIL fixture_missing")
before=(destination/"result.txt").read_bytes()
res=subprocess.run([str(exe),"rehearsal",str(destination),str(source)],
    capture_output=True,text=True,timeout=20)
after=(destination/"result.txt").read_bytes()
if res.returncode!=3 or "TARGET_EXISTS" not in res.stderr or after!=before:
    raise SystemExit("F13_DUPLICATE_TARGET=FAIL")
print("F13_DUPLICATE_TARGET=DENIED output_untouched=true")
