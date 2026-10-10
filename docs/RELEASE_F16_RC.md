# TIMED-MISSION-AGENT — F16 Portable Engineering Release Candidate

**Tag:** `v1.0.0-rc.2-f16-portable`  
**Status:** SOURCE PRERELEASE — NOT PRODUCTION ACCEPTED  
**Canonical project:** `D:\TIMED-MISSION-AGENT`  
**Current productive phase:** F13 IN_PROGRESS; `MISSION_PROVEN=false`.

## Reconciled product

The F16 Windows x64 candidate reconciles 24 project components: nine Rust executables, Go supervision, Node.js 24.18.0 and Chromium, Python 3.14.4/Pillow 11.3.0, Tesseract OCR 5.4, Elixir 1.20.4/Erlang OTP 29, adapters, JSON contracts and Ada/SPARK source. Machine-specific credentials and live databases are excluded from the distribution. Browser integrations remain origin-scoped and require native Edge extension approval.

## Verified engineering gates

- Reconciled modules: **24**.
- Browser regression: **14/14 PASS**.
- F14 background reader / portability: **18/18 PASS**.
- Python/OCR tests: **22/22 PASS**.
- Operator authorization: **12/12 PASS**.
- Platform adapters: **8/8 PASS**.
- Elixir tests: **2/2 PASS**.
- Rust and Go supported native self-tests: **PASS**.
- Archive extracted on same physical PC into a separate location, then `START_TMA.cmd verify` passed without Node, Python or Go available in the system PATH.
- **2,491/2,491** archive files passed SHA-256 verification.

### Locally sealed distribution candidate — not attached to this GitHub release

`TIMED-MISSION-AGENT-F16-WIN-X64.zip`  
**377,255,886 bytes**  
SHA-256: `e4b4df4ea031dfed62611b1c4a769e972bc26622324b7a7bcaec6b421b2ab450`.

The tested ZIP is held on the PC Vendas engineering host. Binary runtime redistribution notices and licenses remain to be reviewed before attaching that package publicly; the public GitHub prerelease provides **source only**.

## Open acceptance requirements

1. Edge F14 extension installed and authorized in the operator's existing authenticated session.
2. Worker independently reads a real HomeOcta lesson, assesses learning and recalls it after restart.
3. Tests on a **second physical computer**, not just a separate folder/profile.
4. Physical integrity of canonical USB D: verified before promoting changes; prior NTFS errors occurred.
5. Runtime redistribution/license notices reviewed before publishing the 377 MB binary ZIP.
6. Ada/SPARK formal proof not claimed; GNATprove not run.
7. F13 economic outcome, productive recovery and complete end-to-end mission acceptance are still pending.

**This tag denotes the engineering source release candidate, not a production release.** Do not change F13's canonical `IN_PROGRESS` pointer or claim `MISSION_PROVEN`.
