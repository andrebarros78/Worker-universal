from pathlib import Path
import re
root=Path(__file__).resolve().parents[1]
concept=(root/"docs/F09_PLATFORM_ADAPTERS_MAP.md").read_text(encoding="utf-8")
matrix=(root/"docs/F09_IMPLEMENTATION_MATRIX.md").read_text(encoding="utf-8")
expected={f"PAD-{i:03d}" for i in range(1,51)}
found=set(re.findall(r"PAD-\d{3}",concept))
mapped=set()
for m in re.finditer(r"PAD-(\d{3})(?:\.\.(\d{3}))?",matrix):
 for i in range(int(m.group(1)),int(m.group(2) or m.group(1))+1):
  mapped.add(f"PAD-{i:03d}")
if found!=expected or mapped!=expected:
 raise SystemExit(f"F09_MAP_FAIL missing={sorted(expected-found)} unmapped={sorted(expected-mapped)}")
print("F09_MAP_OK requirements=50 mapped=50")
