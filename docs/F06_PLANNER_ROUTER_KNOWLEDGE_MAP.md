# F06 — Planner/Router/Knowledge Complete Map

Phase authority: F06
Status: IN_PROGRESS

Goal: convert a high-level mission into a bounded deterministic execution DAG, route each step to an eligible capability, expose provider-neutral Knowledge/Research and Reasoner interfaces, and persist the plan through the F05 durable runtime.

| ID | Requirement | F06 deliverable | Proof |
|---|---|---|---|
| PLN-001 | Separate orchestration layer | planner-rust crate | build/purity gate |
| PLN-002 | Mission request contract | MissionRequest | unit tests |
| PLN-003 | Deterministic mission classifier | MissionClassifier | fixture tests |
| PLN-004 | Mission classes | deterministic/document/browser/research/composite | tests |
| PLN-005 | Planner interface | Planner trait | compile/test |
| PLN-006 | Deterministic local planner | DeterministicPlanner | no-provider test |
| PLN-007 | Execution DAG | ExecutionPlan + PlanStep dependencies | DAG tests |
| PLN-008 | Unique step IDs | validation | invalid-plan test |
| PLN-009 | Dependency existence | validation | invalid-plan test |
| PLN-010 | Cycle rejection | topological validation | cycle test |
| PLN-011 | Deadline bound | estimated latency <= mission budget | budget test |
| PLN-012 | Cost bound | estimated cost <= mission budget | budget test |
| PLN-013 | Policy scope bound | required scopes enforced | policy test |
| PLN-014 | Capability requirement contract | CapabilityRequirement | tests |
| PLN-015 | Router interface | Router | compile/test |
| PLN-016 | F02 registry integration | CapabilityRegistry.resolve | routing tests |
| PLN-017 | Health/readiness filtering | registry operational rules | outage test |
| PLN-018 | Dependency readiness filtering | F02 dependency resolver | dependency test |
| PLN-019 | Promotion filtering | minimum promotion | test |
| PLN-020 | Reliability filtering | minimum reliability | test |
| PLN-021 | Cost filtering | maximum cost | test |
| PLN-022 | Latency filtering | maximum p95 | test |
| PLN-023 | Permission filtering | policy + registry | test |
| PLN-024 | Evidence filtering | required evidence | test |
| PLN-025 | Provider preference | provider preference ordering | API/MCP/browser fixture |
| PLN-026 | Deterministic candidate tie-break | reliability→cost→latency→id | routing test |
| PLN-027 | Deterministic fallback chain | primary + ready fallbacks | outage test |
| PLN-028 | Provider outage recovery | failed primary selects fallback | outage test |
| PLN-029 | Missing dependency rejection | no route returned | dependency test |
| PLN-030 | Route plan | RoutedPlan/RoutedStep | tests |
| PLN-031 | API selection fixture | API capability wins appropriate route | fixture test |
| PLN-032 | MCP selection fixture | MCP capability wins provider preference | fixture test |
| PLN-033 | Browser selection fixture | browser route selected when required | fixture test |
| PLN-034 | Vision/OCR integration fixture | F03 capability routable | fixture test |
| PLN-035 | Local deterministic fallback | mission class works without LLM/API | offline test |
| PLN-036 | Knowledge query contract | KnowledgeQuery/KnowledgeResult | tests |
| PLN-037 | Research provider abstraction | ResearchProvider trait | mock/local test |
| PLN-038 | Reasoner provider abstraction | ReasonerProvider trait | mock/local test |
| PLN-039 | Local/no-LLM reasoner | LocalRuleReasoner | offline test |
| PLN-040 | No LLM authority | planner/router validates all reasoner output | invalid reasoner test |
| PLN-041 | Bounded research | max sources + deadline/cost constraints | test |
| PLN-042 | Canonical plan serialization | stable text encoding | roundtrip/hash test |
| PLN-043 | Plan fingerprint | SHA-256 stable fingerprint | deterministic test |
| PLN-044 | F05 checkpoint integration | persist plan fingerprint/checkpoint | restart test |
| PLN-045 | Restart plan recovery | reopen durable DB and recover plan checkpoint | restart test |
| PLN-046 | Replay-safe route metadata | idempotency/side-effect propagated | test |
| PLN-047 | Resource-lock metadata | capability locks propagated | test |
| PLN-048 | No provider lock-in | no concrete API/LLM SDK dependency | purity gate |
| PLN-049 | No browser/OCR code duplication | F06 uses capability IDs/contracts only | purity gate |
| PLN-050 | One-command phase gate | verify_f06.ps1 | F06_VERIFY=PASS |

## Authority boundaries

- tma-core remains mission-state and durable-truth authority.
- tma-foundation remains capability/contract/health/policy authority.
- planner-rust owns classification, planning and routing only.
- LLM/research providers may propose or enrich; they never bypass plan validation, policy, budgets, capability readiness or durable mission state.
