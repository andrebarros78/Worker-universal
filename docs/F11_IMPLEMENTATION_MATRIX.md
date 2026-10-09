# F11 Implementation Matrix

| Requirement ranges | Implementation | Evidence |
|---|---|---|
| AVC-001..020 | availability-rust/src, tests | Rust matrix/soak |
| AVC-021..042 | supervisor-go/supervisor/pool.go and tests | Go matrix/race/HTTP |
| AVC-043..048 | F05 real restart/reconcile and regression | kill/restart gate |
| AVC-049..052 | JSON schemas, fixtures, map and purity | contract/map/purity gates |
| AVC-053..056 | verify_f11.ps1, global baseline, Git release | F11_VERIFY/BASELINE_VERIFY |
