# F09 Implementation Matrix

| Requirement ranges | Implementation | Verification |
|---|---|---|
| PAD-001..008 | src/models.ts, adapters.ts, registry | manifests and 5-adapter tests |
| PAD-009..015 | src/runner.ts reusing F04 | real simulated browser runs + evidence |
| PAD-016..019 | delegatedCapability/health metadata | plan/manifest assertions |
| PAD-020..030 | src/policy.ts | fail-closed tests |
| PAD-031..035 | BrowserRuntime bounded restart + request guard | real crash recovery & timeout tests |
| PAD-036..040 | status envelopes, schemas, fixtures | contract verification |
| PAD-041..047 | tests and purity scanner | F09_VERIFY |
| PAD-048..050 | proof, continuity, Git release | final gate and remote proof |
