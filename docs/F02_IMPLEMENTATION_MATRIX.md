# F02 Implementation Matrix

This matrix maps the complete conceptual capability map to implementation and proof.

| IDs | Implementation | Proof |
|---|---|---|
| CAP-001..004 | `foundation-rust/src/model.rs`, `registry.rs` | registry/snapshot tests |
| CAP-005..006 | `foundation-rust/src/contracts.rs` | contract negotiation tests |
| CAP-007..008 | `adapter-sdk/{python,typescript,go}`, adapter manifest schema | SDK tests + fixture gate |
| CAP-009..010 | registry dependency graph/resolver | dependency/cycle tests |
| CAP-011..013 | lifecycle/readiness in descriptor + health mutation | health gate tests |
| CAP-014..015 | `services.rs` ScopePolicy | permission/deny tests |
| CAP-016..017 | SecretRef/SecretProvider/SecretValue redaction | secret redaction + result-contract scan |
| CAP-018 | LocalArtifactStore | artifact round-trip |
| CAP-019 | InMemorySessionManager | opaque session lifecycle |
| CAP-020..022 | `tool_bus.rs` + invocation/result schemas | offline echo integration |
| CAP-023 | StableError/ErrorClass + error schema | error assertions |
| CAP-024..025 | invocation timeout + CancellationToken | timeout/cancellation tests |
| CAP-026..029 | descriptor side-effect/idempotency/concurrency/resource locks | descriptor validation/model tests |
| CAP-030..035 | cost/p95/reliability/evidence/fallback/provider preference | deterministic resolver tests |
| CAP-036 | empty registry + LocalEchoAdapter | providerless boot/self-test |
| CAP-037 | TransportKind + capability schema | schema/descriptor tests |
| CAP-038 | TelemetrySink/InMemoryTelemetry | start/finish telemetry test |
| CAP-039 | Invocation replayable/idempotency metadata | invocation model/test fixture |
| CAP-040 | simulation flag + adapter supports_simulation | conformance test kit |
| CAP-041 | `testkit.rs` adapter_conformance | conformance test |
| CAP-042 | PromotionState | promotion gate test |
| CAP-043 | CapabilitySnapshot | ordered snapshot test |
| CAP-044 | config_ref + adapter-config schema | contract gate |
| CAP-045 | providerless registry + foundation self-test | self-test |

## Runtime dependency rule

F02 has no mandatory external API, MCP server, LLM provider or browser provider.

The local/offline echo adapter exists solely to prove the foundation can boot, discover, invoke, measure and validate a capability without external infrastructure.
