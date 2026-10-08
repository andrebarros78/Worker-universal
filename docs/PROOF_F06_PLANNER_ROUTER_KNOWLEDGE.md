# Proof — F06 Planner/Router/Knowledge

Date: 2026-10-08
Host: PC Vendas
Canonical root: D:\TIMED-MISSION-AGENT

## Objective

Prove a provider-neutral orchestration layer that converts high-level missions into bounded deterministic execution DAGs, routes every step through the F02 Capability Foundation, exposes Knowledge/Research and Reasoner abstractions, and persists the plan checkpoint through the F05 Durable Mission Runtime.

## Architecture

F06 is implemented as a separate Rust crate:

planner-rust

Authority boundaries:

- tma-core remains mission-state and durable-truth authority;
- tma-foundation remains capability/contract/health/policy authority;
- tma-planner owns classification, planning, routing and Knowledge/Reasoner abstractions;
- no LLM, browser, OCR, API or MCP provider SDK is embedded in planner-rust.

## Requirement map

- docs/F06_PLANNER_ROUTER_KNOWLEDGE_MAP.md
- PLN-001..PLN-050
- docs/F06_IMPLEMENTATION_MATRIX.md

Observed:

F06_MAP_OK requirements=50 mapped=50

## Delivered

- MissionRequest contract;
- deterministic MissionClassifier;
- mission classes: deterministic/document/browser/research/composite;
- Planner trait;
- DeterministicPlanner local fallback;
- ExecutionPlan DAG;
- step dependency validation;
- cycle detection;
- deadline and cost budget validation;
- scope/policy validation;
- CapabilityRequirement;
- Router integrated with tma-foundation CapabilityRegistry;
- health/readiness/dependency/promotion/reliability/cost/latency filters;
- provider preference;
- deterministic reliability → cost → latency → id tie-break;
- deterministic fallback chain;
- outage fallback;
- missing-dependency rejection;
- RoutedPlan metadata;
- API/MCP/browser selection fixtures;
- F03 vision/OCR routing fixture;
- replay-safe/idempotency propagation;
- resource-lock propagation;
- KnowledgeQuery/KnowledgeResult;
- ResearchProvider trait;
- ReasonerProvider trait;
- LocalResearchProvider;
- LocalRuleReasoner;
- reasoner-output evidence validation;
- stable canonical plan encoding;
- SHA-256 plan fingerprint;
- F05 durable plan checkpoint;
- reopen/reconcile/reclaim recovery of plan fingerprint.

## Focused tests

Observed:

16 tests PASS / 0 FAIL

Covered scenarios include:

- deterministic mission with no LLM/API;
- research mission with local/no-LLM reasoner;
- invalid dependency;
- dependency cycle;
- deadline/cost bound;
- API selection;
- MCP selection;
- browser selection;
- provider preference;
- deterministic candidate ordering;
- provider outage fallback;
- missing capability dependency;
- policy-scope denial;
- F03 Tesseract capability routing;
- unsafe capability disables replay;
- bounded local research;
- invalid reasoner evidence rejection;
- stable plan fingerprint;
- durable checkpoint restart/reclaim.

## Self-test

Observed:

tma-planner status=ok class=Deterministic steps=2 routed=2 fingerprint=d2645d04f962812de5367f9d2e7eccf6c57f4874a402fbafe81a0c508102c268

## Contracts

Added:

- contracts/planner-mission.schema.json
- contracts/execution-plan.schema.json
- contracts/knowledge-query.schema.json
- contracts/fixtures/planner.mission.deterministic.json
- contracts/fixtures/execution.plan.deterministic.json
- contracts/fixtures/knowledge.query.local.json
- contracts/fixtures/capability.research.local.json

Observed:

F06_CONTRACTS_OK fixtures=4 planner=1 knowledge=1 research_capability=1

## Provider neutrality

Observed:

F06_PURITY_OK provider_sdks=0 browser_ocr_duplication=0 capability_authority=tma-foundation durable_authority=tma-core

Planner dependencies are limited to:

- sha2;
- tma-foundation;
- tma-core.

No OpenAI/Anthropic/Gemini/OpenRouter/Playwright/Tesseract/Selenium/provider SDK dependency is present in planner-rust.

## Full product regression with F06 active

Observed:

CONTINUITY_OK current=F06 next=F07 terminal=F13 root=D:\TIMED-MISSION-AGENT

F06_VERIFY=PASS

SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED

BASELINE_VERIFY=PASS

## Final post-advancement regression

Observed after the canonical pointer advanced to F07:

CONTINUITY_OK current=F07 next=F08 terminal=F13 root=D:\TIMED-MISSION-AGENT

F06_CONTRACTS_OK fixtures=4 planner=1 knowledge=1 research_capability=1
F06_MAP_OK requirements=50 mapped=50
F06_PURITY_OK provider_sdks=0 browser_ocr_duplication=0 capability_authority=tma-foundation durable_authority=tma-core
F06_VERIFY=PASS

SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED

BASELINE_VERIFY=PASS

## Closure decision

F06 exit criteria are satisfied. The post-advancement full regression passed.

Canonical advancement:
- F06 → CLOSED
- F07 INDEPENDENT_VALIDATION_RECOVERY → IN_PROGRESS
- F08 TRAINING_BENCHMARK_QUALIFICATION_GATES → PLANNED

Release tag:

v0.5.1-planner-router-knowledge
