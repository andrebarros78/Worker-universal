# TIMED-MISSION-AGENT — Modular Architecture Release Candidate

**Version:** `v1.0.0-rc.3-modular` — product architecture `M1–M16`  
**Status:** Engineering prerelease. Not production homologated; `MISSION_PROVEN=false`.

## Architectural correction

- **M = permanent product module.** The product registry `architecture/PRODUCT_MODULES.json` defines sixteen stable identifiers `M1` through `M16`, with each module's functional responsibility, implementation paths, dependencies, local verification status and independent production-acceptance status.
- **F = construction phase or engineering activity**, never a product module. Historic F00–F13 remain in `docs/PHASE_LEDGER.md`. Historical F14–F16 references were correction and distribution work packages, not new canonical active phases. The trace lives in `docs/engineering/PHASE_MODULE_TRACEABILITY.md`.
- `M1 = Cérebro e Decisão`, `M2 = Core de Missões`, `M3 = Capacidades e Ferramentas`; the remaining `M4–M16` are documented in `docs/PRODUCT_ARCHITECTURE_MODULES.md`.
- `START_TMA.cmd modules` lists product modules; `START_TMA.cmd module M1` explains responsibilities; `START_TMA.cmd verify` refuses missing implementations, duplicate IDs, cycles and F identifiers as modules.
- Historical phase-named error identifiers remain for backward compatibility with existing contracts; no such identifier is registered as a product module. They must be migrated only under explicit compatibility gates, never with blind global substitution.

## Tests and distribution evidence

- **5/5** product-module architecture tests passed.
- **23/23** combined module/F14 tests passed.
- **14/14** original browser tests passed.
- ZIP extracted and `START_TMA.cmd verify` passed with Node, Python and Go unavailable from system `PATH`.
- `START_TMA.cmd module M2` succeeded, and `START_TMA.cmd module F14` was correctly rejected.
- ZIP integrity: **2,496/2,496** source/runtime files verified by SHA-256 inside the archive.

Local engineering ZIP on PC Vendas (not uploaded as a public GitHub asset):
`TIMED-MISSION-AGENT-MODULAR-RC3-WIN-X64.zip`  
**377,263,094 bytes**, SHA-256:
`8efa34ce2df3236820a75e7d2270e2ff6b63ee09cbdbcd7454b4f07c59ec6a55`.

Binary redistribution/license review is still pending. Source prerelease only.

## Product-acceptance boundaries

Authenticated HomeOcta lesson capture, semantic learning, browser extension approval, second physical PC, Ada formal proof, USB `D:` physical reliability and end-to-end production are **not proven**. Canonical `D:\TIMED-MISSION-AGENT` and `main` remain unchanged. Do not claim stable release or `MISSION_PROVEN`.
