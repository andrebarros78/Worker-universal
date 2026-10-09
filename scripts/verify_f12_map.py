from pathlib import Path
import re
root=Path(__file__).resolve().parents[1]
concept=(root/"docs/F12_SECURITY_FORMAL_OPERATIONAL_MAP.md").read_text(encoding="utf-8")
matrix=(root/"docs/F12_IMPLEMENTATION_MATRIX.md").read_text(encoding="utf-8")
expected={f"SEC-{n:03}" for n in range(1,61)}
found=set(re.findall(r"SEC-\d{3}",concept))
mapped=set()
for m in re.finditer(r"SEC-(\d{3})(?:\.\.(\d{3}))?",matrix):
    mapped.update(f"SEC-{n:03}" for n in range(int(m.group(1)),int(m.group(2) or m.group(1))+1))
if expected!=found or expected!=mapped:
    raise SystemExit(f"F12_MAP_FAIL missing={sorted(expected-found)} unmapped={sorted(expected-mapped)}")
print("F12_MAP_OK requirements=60 mapped=60")
