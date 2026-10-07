# AGENTS.md — mandatory instructions

Scope: entire repository `D:\TIMED-MISSION-AGENT`.

Before changing code, read `00_START_HERE.md` and follow its read order.

## Invariants

- One canonical root only.
- One canonical mission state only.
- Rust owns deterministic mission state, deadline/SLA and result acceptance.
- No vendor, platform, LLM provider, API, MCP server or browser implementation belongs in the Rust domain core.
- External capabilities enter through versioned adapters/contracts.
- Go owns worker/process supervision, not business truth.
- Browser/Computer, OCR/AI, API/MCP and platform integrations are replaceable workers/adapters.
- Validation must be independent from execution where feasible.
- Evidence is required before terminal success.
- Idempotency and fencing are mandatory before real multi-worker submission.
- Never claim SPARK proof unless GNATprove actually ran successfully.
- Preserve unrelated work. Do not clean or rewrite other projects.
- A CLOSED phase is immutable; remediation uses a new phase ID.

## Required engineering loop

DISCOVER → RECONCILE → PLAN → IMPLEMENT → BUILD → TEST → DIAGNOSE → CORRECT → RETEST → INTEGRATE → PROVE → COMMIT → ADVANCE STATE.

## Continuity

Always update together when a phase changes:

- `state/EXECUTION_STATE.json`
- `docs/CANONICAL_STATE.md`
- `docs/PHASE_LEDGER.md`
- proof artifact for the phase

Do not advance `current_phase` until the current phase exit gate passes.
