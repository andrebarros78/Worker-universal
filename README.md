# TIMED MISSION AGENT

> New AI/chat/operator: read `00_START_HERE.md` before any implementation. The machine-readable continuation pointer is `state/EXECUTION_STATE.json`.

Canonical root: `D:\TIMED-MISSION-AGENT`

Personal autonomous work-execution platform for timed microtasks and future multi-step missions.

## Product objective

Generate dependable personal income by converting eligible automatable work into completed, validated missions with minimal operator burden. The engineering target is reliable useful work: hard SLA enforcement, independent validation, recovery from technical failures, durable evidence and low maintenance cost.

## Polyglot baseline

- **Rust** — deterministic mission authority, deadlines, state transitions and invariant gates.
- **Go** — worker supervision, restart policy, process lifecycle and infrastructure concurrency.
- **TypeScript / Node** — isolated browser/web worker contract with per-step timeout.
- **Python** — OCR/vision/document intelligence and current invoice benchmark core.
- **Elixir / Erlang OTP** — long-lived fault-tolerant supervision and future distributed coordination.
- **Ada / SPARK** — compact formal invariants; source is present, formal proof waits for GNATprove.
- **JSON Schema** — versioned mission/result contracts across runtimes.

## One-command verification

```powershell
powershell -ExecutionPolicy Bypass -File D:\TIMED-MISSION-AGENT\scripts\verify_baseline.ps1
```

The gate fails if a verified runtime layer fails. SPARK is reported separately until its proof toolchain exists.

## Canonical documents

- `PROJECT_DNA.md` — sovereign objective and operating rules.
- `docs/POLYGLOT_ARCHITECTURE.md` — technology ownership and failure containment.
- `docs/BUILD_BASELINE.md` — construction proof.
- `docs/ROADMAP.md` — staged path to production.
- `docs/CANONICAL_STATE.md` — exact continuation point.

Do not create a parallel root. Continue from Git history and `docs/CANONICAL_STATE.md`.

## Current canonical phase

F06 Planner/Router/Knowledge is closed and proven under `docs/PROOF_F06_PLANNER_ROUTER_KNOWLEDGE.md`.

Current construction lane: `F07 INDEPENDENT_VALIDATION_RECOVERY`.

Always confirm the live pointer in `state/EXECUTION_STATE.json`.
