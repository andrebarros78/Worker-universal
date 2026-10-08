from __future__ import annotations

import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
durable = (ROOT / "core-rust" / "src" / "durable.rs").read_text(encoding="utf-8").lower()
cargo = tomllib.loads((ROOT / "core-rust" / "Cargo.toml").read_text(encoding="utf-8"))

forbidden = (
    "redis",
    "rabbitmq",
    "kafka",
    "openai",
    "anthropic",
    "gemini",
    "playwright",
    "browser",
    "http://",
    "https://",
)
for token in forbidden:
    if token in durable:
        raise SystemExit("F05_PURITY_FAIL durable_token=" + token)

dependencies = set(cargo.get("dependencies", {}))
if dependencies != {"rusqlite", "sha2"}:
    raise SystemExit(f"F05_PURITY_FAIL dependencies={sorted(dependencies)}")

required = (
    "pragma journal_mode=wal",
    "pragma synchronous=full",
    "transactionbehavior::immediate",
    "mission_events_no_update",
    "mission_events_no_delete",
    "fencing_token",
    "idempotency_key",
    "verify_ledger",
)
for token in required:
    if token not in durable:
        raise SystemExit("F05_PURITY_FAIL missing=" + token)

print("F05_PURITY_OK storage=sqlite_wal broker_dependencies=0 provider_dependencies=0 transaction=immediate")
