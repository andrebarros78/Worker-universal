# TIMED-MISSION-AGENT — START HERE

This file is the canonical entry point for any new AI, chat, engineer or automation.

## Canonical root

`D:\TIMED-MISSION-AGENT`

Never create a parallel root, replacement repository or competing execution plan.

## Mandatory read order

1. `PROJECT_DNA.md`
2. `docs/PROJECT_CONCEPT.md`
3. `docs/EXECUTIVE_PROJECT.md`
4. `state/EXECUTION_STATE.json`
5. `docs/CANONICAL_STATE.md`
6. `docs/PHASE_LEDGER.md`
7. latest applicable proof under `docs/PROOF_*.md`

## Continuation rule

The machine-readable authority for "where to continue" is:

`state/EXECUTION_STATE.json`

The human-readable authority is:

`docs/CANONICAL_STATE.md`

If they disagree, stop mutation, reconcile the inconsistency from Git history and evidence, then update both in one commit.

## Mission completion rule

No phase is CLOSED without:

- required deliverables present;
- phase-specific tests passing;
- regression gate passing;
- evidence persisted;
- Git commit recorded;
- state pointer advanced to exactly one next phase.

Do not reopen a CLOSED phase. A correction becomes a new remediation phase referencing the closed phase.

## Project terminal state

The project reaches V1.0 when the universal Mission Worker can, within its configured operating rules:

receive mission → classify → plan → select capability → execute → observe → validate independently → recover → persist evidence → calculate economic result → close or surface a genuine external dependency.

The product must survive worker/process failure without losing canonical mission state.
