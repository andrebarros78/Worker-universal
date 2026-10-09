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

F09 initial governed simulation-only platform adapters are closed and proven in docs/PROOF_F09_PLATFORM_ADAPTERS.md. No live OCTA/99Freelas production integration is claimed.

F10 Economic Controller + Scheduler is CLOSED; proof: docs/PROOF_F10_ECONOMIC_CONTROLLER_SCHEDULER.md. Monetary observations are not externally bank-reconciled.

F11 Availability + Concurrency Hardening is CLOSED and proven under docs/PROOF_F11_AVAILABILITY_CONCURRENCY_HARDENING.md.

F12 Security/Formal/Operational Hardening is closed under docs/PROOF_F12_SECURITY_FORMAL_OPERATIONAL_HARDENING.md; GNATprove formal proof remains unavailable.

Current construction lane: F13 PERSONAL_PRODUCTION_V1_0 — terminal acceptance still pending.

Always confirm the live pointer in `state/EXECUTION_STATE.json`.

F13 technical candidate: production-rust v1.0.0-rc.1, engineering-only. Gate: scripts/verify_f13_engineering.ps1. Full F13 acceptance remains IN_PROGRESS; external F09 adapter mode and revenue proof are blocked. See docs/F13_OPERATIONAL_RUNBOOK.md and docs/F13_ACCEPTANCE_BLOCKERS.md.

F13 authority split: capability reports technical availability; engineering builds missing actions; decision agents only propose; the operator chooses and authorizes. Contract and tests in operator-control/, proof in docs/PROOF_F13_ROLE_SEPARATION.md. This does not authorize platform actions or establish MISSION_PROVEN.
