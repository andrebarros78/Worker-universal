# F04 — Isolated Browser/Computer Worker Complete Map

Phase authority: F04
Status: IN_PROGRESS

Goal: execute browser/GUI work in isolated worker processes without taking control of the operator desktop, while producing durable evidence and bounded recovery.

| ID | Requirement | F04 deliverable | Proof |
|---|---|---|---|
| BRC-001 | Browser process isolation | Playwright BrowserServer child process | integration test |
| BRC-002 | Headless default | headless=true mandatory default | isolation gate |
| BRC-003 | Browser runtime independence | Playwright-managed Chromium, not operator Edge profile | dependency/isolation gate |
| BRC-004 | Isolated mission context | new browser context per mission | integration test |
| BRC-005 | Isolated profile state | explicit storageState file per mission/profile | session test |
| BRC-006 | Mission deadline | hard mission budget | deadline test |
| BRC-007 | Step timeout | per-step timeout bounded by remaining mission budget | regression |
| BRC-008 | URL navigation | goto action | integration test |
| BRC-009 | DOM observation | content snapshot | evidence test |
| BRC-010 | Screenshot evidence | PNG screenshot | evidence test |
| BRC-011 | Semantic locator — role/name | getByRole | layout drift test |
| BRC-012 | Semantic locator — label | getByLabel | form test |
| BRC-013 | Semantic locator — placeholder | getByPlaceholder | locator integration |
| BRC-014 | Semantic locator — test id | getByTestId | session test |
| BRC-015 | Semantic locator — text | getByText | assertion flow |
| BRC-016 | CSS fallback | locator(css) as final fallback | locator test |
| BRC-017 | Fill/type | fill action | form test |
| BRC-018 | Click | click action | form test |
| BRC-019 | Form submission | browser-driven synthetic submit | form test |
| BRC-020 | File upload | setInputFiles | upload test |
| BRC-021 | Download capture | waitForEvent(download)+saveAs | download test |
| BRC-022 | Download hash/evidence | SHA-256 artifact record | download test |
| BRC-023 | Session save | context.storageState | session test |
| BRC-024 | Session restore | storageState on replacement context | crash/session test |
| BRC-025 | Browser crash detection | disconnected/browser-closed classification | crash test |
| BRC-026 | Bounded crash recovery | configured recovery budget | crash test |
| BRC-027 | Replay-safe retry only | crash retry requires replaySafe=true | crash policy test |
| BRC-028 | Moved-element recovery | semantic locator survives changed DOM ids | layout drift test |
| BRC-029 | Failure evidence | error JSON plus best-effort page evidence | failure test |
| BRC-030 | Evidence manifest | ordered evidence records with hashes | evidence test |
| BRC-031 | No operator mouse/keyboard | no native input-control dependency/API | static isolation gate |
| BRC-032 | No visible browser | headless process contract | runtime/isolation gate |
| BRC-033 | Synthetic local test site | HTTP fixture bound to 127.0.0.1 | integration suite |
| BRC-034 | External-network independence | F04 tests use local fixture/data only | source/test gate |
| BRC-035 | Computer Worker contract | isolated Windows session/VM contract | schema + unit test |
| BRC-036 | Computer Worker desktop boundary | dedicated session required, shared operator desktop rejected | unit test |
| BRC-037 | Capability descriptor | browser.playwright fixture | contract gate |
| BRC-038 | Adapter manifest | browser.playwright adapter fixture | contract gate |
| BRC-039 | Browser telemetry | pid, recovery count, session-restored flag | report test |
| BRC-040 | Reproducible one-command gate | verify_f04.ps1 | F04_VERIFY=PASS |

## Safety/operational boundary

F04 implements standard browser automation and isolated computer-worker contracts only.

It does not implement CAPTCHA bypass, MFA/KYC bypass, bot-detection evasion, stealth/fingerprint spoofing, automation disguising, or human impersonation.

Preferred execution hierarchy remains API/MCP/native integration first, then browser DOM automation, then isolated computer/GUI only when required.
