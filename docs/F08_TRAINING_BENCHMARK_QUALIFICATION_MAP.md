# F08 — Training, Benchmark and Qualification Gates Complete Map

Phase authority: F08
Status: IN_PROGRESS

Goal: prove capabilities under controlled timed training and benchmark runs before promotion, and make production readiness impossible without sequential promotion plus synthetic registration and recovery evidence.

| ID | Requirement | Deliverable | Proof |
|---|---|---|---|
| QUA-001 | Separate qualification layer | qualification-rust crate | build/purity |
| QUA-002 | Training mode | RunMode::Training | tests |
| QUA-003 | Benchmark mode | RunMode::Benchmark | tests |
| QUA-004 | Timed case contract | BenchmarkCase | tests |
| QUA-005 | Timed sample contract | BenchmarkSample | tests |
| QUA-006 | Wall-clock harness | WallClockHarness | timed test |
| QUA-007 | Deterministic synthetic harness | SyntheticHarness | reproducibility test |
| QUA-008 | Minimum run count | QualificationThresholds | rejection test |
| QUA-009 | Success-rate threshold | scorecard gate | rejection test |
| QUA-010 | Accuracy threshold | scorecard gate | rejection test |
| QUA-011 | Validation threshold | F07-proven ratio | rejection test |
| QUA-012 | p95 latency threshold | percentile gate | rejection test |
| QUA-013 | Recovery drill requirement | recovery_passed | rejection test |
| QUA-014 | p50 metric | Scorecard | test |
| QUA-015 | p95 metric | Scorecard | test |
| QUA-016 | p99 metric | Scorecard | test |
| QUA-017 | Success bps | Scorecard | test |
| QUA-018 | Accuracy bps | Scorecard | test |
| QUA-019 | Validation bps | Scorecard | test |
| QUA-020 | Scorecard fingerprint | SHA-256 | stability test |
| QUA-021 | Per-case canonical digest | sample fingerprint | stability test |
| QUA-022 | Reproducible decision | repeated synthetic runs | 5-run proof |
| QUA-023 | Capability scorecard | capability_id bound | tests |
| QUA-024 | PRE_REGISTRATION state | RegistrationState | lifecycle test |
| QUA-025 | REGISTERED state | RegistrationState | lifecycle test |
| QUA-026 | POST_REGISTRATION state | RegistrationState | lifecycle test |
| QUA-027 | PRODUCTION_READY state | RegistrationState | lifecycle test |
| QUA-028 | Illegal lifecycle transition rejected | QualificationSession | tests |
| QUA-029 | Training gate Experimental→Tested | PromotionGate | tests |
| QUA-030 | Benchmark gate Tested→Qualified | PromotionGate | tests |
| QUA-031 | Production gate Qualified→Production | PromotionGate | tests |
| QUA-032 | Sequential promotion enforced in F02 | set_promotion hardening | foundation tests |
| QUA-033 | Direct Experimental→Production rejected | F02 remediation | test |
| QUA-034 | Demotion remains available | safety behavior | test |
| QUA-035 | Training receipt | QualificationReceipt | tests |
| QUA-036 | Benchmark receipt | QualificationReceipt | tests |
| QUA-037 | Production receipt | QualificationReceipt | tests |
| QUA-038 | Receipt binds scorecard fingerprint | gate | tests |
| QUA-039 | Receipt binds registration state | gate | tests |
| QUA-040 | Failed scorecard cannot promote | gate | tests |
| QUA-041 | Missing recovery drill cannot reach production | gate | tests |
| QUA-042 | Missing F07 validation cannot qualify | validation_bps gate | tests |
| QUA-043 | Registration lifecycle is synthetic/offline | no platform dependency | purity |
| QUA-044 | Adapter cannot self-promote | QualificationAuthority owns apply | tests |
| QUA-045 | CapabilityRegistry integration | apply_receipt | tests |
| QUA-046 | Registry current state verified | from-state binding | stale receipt test |
| QUA-047 | Stale receipt rejected | expected current promotion | test |
| QUA-048 | Receipt replay idempotent only after applied state | deterministic behavior | test |
| QUA-049 | Scorecard serialization | canonical text | test |
| QUA-050 | Qualification report serialization | canonical text | test |
| QUA-051 | Benchmark dataset version binding | dataset_id/version | tests |
| QUA-052 | Seed binding | deterministic seed | tests |
| QUA-053 | Deadline/SLA per case | deadline_ms | wall-clock test |
| QUA-054 | Timeout sample counted as failure | harness | test |
| QUA-055 | Recovery sample counted distinctly | scorecard | test |
| QUA-056 | No provider SDK | provider-neutral crate | purity |
| QUA-057 | No platform registration API | synthetic lifecycle only | purity |
| QUA-058 | F07 dependency is validation contract only | no validation bypass | purity |
| QUA-059 | Contract schemas | benchmark/scorecard/receipt | contract gate |
| QUA-060 | One-command phase gate | verify_f08.ps1 | F08_VERIFY=PASS |

## Authority boundaries

- tma-foundation remains capability registry and promotion-state authority.
- tma-qualification evaluates evidence and issues qualification receipts.
- tma-validation remains independent validation/recovery authority.
- adapters cannot directly create a passing scorecard or skip a promotion state through qualification APIs.
- F08 registration lifecycle is synthetic and provider/platform-neutral.
