# Proof — F08 Training, Benchmark and Qualification Gates

Date: 2026-10-08
Host: PC Vendas
Canonical root: D:\TIMED-MISSION-AGENT

## Objective

Prove that capabilities are exercised in controlled timed training and benchmark flows before promotion, and that production readiness requires sequential promotion, synthetic registration lifecycle, independent F07 validation and recovery evidence.

## Architecture

F08 is implemented as:

qualification-rust

Authority boundaries:

- tma-foundation remains capability registry and promotion-state authority;
- tma-qualification owns benchmark scorecards, qualification receipts and synthetic registration lifecycle;
- tma-validation remains independent validation/recovery authority;
- no platform or provider SDK is embedded in qualification-rust.

## Requirement map

- docs/F08_TRAINING_BENCHMARK_QUALIFICATION_MAP.md
- QUA-001..QUA-060
- docs/F08_IMPLEMENTATION_MATRIX.md

Observed:

F08_MAP_OK requirements=60 mapped=60

## Delivered

- Training mode;
- Benchmark mode;
- deterministic synthetic benchmark harness;
- real wall-clock harness;
- per-case deadlines;
- dataset/version/seed binding;
- success, accuracy and F07-validation thresholds;
- p50/p95/p99 latency scorecard;
- recovery drill gate;
- deterministic SHA-256 scorecard fingerprint;
- synthetic PRE_REGISTRATION → REGISTERED → POST_REGISTRATION → PRODUCTION_READY lifecycle;
- qualification receipts bound to capability, scorecard and registration state;
- Experimental → Tested training gate;
- Tested → Qualified benchmark gate;
- Qualified → Production post-registration gate;
- stale/rejected receipt rejection;
- F07 ValidationReport binding for benchmark samples;
- scanner preventing promotion mutation in core/planner/validation production code.

## Remediation R02-01

F08 discovered that the original F02 CapabilityRegistry allowed arbitrary promotion assignments.

Correction:

Experimental → Tested → Qualified → Production is now the only allowed upward sequence.

Observed:
- direct Experimental → Production rejected;
- sequential promotion accepted;
- demotion remains available for safety;
- current F02 suite: 22 PASS / 0 FAIL;
- F02_VERIFY=PASS.

Proof:
docs/REMEDIATION_F02R1_SEQUENTIAL_PROMOTION_GATE.md

## Focused qualification tests

Observed:

19 tests PASS / 0 FAIL

Covered:
- 5-run deterministic reproducibility;
- wall-clock deadline timeout;
- percentile/rate scorecards;
- minimum run count;
- success threshold;
- accuracy threshold;
- F07 validation threshold;
- p95 threshold;
- recovery drill gate;
- dataset/version/seed binding;
- stable scorecard serialization;
- ordered registration lifecycle;
- failed scorecard rejection;
- capability/session binding;
- full sequential promotion;
- direct promotion skip rejection;
- safety demotion;
- stale receipt rejection;
- rejected receipt rejection;
- production readiness/recovery requirement;
- deterministic receipt serialization;
- benchmark sample binding to F07 ValidationReport.

## Dedicated gate

Observed:

F08_CONTRACTS_OK fixtures=4 benchmark=timed lifecycle=synthetic promotion=sequential
F08_MAP_OK requirements=60 mapped=60
F08_PURITY_OK provider_sdks=0 platform_apis=0 f07_validation_bound=1 promotion_bypass=0 sequential_registry_gate=1
F08_VERIFY=PASS

Self-test:

tma-qualification status=ok passed=true runs=8 p95_ms=10 fingerprint=7506ac281363ef42616f6bf520f77f5b13d731ec74ba9c5f6b3de9de28bc6b8f

## Full product regression with F08 active

Observed:

CONTINUITY_OK current=F08 next=F09 terminal=F13 root=D:\TIMED-MISSION-AGENT

F08_VERIFY=PASS

SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED

BASELINE_VERIFY=PASS

## Final post-advancement regression

Observed after the canonical pointer advanced to F09:

CONTINUITY_OK current=F09 next=F10 terminal=F13 root=D:\TIMED-MISSION-AGENT

F02_VERIFY=PASS
F08_CONTRACTS_OK fixtures=4 benchmark=timed lifecycle=synthetic promotion=sequential
F08_MAP_OK requirements=60 mapped=60
F08_PURITY_OK provider_sdks=0 platform_apis=0 f07_validation_bound=1 promotion_bypass=0 sequential_registry_gate=1
F08_VERIFY=PASS

SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED

BASELINE_VERIFY=PASS

## Closure decision

F08 exit criteria are satisfied. The post-advancement full regression passed.

Canonical advancement:
- F08 → CLOSED
- F09 PLATFORM_ADAPTERS → IN_PROGRESS
- F10 ECONOMIC_CONTROLLER_SCHEDULER → PLANNED

Release tag:

v0.6.0-training-qualification
