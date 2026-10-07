# Project Concept — TIMED-MISSION-AGENT

## 1. Product definition

TIMED-MISSION-AGENT is a personal Universal Mission Worker designed to transform eligible automatable work into reliable completed outcomes with minimal operator attention.

It is not a bot for one website. It is a reusable mission platform with platform-specific adapters.

## 2. Sovereign purpose

Primary outcome: dependable personal income.

Engineering exists to support that outcome through:

- correctness;
- speed under deadline;
- low operating cost;
- low maintenance burden;
- recovery from technical failure;
- evidence of completion;
- reuse across different work sources.

## 3. Universal mission model

Every mission is represented by:

- objective;
- mission type;
- hard/soft deadline;
- expected economic value;
- constraints;
- required capabilities;
- completion criteria;
- validation criteria;
- evidence requirements;
- retry/recovery budget;
- external dependencies.

The system must not encode HomeOcta, 99Freelas or any provider into the domain core.

## 4. Primary use families

### 4.1 Timed microtasks

Examples: document transcription, structured data entry, classification, extraction, verification and similar bounded work.

Required qualities:

- hard SLA;
- field-level confidence;
- deterministic validation where possible;
- fast retry/recovery;
- benchmark before release.

### 4.2 Training and benchmark

The same execution engine must run synthetic/training workloads before a capability is promoted.

Metrics include:

- success rate;
- field accuracy;
- p50/p95/p99 latency;
- deadline pass rate;
- retries;
- recovery rate;
- cost per successful task;
- failure-class distribution.

### 4.3 Operational execution

Once an adapter/capability satisfies its release gate, it may be used for eligible operational work according to that adapter's configured rules.

## 5. Product operating flow

MISSION
→ CLASSIFY
→ CHECK ELIGIBILITY/RULES
→ PLAN
→ SELECT CAPABILITIES
→ PRE-REGISTRATION GATE WHEN APPLICABLE
→ REGISTRATION/SETUP
→ POST-REGISTRATION GATE
→ EXECUTE
→ OBSERVE
→ VALIDATE
→ RECOVER IF NEEDED
→ CALCULATE ECONOMIC RESULT
→ PERSIST EVIDENCE
→ CLOSE

## 6. Logical agents

The product uses a small number of logical roles, not uncontrolled agent proliferation:

1. Mission Controller — objective, decomposition, state and closing gate.
2. Planner/Router — plan and capability selection.
3. Browser/Computer Worker — browser or isolated desktop actions.
4. API/Automation Worker — direct integrations when preferable.
5. Knowledge/Research Worker — rules, documentation and interpretation.
6. Document/Vision Worker — OCR, document extraction and visual reasoning.
7. Validation Agent — independent proof of result.
8. Recovery Agent — diagnosis and bounded recovery.
9. Economic Controller — value, cost, time and margin.

Scheduler/Queue, ledger, secrets, sessions, telemetry and state are infrastructure, not autonomous decision agents.

## 7. Capability model

The core boots with zero external service dependencies.

Capabilities are registered dynamically:

- browser.playwright
- computer.windows
- vision.ocr.local
- vision.model.remote
- document.invoice
- api.http
- mcp.*
- research.web
- llm.local
- llm.remote
- storage.*
- notification.*

API, MCP and LLM are optional capability transports/providers. They are never the product itself.

## 8. Clean-core principle

Rust core contains only:

- mission identity;
- state machine;
- time/deadline rules;
- idempotency;
- leases/fencing;
- result acceptance;
- invariant checks;
- evidence requirements.

It must not import provider SDKs.

## 9. Execution isolation

The operator desktop is not the agent desktop.

Preferred order:

1. API/MCP/native integration;
2. isolated headless browser;
3. isolated browser session;
4. isolated Windows/VM desktop worker when a real GUI is required.

Workers are replaceable. Mission truth survives them.

## 10. Platform adapters

Adapters live outside core and provide:

- discovery;
- capability mapping;
- field/form mapping;
- session behavior;
- validation strategy;
- evidence strategy;
- error classification;
- economic parameters.

Initial adapter families:

- generic_web
- generic_form
- generic_invoice
- homeocta
- 99freelas
- future adapters

## 11. Reliability model

Failure classes are explicit:

- NETWORK_FAILURE
- SESSION_EXPIRED
- ELEMENT_MOVED
- LAYOUT_CHANGED
- RATE_LIMIT
- WORKER_CRASH
- MODEL_TIMEOUT
- LOW_CONFIDENCE
- VALIDATION_FAILED
- DUPLICATE_RISK
- DEADLINE_RISK
- EXTERNAL_DEPENDENCY

Each failure class maps to a bounded recovery policy.

## 12. Economic model

The system eventually ranks opportunities by expected net value:

`P(success) × expected_revenue - compute - API - LLM - expected_rework - risk_reserve`

The economic controller is advisory until enough observed data exists; deterministic mission correctness always has priority.

## 13. Final product definition

V1.0 is not "all features implemented". It is a proven personal production system that can execute supported mission classes end-to-end, recover from technical failures, reject stale/duplicate results, preserve evidence and report economic outcomes without depending on the originating chat.
