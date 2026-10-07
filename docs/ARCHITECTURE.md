# Architecture v0.1

Pipeline:

MISSION -> TASK -> DEADLINE GATE -> EXTRACT -> VALIDATE -> CONFIDENCE GATE -> RETRY/RECOVER -> FINISH -> EVIDENCE

## Components

- MissionTask: task intent and SLA.
- Extractor: perception adapter contract.
- validate_invoice: CNPJ, date, required fields and arithmetic.
- TimedMissionRunner: SLA, retries, recovery and terminal state.
- EvidenceLedger: SQLite WAL event evidence.
- MissionOrchestrator: bounded parallel execution.
- benchmark: success, SLA, accuracy and latency KPIs.

## Next layers

1. Image ingestion and OCR/vision adapter.
2. Field-level ensemble/conflict resolver.
3. Browser worker isolated from operator desktop.
4. Form mapping and deterministic submission transaction.
5. DOM/screenshot evidence around submission.
6. Session-expiration and layout-drift recovery.
7. Economic scheduler and profitability scoring.
