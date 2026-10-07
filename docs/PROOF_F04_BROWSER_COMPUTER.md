# Proof — F04 Isolated Browser/Computer Worker

Date: 2026-10-07
Host: PC Vendas
Canonical root: D:\TIMED-MISSION-AGENT

## Objective

Prove isolated browser execution with real Playwright/Chromium, semantic DOM control, form/upload/download operations, evidence capture, session persistence, bounded crash recovery, deadline containment and a strict Computer Worker isolation contract.

## Requirement map

- docs/F04_BROWSER_COMPUTER_MAP.md
- BRC-001..BRC-040
- docs/F04_IMPLEMENTATION_MATRIX.md

Observed:
F04_MAP_OK requirements=40 mapped=40

## Browser runtime

Implemented under web-worker:

- Playwright BrowserServer child process;
- enforced headless mode;
- one BrowserContext per mission;
- explicit storageState session files;
- mission deadline and step timeout containment;
- goto/fill/click/upload/download/assert/observe/session actions;
- semantic locators by role/name, label, placeholder, test id, text and CSS fallback;
- DOM and screenshot evidence;
- download evidence with SHA-256;
- sanitized download filename containment;
- error evidence;
- evidence manifest;
- browser process PID telemetry;
- bounded browser crash recovery;
- retry only when replaySafe is true;
- session restore after replacement browser process;
- mission telemetry reset when the runtime object is reused.

## Computer Worker boundary

The Computer Worker contract requires:

- isolated Windows session or VM;
- dedicated session;
- no operator mouse/keyboard control;
- bounded runtime;
- evidence requirement.

A shared operator desktop is rejected by contract.

## Local runtime dependencies

- Node v24.18.0
- npm 11.16.0
- Playwright 1.64.0
- Playwright-managed Chromium
- dependency store: C:\ProgramData\SentinelX\workspace\tma-web-worker-deps
- browser store: C:\ProgramData\SentinelX\workspace\tma-playwright-browsers
- web-worker\node_modules is a gitignored junction to the external dependency store

Reproducible bootstrap:
scripts/bootstrap_f04_playwright.ps1

Observed bootstrap:
F04_BOOTSTRAP=PASS
PLAYWRIGHT_READY=true

## Focused verification

scripts/verify_f04.ps1

Observed after final hardening:

- 14 Node tests PASS / 0 FAIL;
- web-worker self-test PASS;
- F04_CONTRACTS_OK fixtures=5;
- F04_MAP_OK requirements=40 mapped=40;
- F04_PURITY_OK;
- F04_VERIFY=PASS.

Covered scenarios include:

- real headless browser form execution;
- file upload;
- file download;
- evidence capture;
- download path containment;
- runtime reuse;
- moved-element/layout drift;
- real Chromium process crash;
- session restore after crash;
- unsafe-step no-replay policy;
- mission deadline;
- assertion failure evidence;
- Computer Worker dedicated-session acceptance;
- shared desktop rejection.

## Operator desktop isolation proof

Observed isolation probe:

browserSession=0
explorerSession=1
headless=true
sharedOperatorSession=false

The browser process did not run in the operator Explorer session and headless mode is mandatory.

## Safety boundary

Static source verification proves no native mouse/keyboard automation dependency and no stealth/evasion implementation.

Explicitly not implemented:

- CAPTCHA bypass;
- MFA/KYC bypass;
- bot-detection evasion;
- stealth/fingerprint spoofing;
- automation disguise;
- human impersonation.

## F03 regression-gate remediation discovered during F04

Record:
docs/REMEDIATION_F03R1_REGRESSION_GATE_HYGIENE.md

The F03 runtime benchmark previously rewrote a tracked release report on every regression run because latency measurements vary. The regression gate now writes its live report outside the repository while preserving the canonical v0.3.0 release report.

Observed proof:

F03_REPORT_BEFORE=cb8b215f08e7a8a232b744a35753d3c60dd69830
F03_REPORT_AFTER=cb8b215f08e7a8a232b744a35753d3c60dd69830
F03R1_NON_MUTATING=PASS

## Full product regression

After canonical advancement to F05, the complete product gate was executed.

Observed:

CONTINUITY_OK current=F05 next=F06 terminal=F13 root=D:\TIMED-MISSION-AGENT

F04 focused tests in the full gate:
- 14 tests PASS / 0 FAIL;
- isolation probe PASS;
- contracts PASS;
- map 40/40 PASS;
- purity PASS.

Final result:

BASELINE_VERIFY=PASS

The F03 canonical release report remained unchanged because R03-01 routes live regression metrics outside the repository.

## Closure decision

F04 exit criteria are satisfied and the post-advancement full regression passed.

Canonical advancement:
- F04 → CLOSED
- F05 DURABLE_MISSION_RUNTIME → IN_PROGRESS
- F06 PLANNER_ROUTER_KNOWLEDGE → PLANNED

Release tag target:
v0.4.0-isolated-browser-worker
