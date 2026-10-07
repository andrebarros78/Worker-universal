# CANONICAL STATE

Project: TIMED-MISSION-AGENT
Canonical root: `D:\TIMED-MISSION-AGENT`

## Sovereign objective

Generate dependable personal income by turning eligible automatable work into completed, validated missions with minimal operator burden. Reliability and economic usefulness are first-class product requirements.

## Canonical entry

Every new AI/chat/operator starts at:

`00_START_HERE.md`

Machine-readable execution pointer:

`state\EXECUTION_STATE.json`

## Closed baselines

### F00 — POLYGLOT_V0_2 — CLOSED
- commit: `b2e0d814acefcc8dbc31323d77e5e9671aa7b15c`
- tag: `v0.2.0-polyglot-baseline`
- proof: `docs\PROOF_POLYGLOT_V0_2.md`

### F01 — CONCEPT_EXECUTIVE_CONTINUITY_BASELINE — CLOSED
- conceptual project persisted;
- executive project persisted;
- F00→F13 phases persisted;
- machine-readable state persisted;
- continuation gate integrated into baseline verification;
- proof: `docs\PROOF_F01_EXECUTIVE_CONTINUITY.md`
- release tag: `v0.2.1-executive-baseline`

## Current phase

`F02 — CAPABILITY_FOUNDATION — IN_PROGRESS`

Objective:
Build the clean extension plane without adding provider/platform dependencies to Rust core.

Required deliverables:
- Capability Registry;
- Adapter SDK/contracts;
- capability health model;
- Policy/Eligibility interface;
- SecretProvider/Vault interface;
- Artifact Store interface;
- Session Manager interface;
- Tool Bus descriptors for native/API/MCP/browser/AI capabilities;
- local/offline implementations sufficient for deterministic tests.

Exit gate:
- optional provider absence does not prevent boot;
- registry health/fallback tests pass;
- incompatible contracts are rejected;
- secrets never enter mission/evidence serialization;
- capability selection fixtures are deterministic;
- full baseline gate remains PASS;
- F02 proof + commit + state advancement exist.

## Next phase

`F03 — VISION_OCR_DOCUMENT_INTELLIGENCE — PLANNED`

Do not implement F03 as the main lane before F02 closes.

## Terminal phase

`F13 — PERSONAL_PRODUCTION_V1_0`

Terminal label: `MISSION_PROVEN`.

The complete conceptual and executive definition is in:
- `docs\PROJECT_CONCEPT.md`
- `docs\EXECUTIVE_PROJECT.md`
- `docs\EXECUTION_PHASES.md`
- `docs\PHASE_LEDGER.md`
