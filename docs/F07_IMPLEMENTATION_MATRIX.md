# F07 Implementation Matrix

| Requirements | Implementation | Proof |
|---|---|---|
| VAL-001..004 | validation-rust crate + R05-01 core gate | integration tests |
| VAL-005..007 | core-rust durable validation_receipts | core append-only/binding tests |
| VAL-008..020 | validation-rust validator.rs/model.rs | evidence and false-success tests |
| VAL-021..038 | recovery.rs over F02 ErrorClass | exhaustive 16-class matrix |
| VAL-039..044 | RecoveryContext/RecoveryAgent | budget/replay/fallback tests |
| VAL-045..046 | durable_bridge.rs recovery checkpoint | restart test |
| VAL-047..049 | durable finalization bridge | success/failure integration tests |
| VAL-050..052 | deterministic reports/fingerprints | stability tests |
| VAL-053..054 | purity scanner | F07_PURITY_OK |
| VAL-055 | scripts/verify_f07.ps1 | dedicated gate |
