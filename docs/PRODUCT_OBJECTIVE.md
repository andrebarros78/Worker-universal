# Sovereign product objective

## Why this exists

TIMED-MISSION-AGENT is a personal income-generation system.

Its practical objective is to transform eligible automatable work into reliable completed missions with the lowest possible operator burden. The business result is not "automation for its own sake": it is dependable income that supports the operator's household.

## Product objective

Receive a work objective, decompose it into timed missions, execute the technical work, validate the result independently, recover from technical failures, preserve evidence, and continue until success or a genuine external dependency prevents progress.

## Engineering priorities

1. Correct result before apparent activity.
2. Deadline/SLA is a hard invariant, not a dashboard metric.
3. Failed workers must be replaceable without losing mission state.
4. Evidence must survive process restart.
5. Each language has one bounded responsibility.
6. Cross-language communication uses versioned contracts.
7. No component may declare success if the deterministic core rejects it.
8. Concurrency must not produce duplicate submission or contradictory state.
9. Runtime failures trigger recovery, not silent abandonment.
10. The system should minimize ongoing maintenance cost because this is a personal product.

## Income control plane

Future production scoring:

expected_value = probability_of_success * expected_revenue - compute_cost - API_cost - expected_rework_cost

A mission should be prioritized only when it meets the configured economic threshold and technical eligibility rules.
