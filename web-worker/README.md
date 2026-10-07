# Web Worker — F04

Purpose: isolated browser execution for TIMED-MISSION-AGENT.

## Runtime

- Node 24.
- Playwright 1.64.0.
- Playwright-managed Chromium.
- BrowserServer child process.
- Headless mode is enforced; non-headless launch is rejected.
- Each mission receives a new BrowserContext.
- Session persistence uses explicit Playwright storageState files.
- Evidence includes DOM snapshots, screenshots, downloads and error records.

## Dependency storage

Source remains under D:\TIMED-MISSION-AGENT.

Generated npm packages and browser binaries live outside the source tree:

- C:\ProgramData\SentinelX\workspace\tma-web-worker-deps
- C:\ProgramData\SentinelX\workspace\tma-playwright-browsers

web-worker\node_modules is a local junction to the dependency store and is gitignored.

Rebuild the local runtime with:

powershell -ExecutionPolicy Bypass -File D:\TIMED-MISSION-AGENT\scripts\bootstrap_f04_playwright.ps1

## Isolation

Browser automation does not use the operator mouse or keyboard. The verified runtime launches Chromium headless in a process/session isolated from the Explorer desktop.

The Computer Worker contract requires a dedicated Windows session or VM for tasks that genuinely require native desktop GUI interaction.

## Explicit non-goals

No CAPTCHA bypass, MFA/KYC bypass, stealth plugin, fingerprint spoofing, bot-detection evasion, or automation disguise is implemented.
