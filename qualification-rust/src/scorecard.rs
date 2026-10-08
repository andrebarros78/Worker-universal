use sha2::{Digest, Sha256};

use crate::model::{BenchmarkCase, BenchmarkSample, QualificationThresholds, RunMode, Scorecard};

pub fn build_scorecard(
    capability_id: &str,
    mode: RunMode,
    cases: &[BenchmarkCase],
    samples: &[BenchmarkSample],
    thresholds: &QualificationThresholds,
) -> Result<Scorecard, String> {
    if capability_id.trim().is_empty() {
        return Err("capability_id required".to_string());
    }
    if cases.is_empty() || cases.len() != samples.len() {
        return Err("cases/samples length mismatch".to_string());
    }
    if thresholds.minimum_runs == 0 || thresholds.minimum_runs > cases.len() {
        return Err("invalid minimum_runs".to_string());
    }

    let dataset_id = cases[0].dataset_id.clone();
    let dataset_version = cases[0].dataset_version.clone();
    let seed = cases[0].seed;
    if cases.iter().any(|case| {
        case.dataset_id != dataset_id
            || case.dataset_version != dataset_version
            || case.seed != seed
    }) {
        return Err("dataset/seed mismatch".to_string());
    }
    for (case, sample) in cases.iter().zip(samples) {
        if case.case_id != sample.case_id {
            return Err("case/sample id mismatch".to_string());
        }
    }

    let runs = samples.len();
    let success = samples
        .iter()
        .filter(|item| item.success && !item.timed_out)
        .count();
    let validation = samples.iter().filter(|item| item.validation_proven).count();
    let recovered_runs = samples.iter().filter(|item| item.recovered).count();
    let recovery_required_cases = cases.iter().filter(|item| item.should_recover).count();
    let recovery_passed = if recovery_required_cases == 0 {
        !thresholds.require_recovery_drill
    } else {
        recovered_runs >= recovery_required_cases
    };

    let success_bps = ratio_bps(success, runs);
    let validation_bps = ratio_bps(validation, runs);
    let accuracy_bps = samples
        .iter()
        .map(|item| u64::from(item.accuracy_bps))
        .sum::<u64>()
        .checked_div(runs as u64)
        .unwrap_or(0) as u16;

    let mut latencies = samples
        .iter()
        .map(|item| item.latency_ms)
        .collect::<Vec<_>>();
    latencies.sort_unstable();
    let p50_ms = percentile(&latencies, 50);
    let p95_ms = percentile(&latencies, 95);
    let p99_ms = percentile(&latencies, 99);

    let mut failures = Vec::new();
    if runs < thresholds.minimum_runs {
        failures.push("minimum_runs".to_string());
    }
    if success_bps < thresholds.minimum_success_bps {
        failures.push("success_rate".to_string());
    }
    if accuracy_bps < thresholds.minimum_accuracy_bps {
        failures.push("accuracy".to_string());
    }
    if validation_bps < thresholds.minimum_validation_bps {
        failures.push("validation".to_string());
    }
    if p95_ms > thresholds.maximum_p95_ms {
        failures.push("p95_latency".to_string());
    }
    if thresholds.require_recovery_drill && !recovery_passed {
        failures.push("recovery_drill".to_string());
    }

    let fingerprint = scorecard_fingerprint(
        capability_id,
        mode,
        &dataset_id,
        &dataset_version,
        seed,
        samples,
        thresholds,
    );

    Ok(Scorecard {
        capability_id: capability_id.to_string(),
        mode,
        dataset_id,
        dataset_version,
        seed,
        runs,
        success_bps,
        accuracy_bps,
        validation_bps,
        p50_ms,
        p95_ms,
        p99_ms,
        recovered_runs,
        recovery_passed,
        passed: failures.is_empty(),
        failures,
        fingerprint,
    })
}

pub fn canonical_scorecard(scorecard: &Scorecard) -> String {
    format!(
        "capability={}|mode={:?}|dataset={}|version={}|seed={}|runs={}|success={}|accuracy={}|validation={}|p50={}|p95={}|p99={}|recovered={}|recovery_passed={}|passed={}|failures={}|fingerprint={}",
        scorecard.capability_id,
        scorecard.mode,
        scorecard.dataset_id,
        scorecard.dataset_version,
        scorecard.seed,
        scorecard.runs,
        scorecard.success_bps,
        scorecard.accuracy_bps,
        scorecard.validation_bps,
        scorecard.p50_ms,
        scorecard.p95_ms,
        scorecard.p99_ms,
        scorecard.recovered_runs,
        scorecard.recovery_passed,
        scorecard.passed,
        scorecard.failures.join(","),
        scorecard.fingerprint,
    )
}

fn ratio_bps(count: usize, total: usize) -> u16 {
    if total == 0 {
        return 0;
    }
    ((count as u64 * 10_000) / total as u64) as u16
}

fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = ((percentile * sorted.len()).div_ceil(100)).max(1);
    sorted[rank - 1]
}

fn scorecard_fingerprint(
    capability_id: &str,
    mode: RunMode,
    dataset_id: &str,
    dataset_version: &str,
    seed: u64,
    samples: &[BenchmarkSample],
    thresholds: &QualificationThresholds,
) -> String {
    let mut canonical_samples = samples.to_vec();
    canonical_samples.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    let sample_text = canonical_samples
        .iter()
        .map(|item| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}",
                item.case_id,
                item.success,
                item.accuracy_bps,
                item.validation_proven,
                item.latency_ms,
                item.recovered,
                item.timed_out
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let canonical = format!(
        "capability={capability_id}|mode={mode:?}|dataset={dataset_id}|version={dataset_version}|seed={seed}|thresholds={:?}\n{sample_text}",
        thresholds
    );
    let digest = Sha256::digest(canonical.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
