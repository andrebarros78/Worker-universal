# Remediation R07-01 — Concurrent Verification Gate Isolation

Status: CLOSED
Discovered during: F07 closure/post-release reconciliation
References: F03, F05, F07

## Finding

Concurrent verification runs on PC Vendas exposed collisions in test-only runtime state:

- F03 compile runs shared Python bytecode cache paths;
- F03 corpus regeneration is intentionally canonical/shared and cannot be mutated concurrently;
- F05 process-recovery drills shared one fixed SQLite runtime directory;
- F07 process-restart drills initially shared one fixed SQLite runtime directory;
- one concurrent lane also inserted a duplicate F07 baseline gate entry.

These were verification-harness isolation defects. Product mission state, fencing, validation and recovery logic remained valid.

## Corrections

- baseline Python bytecode cache is GUID-scoped under C:\ProgramData\SentinelX\workspace\tma-pycache;
- F03 live report/pycache use a GUID-scoped runtime directory;
- F03 canonical corpus verification is serialized with Global\TMA_F03_VERIFY_GATE;
- F05 recovery drill uses a GUID-scoped runtime directory;
- F07 recovery drill uses a GUID-scoped runtime directory and cleans its worker/runtime in finally;
- duplicate F07 baseline gate entry was removed.

Implementation hardening commit:

0a7d919511603dd335d26f85a7510b7a88349c5d — test: isolate concurrent regression runtimes

F07 release proof/tag remain immutable:

965e8a9e10ccca17e8b254249bf8626b6b04aa64
v0.5.2-independent-validation-recovery

## Concurrent proof

### F03

Two verify_f03.ps1 processes started concurrently.

Run 1:
- F03_RUNTIME=8b8837c769c54e46ae4c0160985a2f84
- F03_VERIFY=PASS

Run 2:
- F03_RUNTIME=be10cbd724ad4317858eb9f70774fad7
- F03_VERIFY=PASS

Result:

F03_CONCURRENT_GATE=PASS runs=2

### F05 + F07

Four process-recovery drills ran concurrently.

F05 run 1:
- runtime 32a11b8094b94d0da38c64b2f6186a14
- F05_PROCESS_RECOVERY=PASS

F05 run 2:
- runtime 764252c80f4c48c3b678b60c84cdab2d
- F05_PROCESS_RECOVERY=PASS

F07 run 1:
- runtime e97d8686f0c14e56b395946c55d8ec12
- F07_RECOVERY_RESTART=PASS

F07 run 2:
- runtime bd690d0614494f84a56bd145eb9416f5
- F07_RECOVERY_RESTART=PASS

Result:

F05_F07_CONCURRENT_RECOVERY=PASS runs=4

## Final regression

After the isolation fixes:

CONTINUITY_OK current=F08 next=F09 terminal=F13

F03_VERIFY=PASS
F04_VERIFY=PASS
F05_VERIFY=PASS
F06_VERIFY=PASS
F07_VERIFY=PASS
F07_FAILURE_MATRIX=PASS classes=16
F07_RECOVERY_RESTART=PASS
SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED
BASELINE_VERIFY=PASS

This remediation does not reopen F03, F05 or F07.
