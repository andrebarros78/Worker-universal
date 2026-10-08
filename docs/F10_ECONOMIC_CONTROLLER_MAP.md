# F10 — Economic Controller and Scheduler: Contract Map

Phase: F10 IN_PROGRESS
Scope: deterministic evaluation + durable append-only accounting; no third-party payment or earnings claim.

| ID | Requirement | Implementation |
|---|---|---|
| ECO-001 | Independent economic crate | economic-rust |
| ECO-002 | BRL cents fixed-point | model.rs |
| ECO-003 | Opportunity record | Opportunity |
| ECO-004 | Expected reward bps | score |
| ECO-005 | Estimated cost | score |
| ECO-006 | Expected net profit | score |
| ECO-007 | Margin bps | score |
| ECO-008 | Profit per hour | score |
| ECO-009 | Deterministic tie-break | schedule |
| ECO-010 | Fingerprinted plan | SHA-256 |
| ECO-011 | Observed F05 success rates | observed_success_from_missions |
| ECO-012 | No fake success with zero observations | observed_success |
| ECO-013 | Input deduplication | schedule |
| ECO-014 | Eligibility gate | schedule |
| ECO-015 | F02 Production gate | schedule_registered |
| ECO-016 | F02 Healthy/Operational gate | schedule_registered |
| ECO-017 | Missing registry mapping blocks | schedule_registered |
| ECO-018 | Deadline risk gate | schedule |
| ECO-019 | Negative EV rejection | schedule |
| ECO-020 | Minimum profit gate | schedule |
| ECO-021 | Minimum margin gate | schedule |
| ECO-022 | Per-mission cost cap | schedule |
| ECO-023 | Daily cost cap | costs_since + schedule |
| ECO-024 | Reserved budget accounted | schedule |
| ECO-025 | Stop-loss | schedule |
| ECO-026 | Concurrency slots | schedule |
| ECO-027 | External busy slots | schedule |
| ECO-028 | Resource locks | schedule |
| ECO-029 | Cost spike changes ranking | tests |
| ECO-030 | Fixed-point overflow checks | score |
| ECO-031 | Append-only SQLite events | ledger |
| ECO-032 | SQLite WAL/FULL | ledger |
| ECO-033 | Idempotent event keys | ledger |
| ECO-034 | Conflicting key rejected | ledger |
| ECO-035 | Hash-chain evidence | verify_chain |
| ECO-036 | Mutation trigger | ledger |
| ECO-037 | Deletion trigger | ledger |
| ECO-038 | Verified earning bound to F05 Succeeded | record_verified_earning |
| ECO-039 | Earning bound to F05 result hash | record_verified_earning |
| ECO-040 | One verified earning per mission | partial unique index |
| ECO-041 | Payment kept separate from accrual | record_payment |
| ECO-042 | Settlement reference deduplication | partial unique index |
| ECO-043 | Payment cannot exceed earned | ledger |
| ECO-044 | Cost accounting | record_cost |
| ECO-045 | Daily cost projection | costs_since |
| ECO-046 | Accrued profit report | IncomeReport |
| ECO-047 | Recorded cash profit report | IncomeReport |
| ECO-048 | Accounts receivable | IncomeReport |
| ECO-049 | JSON financial dashboard | report.rs + CLI |
| ECO-050 | Restart/reopen consistency | ledger tests |
| ECO-051 | 16 parallel writer proof | concurrency test |
| ECO-052 | No vendor SDK in economic core | purity |
| ECO-053 | Contract schemas and fixtures | contracts |
| ECO-054 | One-command F10 gate | verify_f10.ps1 |
| ECO-055 | Full baseline and Git seal | proof + release |

## Authority

- F02: capability health, readiness, promotion. The production scheduler reads F02, it does not set promotion.
- F05: durable mission success and validated terminal hash.
- F07: independent validation receipt required by F05 for Succeeded.
- F08: qualification and promotion lifecycle.
- F09: platform adapters remain simulation-only pending authorized productive integration.
- F10: accounting and economic ranking, never automatic external payment.

## Accounting boundaries

A **verified earning** is an accrued claim from a mission with F05 Succeeded and a matching result hash. It is NOT confirmation a platform has paid. A **recorded payment** is a separately entered settlement amount/reference; external bank/platform reconciliation is not implemented in F10. Never equate either with actual money earned unless independently verified.
