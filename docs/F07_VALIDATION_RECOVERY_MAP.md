# F07 — Independent Validation and Recovery Complete Map

Phase authority: F07
Status: IN_PROGRESS

Goal: never treat executor-declared success as mission success until independent evidence validation proves it, and classify/recover failures under bounded, deterministic policy.

| ID | Requirement | F07 deliverable | Proof |
|---|---|---|---|
| VAL-001 | Separate validation layer | validation-rust crate | build/purity gate |
| VAL-002 | Executor result contract | ExecutorResult | unit tests |
| VAL-003 | Independent validator identity | validator id distinct from lease owner | durable receipt test |
| VAL-004 | Success receipt requirement | F05 R05-01 gate | bypass test |
| VAL-005 | Append-only validation receipt | validation_receipts table + triggers | core tests |
| VAL-006 | Receipt payload binding | subject result hash must match | mismatch test |
| VAL-007 | Receipt evidence binding | evidence digest | receipt test |
| VAL-008 | Evidence item contract | EvidenceItem | tests |
| VAL-009 | Evidence SHA-256 validation | 64-hex digest | invalid evidence test |
| VAL-010 | Evidence non-empty validation | bytes > 0 | test |
| VAL-011 | Evidence completeness gate | required evidence kinds | false-success test |
| VAL-012 | Capability-specific proof strategy | ProofStrategy | tests |
| VAL-013 | Document proof strategy | OCR/field confidence requirements | F03 fixture test |
| VAL-014 | Browser proof strategy | DOM+screenshot requirements | F04 fixture test |
| VAL-015 | Research proof strategy | sources requirement | F06 fixture test |
| VAL-016 | Deterministic proof strategy | artifact/value evidence | fixture test |
| VAL-017 | Confidence gate | minimum confidence | low-confidence test |
| VAL-018 | Deadline gate | deadline risk preempts success | deadline test |
| VAL-019 | Declared success not trusted | success + missing evidence rejected | bypass test |
| VAL-020 | Declared failure respected as evidence input | classifier path | test |
| VAL-021 | Stable failure taxonomy | reuse F02 ErrorClass | exhaustive test |
| VAL-022 | All ErrorClass variants covered | 16/16 injection matrix | matrix test |
| VAL-023 | NetworkFailure recovery | retry/fallback | matrix test |
| VAL-024 | SessionExpired recovery | refresh session | matrix test |
| VAL-025 | ElementMoved recovery | relocate element | matrix test |
| VAL-026 | LayoutChanged recovery | reobserve/replan | matrix test |
| VAL-027 | RateLimit recovery | bounded backoff | matrix test |
| VAL-028 | WorkerCrash recovery | restart worker | matrix test |
| VAL-029 | ModelTimeout recovery | provider fallback | matrix test |
| VAL-030 | LowConfidence recovery | selective reread/retry | matrix test |
| VAL-031 | ValidationFailed recovery | revalidate/reexecute | matrix test |
| VAL-032 | DuplicateRisk recovery | manual/independent review | matrix test |
| VAL-033 | DeadlineRisk terminalization | DeadlineExceeded | matrix test |
| VAL-034 | ExternalDependency recovery | wait/backoff | matrix test |
| VAL-035 | PermissionDenied terminal/review | no blind retry | matrix test |
| VAL-036 | ContractMismatch terminal/replan | no blind retry | matrix test |
| VAL-037 | Cancelled terminalization | cancelled/failed | matrix test |
| VAL-038 | Unknown conservative handling | needs review | matrix test |
| VAL-039 | Recovery context | attempts/deadline/replay-safe | tests |
| VAL-040 | Retry budget | max attempts hard bound | test |
| VAL-041 | Deadline budget | minimum remaining time | test |
| VAL-042 | Replay safety gate | unsafe step never auto-replayed | test |
| VAL-043 | Side-effect gate | external-effect retry conservative | test |
| VAL-044 | Fallback chain consumption | next capability selection | test |
| VAL-045 | Recovery checkpoint | F05 save_checkpoint integration | restart test |
| VAL-046 | Recovery checkpoint restart | reopen/reclaim preserves decision | restart test |
| VAL-047 | Validation receipt finalization bridge | validate→receipt→Succeeded | durable integration test |
| VAL-048 | False receipt rejection | wrong hash/validator identity | integration test |
| VAL-049 | Failure terminalization bridge | validated terminal failure | integration test |
| VAL-050 | Evidence completeness report | missing/invalid evidence list | tests |
| VAL-051 | Validation report fingerprint | deterministic SHA-256 | stability test |
| VAL-052 | Recovery decision fingerprint | deterministic encoding | stability test |
| VAL-053 | No provider SDK authority | provider-neutral validation crate | purity gate |
| VAL-054 | No browser/OCR implementation duplication | evidence-only validation | purity gate |
| VAL-055 | One-command phase gate | verify_f07.ps1 | F07_VERIFY=PASS |

## Authority boundaries

- Executor output is untrusted input.
- tma-validation owns independent proof evaluation and recovery decisions.
- tma-core remains mission-state/durable truth authority.
- tma-foundation remains the canonical failure taxonomy.
- tma-planner remains plan/route authority.
- Success is impossible through the durable result API without an accepted independent validation receipt.
