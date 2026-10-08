# Remediation F04R1 — Deadline-Limited Timeout Classification

Status: CLOSED
Discovered during: F05 final regression
References: F04 ISOLATED_BROWSER_COMPUTER_WORKER

## Finding

The legacy TimedBrowserWorker uses one timer for both per-step timeout and the remaining mission deadline.

When remaining mission time was smaller than the configured step timeout, the timer correctly used the remaining deadline. Under scheduler jitter it could fire just before the next high-resolution elapsed-time read crossed the deadline boundary. The catch path then classified the event as failed instead of deadline_exceeded.

This produced an intermittent regression in test/worker.test.ts while the browser-worker F04 runtime itself remained healthy.

## Correction

The worker now records whether the effective timer was deadline-limited.

Classification rule:
- ordinary per-step timer expiration -> failed;
- timer bounded by remaining mission deadline -> deadline_exceeded;
- elapsed time already beyond mission deadline -> deadline_exceeded.

No threshold was relaxed and no test was weakened.

## Proof

Observed after correction:

- legacy worker test file executed 5 consecutive times;
- each run: 3 PASS / 0 FAIL;
- R04_01_REPEAT_5=PASS;
- F04 focused gate: 14 PASS / 0 FAIL;
- F04_VERIFY=PASS;
- operator isolation probe PASS;
- full product gate after F05 advancement: BASELINE_VERIFY=PASS.

This remediation does not reopen F04.
