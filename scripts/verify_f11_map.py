from pathlib import Path
import re
root=Path(__file__).resolve().parents[1]
p=(root/"docs/F11_AVAILABILITY_CONCURRENCY_MAP.md").read_text(encoding="utf-8")
matrix=(root/"docs/F11_IMPLEMENTATION_MATRIX.md").read_text(encoding="utf-8")
expected={f"AVC-{i:03d}" for i in range(1,57)}
found=set(re.findall(r"AVC-\d{3}",p))
mapped=set()
for m in re.finditer(r"AVC-(\d{3})(?:\.\.(\d{3}))?",matrix):
    mapped.update(f"AVC-{i:03d}" for i in range(int(m.group(1)),int(m.group(2) or m.group(1))+1))
if found!=expected or mapped!=expected:raise SystemExit(f"F11_MAP_FAIL missing={sorted(expected-found)} unmapped={sorted(expected-mapped)}")
print("F11_MAP_OK requirements=56 mapped=56")
