# F04 Implementation Matrix

| Requirements | Implementation | Proof |
|---|---|---|
| BRC-001..007 | web-worker/src/browser_runtime.ts | runtime/deadline/crash tests |
| BRC-008..022 | browser_runtime.ts, semantic_locator.ts, evidence.ts | synthetic form/upload/download tests |
| BRC-023..028 | session state + recovery loop | crash/session/layout tests |
| BRC-029..030 | EvidenceStore | failure/evidence tests |
| BRC-031..034 | headless Chromium + isolation/static gate + local fixture | isolation gate |
| BRC-035..036 | computer_contract.ts + JSON schema | unit/contract gate |
| BRC-037..038 | F02 capability/adapter fixtures | contract gate |
| BRC-039 | BrowserWorkerReport telemetry | integration tests |
| BRC-040 | scripts/verify_f04.ps1 | dedicated gate |
