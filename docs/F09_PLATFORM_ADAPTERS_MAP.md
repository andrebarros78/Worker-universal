# F09 — Platform Adapters: Capability Map

Status: IN_PROGRESS
Scope: provider-neutral initial adapters with real F04 browser execution against an offline synthetic fixture.
External credentials, live submissions, qualification assessments, proposals and external requests are not part of this phase.

| ID | Requirement | Evidence |
|---|---|---|
| PAD-001 | Separate platform-adapters layer | platform-adapters/src |
| PAD-002 | F02 SDK-compatible manifests | manifests test |
| PAD-003 | Five adapter registry | registeredAdapters |
| PAD-004 | generic_web adapter | simulated test |
| PAD-005 | generic_form adapter | simulated test |
| PAD-006 | generic_invoice adapter | simulated test |
| PAD-007 | homeocta initial adapter | simulated test |
| PAD-008 | 99freelas initial adapter | simulated test |
| PAD-009 | F04 isolated Chromium delegation | BrowserRuntime |
| PAD-010 | Real browser execution per adapter | 5/5 test |
| PAD-011 | Evidence DOM capture | 5/5 test |
| PAD-012 | Evidence screenshot capture | 5/5 test |
| PAD-013 | Evidence SHA-256 verification | runner |
| PAD-014 | Evidence byte-size verification | runner |
| PAD-015 | Required evidence completeness | runner |
| PAD-016 | F03 OCR delegation contract | generic_invoice/homeocta |
| PAD-017 | F06 research delegation contract | 99freelas |
| PAD-018 | F07-compatible evidence envelope | outcome contract |
| PAD-019 | F08 experimental quarantine | health test |
| PAD-020 | Production/live execution fail closed | policy test |
| PAD-021 | Explicit platform.read scope | policy test |
| PAD-022 | Explicit practice.fill scope | policy test |
| PAD-023 | Domain-origin binding | policy test |
| PAD-024 | Loopback-only simulation | policy test |
| PAD-025 | URL credentials denied | policy |
| PAD-026 | No submit/click/upload in initial adapters | plan assertion |
| PAD-027 | No external side effects | outcomes/plan |
| PAD-028 | HomeOcta qualification is not automated | policy test |
| PAD-029 | 99Freelas proposal submission is not automated | policy test |
| PAD-030 | No bypass of account/platform eligibility | policy |
| PAD-031 | Recovery bounded to one real browser relaunch | 5/5 recovery test |
| PAD-032 | Browser crash injection for each adapter | real crash test |
| PAD-033 | Request guard persists after browser relaunch | WeakSet page guard |
| PAD-034 | Foreign-origin requests blocked, including redirects | runtime request guard |
| PAD-035 | Deadline exhaustion denies success | timeout test |
| PAD-036 | Simulation result distinct from durable success | status simulated |
| PAD-037 | JSON config contract | schema/fixture |
| PAD-038 | JSON plan contract | schema/fixture |
| PAD-039 | JSON outcome contract | schema/fixture |
| PAD-040 | Five versioned F02 adapter manifest fixtures | schema gate |
| PAD-041 | Cross-adapter independent tests | F09 test |
| PAD-042 | Dependency-free platform package | package.json |
| PAD-043 | Platform SDK isolation from core | purity gate |
| PAD-044 | Single phase verification gate | verify_f09.ps1 |
| PAD-045 | Baseline integration and regression | BASELINE_VERIFY |
| PAD-046 | No external authentication assumptions | simulation-only |
| PAD-047 | No CAPTCHA/MFA/KYC evasion | purity gate |
| PAD-048 | Canonical proof and release | PROOF_F09_PLATFORM_ADAPTERS |
| PAD-049 | F09→F10 state transition after proof | continuity |
| PAD-050 | Git/GitHub release seal | commit/tag/remote proof |

## Authority

tma-core owns mission truth. F02 owns capability registry and policy. F03 owns OCR. F04 owns the browser process. F07 owns independent mission validation. F08 owns capability qualification. F09 owns provider-specific adapter selection and safety constraints only.

The initial HomeOcta and 99Freelas adapters are **SIMULATION_ONLY**. They are not representations of an approved, authenticated or live integration. Live platform eligibility and authorized integration remain explicit future dependencies.
