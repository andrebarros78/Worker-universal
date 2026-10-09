# F13 — Engineering Candidate Operation and Final Acceptance

Canonical machine: PC Vendas. Root: D:\TIMED-MISSION-AGENT.
Current phase: F13 PERSONAL_PRODUCTION_V1_0 IN_PROGRESS, no next phase.

## Verified engineering path

From the canonical project root execute:

    powershell -File scripts/verify_f13_engineering.ps1

This builds the Rust production integrator and runs tests, local CLI, explicit disk artifact verification, F02 permission-gate denial, and duplicate-target rejection. The harness uses a disposable directory in the PC Vendas dedicated SentinelX workspace.

To run an explicitly local exercise, build production-rust with CARGO_TARGET_DIR set to C:\ProgramData\SentinelX\workspace\tma-build\production-rust, then execute the tma-production.exe binary:

    tma-production.exe rehearsal <NEW_DIRECTORY> <EXPLICIT_LOCAL_ASCII_INPUT>

The destination's parent must exist and the destination must be new. Never point it at an existing project directory, historical task state, or paid-provider workflow.

The CLI returns three proof markers:
- F13_LOCAL_E2E=PASS
- F13_PRODUCTION_GATE=BLOCKED
- F13_MISSION_PROVEN=NOT_YET

A local result file is physically written and fsynced. The F05 state is re-opened after a synthetic worker crash, the expired lease is reconciled, stale results denied, a new fencing token acquired, and F07 independently validates the evidence. F10 records only a synthetic computation cost; **no actual payment or invoice is asserted**.

## True production acceptance — separate authority

Before closing F13, provide a legitimate target workflow and authorization for external effects, plus proof of the platform's current terms and required scopes. The adapter must progress through F08 qualification to F02 Production only with real benchmark evidence and applicable approvals. F09's safety policy must not be bypassed or edited to pretend authorization.

Run a real eligible task, save verifiable source/output and external completion receipt, respect deadlines and non-replayable actions, demonstrate crash recovery without duplicate submission, validate independently via F07, reconcile actual costs/revenue in F10 with payment evidence where applicable, verify backups and audit integrity in F12 and execute a comprehensive end-to-end acceptance gate.

**Never mark MISSION_PROVEN based solely on the synthetic local rehearsal, estimates or a successful code compilation.**

## Blockers presently known

- F09 platform adapters explicitly reject non-simulation mode and non-loopback interactions.
- No independently verifiable paid task receipt and no real settlement evidence.
- Formal GNATprove unavailable; do not claim SPARK formal proof.
- Live forced-stop plus backup restore drill was previously blocked; no full production failover proof.
- User-approved productive platform integration and qualification evidence are necessary for final production acceptance.

## Post-candidate operation

Maintain F13 IN_PROGRESS; preserve phase proof history and Git candidate tag. Do not create fictitious F14 or declare the terminal MISSION_PROVEN state before actual sign-off.

Fail-closed production acceptance check: python scripts/verify_f13_acceptance.py. An exit code of 3 is expected until a separately implemented and independently verified productive workflow is available. Do not add this intentionally failing gate to the engineering-only baseline.
