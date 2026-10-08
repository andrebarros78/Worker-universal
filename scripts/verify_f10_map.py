from pathlib import Path
import re

root=Path(__file__).resolve().parents[1]
concept=(root/"docs/F10_ECONOMIC_CONTROLLER_MAP.md").read_text(encoding="utf-8")
matrix=(root/"docs/F10_IMPLEMENTATION_MATRIX.md").read_text(encoding="utf-8")
expected={f"ECO-{i:03d}" for i in range(1,56)}
found=set(re.findall(r"ECO-\d{3}",concept))
mapped=set()
for match in re.finditer(r"ECO-(\d{3})(?:\.\.(\d{3}))?",matrix):
 for i in range(int(match.group(1)),int(match.group(2) or match.group(1))+1):
  mapped.add(f"ECO-{i:03d}")
if found!=expected or mapped!=expected:
 raise SystemExit(f"F10_MAP_FAIL concept_missing={sorted(expected-found)} mapped_missing={sorted(expected-mapped)}")
print("F10_MAP_OK requirements=55 mapped=55")
