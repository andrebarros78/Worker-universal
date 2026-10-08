# F10 Implementation Matrix

| Requirements | Implementation | Verification |
|---|---|---|
| ECO-001..012 | model.rs / scheduler.rs / controller.rs | score and F05 tests |
| ECO-013..030 | deterministic scheduler / F02 registry | eligibility, cost, deadline, margin, concurrency tests |
| ECO-031..045 | ledger.rs / SQLite WAL / hash chain | idempotency, append-only, 16 writer tests |
| ECO-046..050 | report.rs / IncomeReport / CLI | dashboard and reopen tests |
| ECO-051..052 | concurrency and purity gate | economic.rs / verify_f10_purity.py |
| ECO-053..055 | JSON schemas, verifier, baseline | F10_VERIFY=PASS / BASELINE_VERIFY=PASS |
