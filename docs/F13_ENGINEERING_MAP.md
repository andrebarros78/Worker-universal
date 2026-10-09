# F13 Engineering Integration — Candidate v1.0.0-rc.1

**F13 active, not closed. MISSION_PROVEN pending external authorization and acceptance.**

| ID | Requirement | Evidence |
|---|---|---|
| PRD-001 | Single Rust orchestration candidate | production-rust |
| PRD-002 | No new provider dependency | Cargo.toml |
| PRD-003 | F06 deterministic mission plan | DeterministicPlanner |
| PRD-004 | No LLM for deterministic task | MissionRequest allow_llm=false |
| PRD-005 | Plan persisted | persist_plan_checkpoint |
| PRD-006 | Plan rehydrated after reopen | recover_plan_checkpoint |
| PRD-007 | F05 durable mission creation | create_mission |
| PRD-008 | F05 lease acquisition | claim_next |
| PRD-009 | F05 lease expiration and requeue | reconcile |
| PRD-010 | F05 fencing token increment | recovered lease |
| PRD-011 | Old worker result rejected | ResultDisposition |
| PRD-012 | F05 validating transition | transition_with_lease |
| PRD-013 | Local bounded deterministic transform | ASCII uppercase |
| PRD-014 | Persist result file on disk | result.txt |
| PRD-015 | Read back and verify actual evidence bytes | disk_evidence |
| PRD-016 | SHA256 result hash | sha |
| PRD-017 | F07 distinct validator identity | EvidenceValidator |
| PRD-018 | Independent evidence completeness | ValidationReport |
| PRD-019 | F07 authenticated receipt | finalize_validated_success |
| PRD-020 | F05 verified Succeeded terminal hash | terminal_result_hash |
| PRD-021 | F05 ledger hash chain | verify_ledger |
| PRD-022 | SQLite physical integrity | integrity_check |
| PRD-023 | F10 append-only cost | record_cost |
| PRD-024 | No fabricated income or payments | IncomeReport |
| PRD-025 | F02 unqualified production denial | schedule_registered |
| PRD-026 | F09 live adapter denial retained | platform policy |
| PRD-027 | CLI local end-to-end test | tma-production rehearsal |
| PRD-028 | Independent disk/economic evidence check | verify_f13_artifact.py |
| PRD-029 | Dedicated engineering gate and baseline | verify_f13_engineering.ps1 |
| PRD-030 | Production gate separate from simulation | PRODUCTION_ACCEPTANCE=BLOCKED |

## Release criteria not satisfied

A real external platform adapter remains unqualified and unapproved for productive effects. No verified paid task, payer settlement, or real payment evidence is available. Simulation-only success must never confer F08 Production promotion or the F13 MISSION_PROVEN terminal state.
