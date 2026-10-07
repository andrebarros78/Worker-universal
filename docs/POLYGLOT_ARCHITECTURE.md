# Polyglot architecture baseline v0.2

## Authority by component

| Layer | Technology | Authority |
|---|---|---|
| Deterministic core | Rust | mission state, SLA/deadline, confidence gate, invariants |
| Runtime supervisor | Go | process lifecycle, restart budget, health, worker orchestration |
| Web worker | TypeScript / Node | page observation/action adapter and per-step timeout containment |
| AI/OCR adapter | Python | document perception, OCR, field extraction, model experimentation |
| Availability layer | Elixir / Erlang OTP | fault-tolerant long-lived supervision and future distributed coordination |
| Formal invariant module | Ada / SPARK | small mathematically checkable invariants; not a general application layer |
| Contracts | JSON Schema | single versioned inter-process contract |

## Failure containment

Python/OCR failure -> Go restarts adapter -> Rust keeps mission authority.
Web worker failure -> Go replaces worker -> Rust rejects stale/late result.
Go supervisor failure -> Elixir/OTP can supervise the long-lived service in a later deployment mode.
Machine restart -> persistent ledger/state is reconciled before a mission is resumed.
Late result -> Rust deadline gate rejects it even if another component says success.

## Deliberate non-goals

Do not duplicate mission truth in every language. Rust remains the deterministic authority. Other layers are replaceable workers.
