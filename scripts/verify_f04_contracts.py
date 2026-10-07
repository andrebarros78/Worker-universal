from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]

def fail(message: str) -> None:
    raise SystemExit("F04_CONTRACT_FAIL=" + message)

def load(rel: str) -> Any:
    return json.loads((ROOT / rel).read_text(encoding="utf-8"))

def type_ok(value: Any, expected: str) -> bool:
    return {
        "object": isinstance(value, dict),
        "array": isinstance(value, list),
        "string": isinstance(value, str),
        "integer": isinstance(value, int) and not isinstance(value, bool),
        "number": isinstance(value, (int, float)) and not isinstance(value, bool),
        "boolean": isinstance(value, bool),
        "null": value is None,
    }[expected]

def validate(value: Any, schema: dict[str, Any], path: str = "$") -> None:
    if "const" in schema and value != schema["const"]:
        fail(path + ":const")
    if "enum" in schema and value not in schema["enum"]:
        fail(path + ":enum")
    if "type" in schema:
        types = schema["type"] if isinstance(schema["type"], list) else [schema["type"]]
        if not any(type_ok(value, item) for item in types):
            fail(path + ":type")
    if isinstance(value, dict):
        props = schema.get("properties", {})
        for key in schema.get("required", []):
            if key not in value:
                fail(path + ":missing:" + key)
        if schema.get("additionalProperties") is False:
            unknown = set(value) - set(props)
            if unknown:
                fail(path + ":unknown:" + ",".join(sorted(unknown)))
        for key, child in value.items():
            if key in props:
                validate(child, props[key], path + "." + key)
    if isinstance(value, list):
        if len(value) < schema.get("minItems", 0):
            fail(path + ":minItems")
        if "items" in schema:
            for index, item in enumerate(value):
                validate(item, schema["items"], f"{path}[{index}]")
    if isinstance(value, str):
        if len(value) < schema.get("minLength", 0):
            fail(path + ":minLength")
        pattern = schema.get("pattern")
        if pattern and re.fullmatch(pattern, value) is None:
            fail(path + ":pattern")
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        if "minimum" in schema and value < schema["minimum"]:
            fail(path + ":minimum")
        if "maximum" in schema and value > schema["maximum"]:
            fail(path + ":maximum")

pairs = [
    ("contracts/capability-descriptor.schema.json", "contracts/fixtures/capability.browser.playwright.json"),
    ("contracts/adapter-manifest.schema.json", "contracts/fixtures/adapter.browser.playwright.json"),
    ("contracts/computer-worker.schema.json", "contracts/fixtures/computer.windows.isolated.json"),
    ("contracts/browser-mission.schema.json", "contracts/fixtures/browser.mission.synthetic.json"),
    ("contracts/browser-result.schema.json", "contracts/fixtures/browser.result.synthetic.json"),
]

for schema_rel, fixture_rel in pairs:
    validate(load(fixture_rel), load(schema_rel))

print(f"F04_CONTRACTS_OK fixtures={len(pairs)} browser=playwright computer=isolated_windows_session")
