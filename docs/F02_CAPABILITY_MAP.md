# F02 — Capability Foundation Complete Map

Phase authority: F02
Status: IN_PROGRESS
Goal: create the stable extension plane for every future capability without contaminating the deterministic Rust mission core.

## Capability foundation requirements

| ID | Requirement | F02 deliverable | Proof |
|---|---|---|---|
| CAP-001 | Capability identity | stable id/kind/provider/version model | unit test |
| CAP-002 | Capability descriptor | declarative capability metadata | unit test |
| CAP-003 | Capability registry | register/get/remove/list | unit test |
| CAP-004 | Capability discovery | deterministic snapshot/query | unit test |
| CAP-005 | Contract registry | versioned input/output contracts | schema + unit test |
| CAP-006 | Contract negotiation | compatible-major selection/rejection | unit test |
| CAP-007 | Adapter SDK | provider-neutral adapter lifecycle contract | cross-language SDK surfaces |
| CAP-008 | Adapter manifest | versioned adapter metadata | JSON schema + fixture |
| CAP-009 | Dependency graph | declared capability dependencies | unit test |
| CAP-010 | Dependency resolver | readiness requires dependencies | unit test |
| CAP-011 | Lifecycle model | unavailable/starting/healthy/degraded/failed/disabled | unit test |
| CAP-012 | Health contract | readiness/liveness result model | unit test |
| CAP-013 | Capability readiness | installed/configured/authenticated/operational states | unit test |
| CAP-014 | Permission/scope model | declarative required scopes | unit test |
| CAP-015 | Policy/eligibility interface | allow/deny with stable reason | unit test |
| CAP-016 | Secret provider | opaque secret references only | unit test |
| CAP-017 | Secret redaction | secret material never appears in Debug/evidence envelopes | unit test + contract scan |
| CAP-018 | Artifact store | content-addressed/local artifact interface | unit test |
| CAP-019 | Session manager | opaque session handles and lifecycle | unit test |
| CAP-020 | Tool Bus | provider-neutral invocation surface | unit test |
| CAP-021 | Invocation envelope | universal call contract | JSON schema + fixture |
| CAP-022 | Result envelope | universal success/error/evidence result | JSON schema + fixture |
| CAP-023 | Error taxonomy | stable cross-adapter error classes | unit test |
| CAP-024 | Timeout contract | per-call timeout and remaining budget | unit test |
| CAP-025 | Cancellation contract | cooperative cancellation state | unit test |
| CAP-026 | Side-effect classification | none/read/write/submit/external-effect | unit test |
| CAP-027 | Idempotency declaration | safe/conditional/unsafe replay metadata | unit test |
| CAP-028 | Concurrency declaration | max parallel calls | unit test |
| CAP-029 | Resource lock descriptor | named exclusive/shared resources | unit test |
| CAP-030 | Cost metadata | estimated per-call microunits | resolver fixture |
| CAP-031 | Latency metadata | p95 estimate | resolver fixture |
| CAP-032 | Reliability metadata | observed reliability basis points | resolver fixture |
| CAP-033 | Evidence capability | declared evidence types | resolver fixture |
| CAP-034 | Fallback declaration | ordered alternative capability ids | unit test |
| CAP-035 | Provider priority | deterministic provider-neutral selection | resolver fixture |
| CAP-036 | Local/offline provider | foundation boots and self-tests with no external provider | integration test |
| CAP-037 | External provider interface | native/API/MCP/browser/computer/AI transport descriptors | schema/unit test |
| CAP-038 | Telemetry hooks | invocation start/finish measurement contract | unit test |
| CAP-039 | Replay hooks | replay-safe request metadata | unit test |
| CAP-040 | Simulation hooks | replace live adapter with simulator | unit test |
| CAP-041 | Capability test kit | reusable conformance checks | unit test |
| CAP-042 | Promotion state | experimental/tested/qualified/production | unit test |
| CAP-043 | Capability snapshot | reproducible ordered view for mission evidence | unit test |
| CAP-044 | Configuration contract | validated adapter configuration references | schema/unit test |
| CAP-045 | Providerless boot | no API/MCP/LLM/browser required to initialize foundation | integration test |

## F02 boundary

F02 creates extension contracts and deterministic local/offline implementations only.

It does NOT implement:
- production OCR/vision models (F03);
- Playwright production browser adapter (F04);
- durable mission queue/reboot recovery (F05);
- autonomous planner/research reasoning (F06);
- full validation/recovery agents (F07);
- timed qualification framework (F08);
- HomeOcta/99Freelas production adapters (F09);
- economic scheduling (F10);
- 1→4→8→16 hardening/soak (F11).

## Clean-core invariant

`core-rust/` remains provider/vendor free.

F02 implementation authority lives in `foundation-rust/`, `contracts/`, and `adapter-sdk/`.

## Exit gate

F02 closes only if:
1. CAP-001..CAP-045 are represented by code/contract and mapped proof.
2. foundation crate fmt/clippy/tests/self-test PASS.
3. contract fixtures PASS.
4. cross-language adapter SDK contract surfaces are present.
5. empty/providerless registry boots successfully.
6. incompatible contract versions are rejected.
7. secret material cannot enter result/evidence contracts.
8. deterministic resolver fixtures PASS.
9. full baseline verification remains PASS.
10. proof artifact, clean commit, tag and state advancement to F03 exist.
