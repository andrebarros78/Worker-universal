from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]


def fail(message: str) -> None:
    raise SystemExit("F02_CONTRACT_FAIL=" + message)


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
        fail(f"{path}:const")

    if "enum" in schema and value not in schema["enum"]:
        fail(f"{path}:enum")

    if "type" in schema:
        expected = schema["type"]
        types = expected if isinstance(expected, list) else [expected]
        if not any(type_ok(value, item) for item in types):
            fail(f"{path}:type")

    if isinstance(value, dict):
        required = schema.get("required", [])
        for key in required:
            if key not in value:
                fail(f"{path}:missing:{key}")
        properties = schema.get("properties", {})
        if schema.get("additionalProperties") is False:
            unknown = set(value) - set(properties)
            if unknown:
                fail(f"{path}:unknown:{','.join(sorted(unknown))}")
        for key, child in value.items():
            if key in properties:
                validate(child, properties[key], f"{path}.{key}")

    if isinstance(value, list):
        if len(value) < schema.get("minItems", 0):
            fail(f"{path}:minItems")
        item_schema = schema.get("items")
        if item_schema:
            for index, item in enumerate(value):
                validate(item, item_schema, f"{path}[{index}]")

    if isinstance(value, str):
        if len(value) < schema.get("minLength", 0):
            fail(f"{path}:minLength")
        pattern = schema.get("pattern")
        if pattern and re.fullmatch(pattern, value) is None:
            fail(f"{path}:pattern")

    if isinstance(value, (int, float)) and not isinstance(value, bool):
        if "minimum" in schema and value < schema["minimum"]:
            fail(f"{path}:minimum")
        if "maximum" in schema and value > schema["maximum"]:
            fail(f"{path}:maximum")


pairs = [
    ("contracts/capability-descriptor.schema.json", "contracts/fixtures/capability.local.echo.json"),
    ("contracts/adapter-manifest.schema.json", "contracts/fixtures/adapter.local.echo.json"),
    ("contracts/tool-invocation.schema.json", "contracts/fixtures/invocation.local.echo.json"),
    ("contracts/tool-result.schema.json", "contracts/fixtures/result.local.echo.json"),
]

for schema_rel, fixture_rel in pairs:
    schema = load(schema_rel)
    fixture = load(fixture_rel)
    validate(fixture, schema)

result_schema = load("contracts/tool-result.schema.json")
serialized = json.dumps(result_schema, sort_keys=True).lower()
for forbidden in ("secret_value", "secret_material", "credential_value", "password"):
    if forbidden in serialized:
        fail("result_schema_contains_secret_material_field:" + forbidden)

forbidden_fixture = json.dumps(load("contracts/fixtures/result.local.echo.json"), sort_keys=True).lower()
if "secret://" in forbidden_fixture or "super-secret" in forbidden_fixture:
    fail("result_fixture_leaks_secret_reference_or_material")

sdk_files = {
    "python": ROOT / "adapter-sdk/python/sdk.py",
    "typescript": ROOT / "adapter-sdk/typescript/sdk.ts",
    "go": ROOT / "adapter-sdk/go/sdk.go",
}
for language, path in sdk_files.items():
    if not path.is_file():
        fail("missing_sdk:" + language)
    text = path.read_text(encoding="utf-8").lower()
    for required in ("manifest", "invoke", "health"):
        if required not in text:
            fail(f"sdk_missing_surface:{language}:{required}")

print(
    "F02_CONTRACTS_OK "
    f"fixtures={len(pairs)} sdk_languages={len(sdk_files)} "
    "secret_result_surface=clean"
)
