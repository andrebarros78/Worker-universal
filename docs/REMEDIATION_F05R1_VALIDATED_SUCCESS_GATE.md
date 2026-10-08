# Remediation R05-01 — Validated Success Gate

Status: CLOSED
Discovered during: F07
References: F05 DURABLE_MISSION_RUNTIME

## Finding

F05 correctly enforced leases, fencing, idempotency and terminal-state legality, but a lease owner could submit a Succeeded result directly after entering Validating. That allowed executor-declared success to become durable mission success without an independent validation receipt.

## Correction

- add append-only validation_receipts to the F05 SQLite store;
- require validator identity to differ from the current mission lease owner;
- bind each accepted receipt to mission_id, payload hash and evidence digest;
- require an accepted matching validation receipt before ResultSubmission can transition to Succeeded;
- preserve direct Failed/DeadlineExceeded terminalization.

## Proof

Observed:

- direct Succeeded without receipt rejected with validation_receipt_required;
- receipt from current lease owner rejected;
- wrong payload hash receipt rejected with validation_receipt_invalid;
- accepted independent receipt authorizes exactly the bound success payload;
- validation receipt UPDATE/DELETE rejected by append-only triggers;
- tma-core current suite: 21 PASS / 0 FAIL;
- F05_VERIFY=PASS after the remediation;
- F07 end-to-end validation/finalization PASS;
- full BASELINE_VERIFY=PASS.

This remediation does not reopen F05.
