# Remediation R02-01 — Sequential Promotion Gate

Status: CLOSED
Discovered during: F08
References: F02 CAPABILITY_FOUNDATION

## Finding

CapabilityRegistry::set_promotion accepted arbitrary promotion-state assignments, allowing a direct Experimental → Production jump and bypassing the qualification lifecycle required by F08.

## Correction

- upward promotions are limited to exactly one state at a time:
  Experimental → Tested → Qualified → Production;
- idempotent same-state assignment remains allowed;
- demotion remains allowed for safety/quarantine;
- F08 QualificationAuthority issues receipts and applies only the next eligible promotion.

## Proof

Observed:

- direct Experimental → Production rejected;
- Experimental → Tested → Qualified → Production succeeds sequentially;
- demotion from Production to Tested remains possible for safety;
- F02 current suite: 22 PASS / 0 FAIL;
- F02_VERIFY=PASS;
- F08 lifecycle cannot reach Production without training, benchmark, registration and recovery gates;
- F08_VERIFY=PASS;
- full BASELINE_VERIFY=PASS.

This remediation does not reopen F02.
