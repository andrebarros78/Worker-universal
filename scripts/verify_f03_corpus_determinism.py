from __future__ import annotations

import hashlib
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / "benchmarks" / "f03_corpus"
PYTHON = ROOT / ".venv" / "Scripts" / "python.exe"


def snapshot() -> dict[str, str]:
    result: dict[str, str] = {}
    for path in sorted(CORPUS.glob("*")):
        if path.is_file():
            result[path.name] = hashlib.sha256(path.read_bytes()).hexdigest()
    return result


before = snapshot()
subprocess.run(
    [str(PYTHON), str(ROOT / "scripts" / "generate_f03_corpus.py")],
    cwd=ROOT,
    check=True,
)
after = snapshot()

if before != after:
    added = sorted(set(after) - set(before))
    removed = sorted(set(before) - set(after))
    changed = sorted(name for name in before.keys() & after.keys() if before[name] != after[name])
    raise SystemExit(
        f"F03_CORPUS_NONDETERMINISTIC added={added} removed={removed} changed={changed}"
    )

print(f"F03_CORPUS_DETERMINISTIC files={len(after)}")
