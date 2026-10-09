# Proof — F11 Availability and Concurrency Hardening

Date: 2026-10-08
Host: PC Vendas (Vendas)
Canonical root: D:\TIMED-MISSION-AGENT

## Architecture / authority

- F11 availability-rust v0.8.0 consumes F05 DurableStore; it does not replace F05 transaction/lease authority.
- Real SQLite claim_next with BEGIN IMMEDIATE, per-mission F05 lease fencing and checkpoints, event chain and integrity checks.
- F11 Go Supervisor worker pool with capacity guard, resource-key mutex, duplicate-ID admission check, bounded retry policy, context cancellation, progress-based watchdog and health endpoints. The serve-health command binds 127.0.0.1.
- F07 independent validation receipt remains required to complete durable Succeeded; F11 does not invent a validated result.
- F10 financial/event accounting remains separately validated and immutable.
- Elixir/OTP outer supervision is optional and was not added in F11.

## Focused tests

- Rust matrix tiers: 1 → 4 → 8 → 16 workers.
- One SQLite queue per tier; no duplicate claims, no lost claims, fence token = 1 per first claim, Running states, verified checkpoints, SQLite integrity_check=ok, F05 event-chain verification.
- Rust focused tests: 4 PASS, 0 FAIL; fmt and clippy -D warnings PASS.
- Go 1/4/8/16 worker-pool matrix, per-resource lock, bounded restarts, backpressure, cancellation, duplicate task ID rejection, watchdog stale detection, and health endpoint test.
- Go tests, vet, go test -race PASS.
- HTTP /healthz=200 /readyz=200 tested using a real server on a loopback ephemeral port.
- Real process termination/recovery through existing F05 durable-probe: planned/running/validating PASS, recovery fencing 1 → 2 and checkpoints preserved.
- F11_CONTRACTS_OK fixtures=2 tiers=1,4,8,16.
- F11_MAP_OK requirements=56 mapped=56.
- F11_PURITY_OK durable_authority=F05 retries=bounded resource_locks=1 queue_backpressure=1 http=loopback only.
- Dedicated F11_VERIFY=PASS.
- 30-second soak within F11 dedicated gate: 19 cycles / 608 missions / tiers=4 / integrity=ok.
- Separate initial 30-second soak: 27 cycles / 864 missions / tiers=4 / integrity=ok.

## Independent extended soak

F11_SOAK=PASS seconds=120 cycles=100 missions=3200 tiers_covered=4 integrity=ok.

100 complete synthetic concurrency cycles executed across 1/4/8/16 worker configurations, with 3,200 mission claim/checkpoint verifications and no lost/duplicate claims or broken SQLite hash chain. This is a 120-second test, not evidence of multi-day uptime.

## Baseline regression with F11 active

CONTINUITY_OK current=F11 next=F12 terminal=F13 root=D:\TIMED-MISSION-AGENT
F11_VERIFY=PASS
SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED
BASELINE_VERIFY=PASS

## Regression after canonical advancement

CONTINUITY_OK current=F12 next=F13 terminal=F13 root=D:\TIMED-MISSION-AGENT
F11_CONTRACTS_OK fixtures=2 tiers=1,4,8,16
F11_MAP_OK requirements=56 mapped=56
F11_PURITY_OK durable_authority=F05 retries=bounded resource_locks=1 queue_backpressure=1 http=loopback only
F11_VERIFY=PASS
SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED
BASELINE_VERIFY=PASS


## Limits and non-claims

- All concurrency and fault-injection cases are synthetic test missions, not real paid platform jobs.
- F11 only qualifies concurrent claims and bounded supervision, not 24h/7d uptime.
- An uncooperative Go runner that ignores context cancellation cannot be forcibly terminated safely in-process. The watchdog detects stalled progress but does not by itself kill external processes; full operational process isolation is subject to F12.
- No new platform credentials, tokens, or automatic external actions were introduced.
- GNATprove formal proof remains unavailable unless F12 installs/verifies tooling.

## Closure decision

F11 exit gate and 120-second extended soak passed; post-advancement F12 full regression also passed.

F11 CLOSED. F12 SECURITY_FORMAL_OPERATIONAL_HARDENING IN_PROGRESS; F13 PERSONAL_PRODUCTION_V1_0 PLANNED.

Release tag: v0.8.0-availability-concurrency.
