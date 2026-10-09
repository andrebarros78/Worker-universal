from pathlib import Path
import re
root=Path(__file__).resolve().parents[1]
text=(root/"docs/F13_ENGINEERING_MAP.md").read_text(encoding="utf-8")
expected={f"PRD-{n:03}" for n in range(1,31)}
found=set(re.findall(r"PRD-\d{3}",text))
if found!=expected:raise SystemExit(f"F13_MAP_FAIL missing={sorted(expected-found)} extra={sorted(found-expected)}")
print("F13_MAP_OK requirements=30 mapped=30")
