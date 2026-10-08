# Proof — F10 Economic Controller and Scheduler

Date: 2026-10-08
Host: PC Vendas
Root: D:\TIMED-MISSION-AGENT

## Scope and architecture

Economic accounting is a separate provider-neutral Rust crate, economic-rust v0.7.0.
Money is measured in BRL cents (i64), with intermediate i128 arithmetic for overflow safety.
F02 CapabilityRegistry controls production scheduling eligibility.
F05 durable Succeeded + independent F07 validation and matching terminal-result hash is required before verified earnings are entered.
F08 qualification promotion cannot be replaced by an adapter-provided qualified=true flag.
F09 platform adapters remain simulation-only; no real platform revenue or payment is claimed.

## Components delivered

- Opportunity model and observed success probability derived from unique F05 terminal mission records.
- Deterministic expected reward, expected profit, margin bps, profit/hour and stable SHA-256 plan fingerprint.
- Scheduler with eligibility, F02 Production/Healthy/Operational checks, deadline, per-job cap, daily cost cap, reserved-budget, minimum expected profit, minimum margin, stop-loss, bounded concurrency and resource locks.
- SQLite WAL/FULL append-only ledger with idempotent keys, SHA-256 hash chain, no UPDATE/DELETE triggers and restart/reopen integrity.
- One verified earning per independently validated F05 Succeeded mission; exact F05 terminal hash binding.
- Payment and earnings separated; duplicate bank settlement references rejected; payment cannot exceed verified accounts receivable.
- Cost records, daily costs_since and cash/accrual/account-receivable report.
- JSON dashboard + local read-only CLI: tma-economic report <ledger.sqlite3>.
- 16 concurrent writers on same idempotency key: exactly one cost record.
- F10 policy, opportunity, event, schedule and report JSON contract fixtures.

## Verified tests

23 focused Rust tests PASS, 0 FAIL.
Core scheduler tests: 20 PASS.
Controller/F02/F05/report tests: 3 PASS.
16 parallel writer test PASS.
Cargo fmt --check PASS.
Cargo clippy --all-targets -- -D warnings PASS.

F10_CONTRACTS_OK fixtures=5 monetary_unit=BRL_cents
F10_MAP_OK requirements=55 mapped=55
F10_PURITY_OK vendor_sdks=0 f02_registry=1 f05_validated_success=1 append_only_ledger=1 fixed_point=1
F10_VERIFY=PASS

Self-test:

tma-economic status=ok selected=1 expected_profit_cents=3000 fingerprint=a264ac35f10e18cd712a06395b2671bc4bd749375dca8b2c01ef5fa1289bc5ff

## Full baseline with F10 current

CONTINUITY_OK current=F10 next=F11 terminal=F13 root=D:\TIMED-MISSION-AGENT
F10_VERIFY=PASS
SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED
BASELINE_VERIFY=PASS

## Limitations and non-claims

- No productive platform integration or monetary transaction was performed.
- F05 success validates a mission result, not proof that a client actually paid.
- A payment_received event is entered with a supplied settlement reference; F10 does not independently query a bank or payment API. Therefore the JSON report distinguishes recorded cash from verified earnings and must not be presented as externally reconciled cash.
- Scheduler is an economic selection plan, not the F05 dispatch/claim authority.
- A production adapter must independently pass F08 and platform eligibility/authorization gates.

## Closure

F10 exit gate passed, including full post-advancement regression:

CONTINUITY_OK current=F11 next=F12 terminal=F13 root=D:\TIMED-MISSION-AGENT
F10_CONTRACTS_OK fixtures=5 monetary_unit=BRL_cents
F10_MAP_OK requirements=55 mapped=55
F10_PURITY_OK vendor_sdks=0 f02_registry=1 f05_validated_success=1 append_only_ledger=1 fixed_point=1
F10_VERIFY=PASS
SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED
BASELINE_VERIFY=PASS

Canonical advancement: F10 CLOSED → F11 AVAILABILITY_CONCURRENCY_HARDENING IN_PROGRESS → F12 SECURITY_FORMAL_OPERATIONAL_HARDENING PLANNED.
Release tag: v0.7.0-economic-controller.
