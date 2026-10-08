# F06 Implementation Matrix

| Requirements | Implementation | Proof |
|---|---|---|
| PLN-001..006 | planner-rust crate, model.rs, planner.rs | unit tests + purity |
| PLN-007..013 | plan.rs validator/topological order/budgets | invalid/cycle/budget tests |
| PLN-014..030 | router.rs + tma-foundation registry/policy | routing/outage/dependency tests |
| PLN-031..034 | capability fixtures in integration tests | API/MCP/browser/vision tests |
| PLN-035 | deterministic local mission fixture | offline test |
| PLN-036..041 | knowledge.rs | provider-neutral tests |
| PLN-042..043 | canonical plan encoding + SHA-256 | deterministic fingerprint test |
| PLN-044..045 | durable_bridge.rs + tma-core DurableStore | restart test |
| PLN-046..047 | routed metadata | tests |
| PLN-048..049 | dependency/source purity scanner | purity gate |
| PLN-050 | scripts/verify_f06.ps1 | dedicated gate |
