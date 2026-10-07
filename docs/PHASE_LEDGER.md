# Phase Ledger

Status vocabulary: PLANNED | IN_PROGRESS | BLOCKED_EXTERNAL | CLOSED.

| ID | Release target | Phase | Status | Depends on | Exit evidence |
|---|---|---|---|---|---|
| F00 | v0.2.0 | Polyglot deterministic baseline | CLOSED | — | PROOF_POLYGLOT_V0_2.md |
| F01 | v0.2.1 | Concept + executive continuity baseline | CLOSED | F00 | PROOF_F01_EXECUTIVE_CONTINUITY.md |
| F02 | v0.2.2 | Capability foundation | CLOSED | F01 | PROOF_F02_CAPABILITY_FOUNDATION.md |
| F03 | v0.3.x | Vision/OCR & document intelligence | IN_PROGRESS | F02 | invoice corpus benchmark + calibrated confidence |
| F04 | v0.4.x | Isolated Browser/Computer Worker | PLANNED | F02 | browser sandbox + DOM/form/download/upload + evidence + recovery |
| F05 | v0.5.x | Durable mission runtime | PLANNED | F02,F04 | queue + idempotency + leases/fencing + reboot/restart recovery |
| F06 | v0.5.x | Planner/Router/Knowledge | PLANNED | F02,F03,F04,F05 | capability routing + deterministic fallbacks + research contracts |
| F07 | v0.5.x | Independent Validation + Recovery | PLANNED | F03,F04,F05,F06 | validator proofs + failure taxonomy + recovery matrix |
| F08 | v0.6.x | Training/Benchmark/Qualification gates | PLANNED | F03,F04,F07 | timed benchmark harness + pre/post registration state machines |
| F09 | v0.6.x | Platform adapters | PLANNED | F08 | generic adapters + initial platform adapters with isolation tests |
| F10 | v0.7.x | Economic Controller + Scheduler | PLANNED | F05,F09 | cost/revenue ledger + expected-value scheduling |
| F11 | v0.8.x | Availability + concurrency hardening | PLANNED | F05,F07,F10 | 1→4→8→16 concurrency + chaos/soak/restart tests |
| F12 | v0.9.x | Security/formal/operational hardening | PLANNED | F11 | secret isolation + audit + GNATprove when available + recovery drills |
| F13 | v1.0.0 | Personal production release | PLANNED | F12 | end-to-end production proof + clean runbook + MISSION_PROVEN |

## Current authority

Exactly one phase is active:

`F03 — VISION_OCR_DOCUMENT_INTELLIGENCE — IN_PROGRESS`

Do not begin F04 as the main lane until F03 exit gate is proven and F03 is CLOSED.
