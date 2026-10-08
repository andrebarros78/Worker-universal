# Phase Ledger

Status vocabulary: PLANNED | IN_PROGRESS | BLOCKED_EXTERNAL | CLOSED.

| ID | Release target | Phase | Status | Depends on | Exit evidence |
|---|---|---|---|---|---|
| F00 | v0.2.0 | Polyglot deterministic baseline | CLOSED | — | PROOF_POLYGLOT_V0_2.md |
| F01 | v0.2.1 | Concept + executive continuity baseline | CLOSED | F00 | PROOF_F01_EXECUTIVE_CONTINUITY.md |
| F02 | v0.2.2 | Capability foundation | CLOSED | F01 | PROOF_F02_CAPABILITY_FOUNDATION.md |
| F03 | v0.3.0 | Vision/OCR & document intelligence | CLOSED | F02 | PROOF_F03_VISION_OCR.md |
| F04 | v0.4.0 | Isolated Browser/Computer Worker | CLOSED | F02 | PROOF_F04_BROWSER_COMPUTER.md |
| F05 | v0.5.0 | Durable mission runtime | CLOSED | F02,F04 | PROOF_F05_DURABLE_RUNTIME.md |
| F06 | v0.5.1 | Planner/Router/Knowledge | CLOSED | F02,F03,F04,F05 | PROOF_F06_PLANNER_ROUTER_KNOWLEDGE.md |
| F07 | v0.5.x | Independent Validation + Recovery | IN_PROGRESS | F03,F04,F05,F06 | validator proofs + failure taxonomy + recovery matrix |
| F08 | v0.6.x | Training/Benchmark/Qualification gates | PLANNED | F03,F04,F07 | timed benchmark harness + pre/post registration state machines |
| F09 | v0.6.x | Platform adapters | PLANNED | F08 | generic adapters + initial platform adapters with isolation tests |
| F10 | v0.7.x | Economic Controller + Scheduler | PLANNED | F05,F09 | cost/revenue ledger + expected-value scheduling |
| F11 | v0.8.x | Availability + concurrency hardening | PLANNED | F05,F07,F10 | 1→4→8→16 concurrency + chaos/soak/restart tests |
| F12 | v0.9.x | Security/formal/operational hardening | PLANNED | F11 | secret isolation + audit + GNATprove when available + recovery drills |
| F13 | v1.0.0 | Personal production release | PLANNED | F12 | end-to-end production proof + clean runbook + MISSION_PROVEN |

## Remediation records

| ID | References | Status | Evidence |
|---|---|---|---|
| R03-01 | F03 | CLOSED | REMEDIATION_F03R1_REGRESSION_GATE_HYGIENE.md |
| R04-01 | F04 | CLOSED | REMEDIATION_F04R1_DEADLINE_CLASSIFICATION.md |

Remediation records do not reopen a closed phase and do not create a second active construction lane.

## Current authority

Exactly one phase is active:

F07 — INDEPENDENT_VALIDATION_RECOVERY — IN_PROGRESS

Do not begin F08 as the main lane until F07 exit gate is proven and F07 is CLOSED.
