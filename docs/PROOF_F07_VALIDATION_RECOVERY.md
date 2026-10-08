# Proof — F07 Independent Validation and Recovery

Date: 2026-10-08
Host: PC Vendas
Canonical root: D:\TIMED-MISSION-AGENT

## Objective

Prove that executor-declared success is never equivalent to mission success, that evidence is independently validated, every canonical F02 failure class has an explicit recovery/terminal policy, and recovery is bounded by replay safety, retry budget and deadline.

## Architecture

F07 is implemented as a separate Rust crate:

validation-rust

Authority boundaries:

- executor output is untrusted input;
- tma-validation owns independent evidence validation and recovery decisions;
- tma-foundation remains canonical ErrorClass authority;
- tma-core remains mission-state/durable truth authority;
- tma-planner remains plan/route authority.

## Requirement map

- docs/F07_VALIDATION_RECOVERY_MAP.md
- VAL-001..VAL-055
- docs/F07_IMPLEMENTATION_MATRIX.md

Observed:

F07_MAP_OK requirements=55 mapped=55

## Independent validation

Implemented:

- ExecutorResult;
- EvidenceItem;
- ValidationContext;
- EvidenceCompleteness;
- ValidationOutcome;
- ValidationReport;
- EvidenceValidator;
- capability-specific ProofStrategy;
- stable evidence digest;
- deterministic validation fingerprint.

Proof strategies cover:

- Vision/Document: ocr_text + field_confidence;
- Browser: dom + screenshot;
- Research: sources;
- Computer: screenshot;
- deterministic/API/MCP/Storage/Notification: result evidence.

Observed focused tests:

19 PASS / 0 FAIL

Critical false-success proofs:

- declared success with missing required evidence is rejected;
- invalid evidence hash/zero-byte evidence is rejected;
- validator identity equal to executor identity is rejected;
- declared failure is never promoted to Proven;
- low confidence is rejected independently;
- expired deadline preempts declared success;
- extra required evidence is enforced.

## R05-01 — Validated Success Gate

Finding:

F05 allowed a fenced executor in Validating to submit Succeeded directly.

Correction:

- tma-core version advanced to 0.5.2;
- append-only validation_receipts table added;
- validation receipt UPDATE/DELETE triggers reject mutation;
- validator identity must differ from active lease owner;
- receipt binds mission_id + payload hash + evidence digest;
- ResultSubmission for Succeeded requires an accepted matching receipt;
- Failed/DeadlineExceeded remain terminalizable without a success receipt.

Observed core proof:

- succeeded_requires_independent_matching_validation_receipt: PASS;
- validation_receipts_are_append_only: PASS;
- current tma-core suite: 21 PASS / 0 FAIL.

Direct Succeeded without receipt returns:

validation_receipt_required

Wrong payload binding returns:

validation_receipt_invalid

This remediation does not reopen F05.

## Canonical failure matrix

F07 reuses exactly the 16 ErrorClass variants from F02:

1. NetworkFailure
2. SessionExpired
3. ElementMoved
4. LayoutChanged
5. RateLimit
6. WorkerCrash
7. ModelTimeout
8. LowConfidence
9. ValidationFailed
10. DuplicateRisk
11. DeadlineRisk
12. ExternalDependency
13. PermissionDenied
14. ContractMismatch
15. Cancelled
16. Unknown

Observed:

F07_FAILURE_MATRIX=PASS classes=16
F07_FAILURE_MATRIX_OK classes=16 recovery=16 injected=16

Examples:

- network failure → bounded retry;
- session expired → refresh session;
- element moved → relocate element;
- layout changed → reobserve/replan;
- rate limit → bounded backoff;
- worker crash → restart worker;
- model timeout → fallback capability;
- low confidence → selective reread;
- validation failed → reexecute then validate;
- duplicate risk → independent review;
- deadline risk → DeadlineExceeded;
- permission denied → review, no blind retry;
- contract mismatch → terminal/replan;
- unknown → conservative manual review.

## Recovery bounds

Implemented hard gates:

- attempt < max_attempts;
- remaining time >= minimum retry window;
- replay_safe required for automatic replay;
- Submit side effects are not auto-replayed;
- fallback chain consumed deterministically;
- backoff is bounded;
- recovery decisions have deterministic SHA-256 fingerprints.

## Durable recovery

Recovery decisions persist through F05 checkpoints.

Observed real process-kill/restart proof:

READY token=1
F07_WORKER_KILLED=PASS
F07_RECOVERY_RESTART=PASS expired_leases=1 token=2

The checkpoint fingerprint remained identical across process death/reopen/reconcile, while fencing advanced from 1 to 2.

## Contracts

Added:

- contracts/executor-result.schema.json
- contracts/validation-report.schema.json
- contracts/validation-receipt.schema.json
- contracts/recovery-decision.schema.json

Observed:

F07_CONTRACTS_OK fixtures=4 validation=independent recovery=bounded

## Provider neutrality

Observed:

F07_PURITY_OK error_classes=16 provider_sdks=0 success_gate=independent_validation_receipt

No OpenAI/Anthropic/Gemini/OpenRouter/Playwright/Tesseract/Selenium/desktop-control/provider SDK is embedded in validation-rust.

## Dedicated gate

Observed:

F07_VERIFY=PASS

Including:

- core validated-success gate;
- fmt;
- clippy -D warnings;
- 19 focused validation/recovery tests;
- self-test;
- 16-class runtime failure matrix;
- recovery probe build;
- real process kill/restart;
- contracts;
- map 55/55;
- purity.

## Full product regression with F07 active

Observed:

CONTINUITY_OK current=F07 next=F08 terminal=F13 root=D:\TIMED-MISSION-AGENT

F07_VERIFY=PASS

SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED

BASELINE_VERIFY=PASS

## Final post-advancement regression

Observed after the canonical pointer advanced to F08:

CONTINUITY_OK current=F08 next=F09 terminal=F13 root=D:\TIMED-MISSION-AGENT

F07_FAILURE_MATRIX=PASS classes=16
F07_WORKER_KILLED=PASS
F07_RECOVERY_RESTART=PASS expired_leases=1 token=2
F07_CONTRACTS_OK fixtures=4 validation=independent recovery=bounded
F07_MAP_OK requirements=55 mapped=55
F07_PURITY_OK error_classes=16 provider_sdks=0 success_gate=independent_validation_receipt
F07_VERIFY=PASS

SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED

BASELINE_VERIFY=PASS

## Closure decision

F07 exit criteria are satisfied. The post-advancement full regression passed.

Canonical advancement:

- F07 → CLOSED
- F08 TRAINING_BENCHMARK_QUALIFICATION_GATES → IN_PROGRESS
- F09 PLATFORM_ADAPTERS → PLANNED

Release tag:

v0.5.2-independent-validation-recovery
