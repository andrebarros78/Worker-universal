use tma_qualification::{
    BenchmarkCase, BenchmarkExecutor, BenchmarkSample, QualificationThresholds, RunMode,
    SyntheticHarness, build_scorecard,
};

struct LocalExecutor;

impl BenchmarkExecutor for LocalExecutor {
    fn execute(&mut self, case: &BenchmarkCase) -> BenchmarkSample {
        BenchmarkSample {
            case_id: case.case_id.clone(),
            success: true,
            accuracy_bps: case.expected_accuracy_bps,
            validation_proven: true,
            latency_ms: 10,
            recovered: case.should_recover,
            timed_out: false,
        }
    }
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.as_slice() != ["self-test"] {
        eprintln!("usage: tma-qualification self-test");
        std::process::exit(2);
    }
    let cases = (0..8)
        .map(|index| BenchmarkCase {
            case_id: format!("case-{index}"),
            dataset_id: "self-test".to_string(),
            dataset_version: "1.0.0".to_string(),
            seed: 42,
            deadline_ms: 100,
            expected_accuracy_bps: 10_000,
            should_recover: index == 0,
        })
        .collect::<Vec<_>>();
    let samples = SyntheticHarness.run(&cases, &mut LocalExecutor);
    let scorecard = build_scorecard(
        "native.self-test",
        RunMode::Benchmark,
        &cases,
        &samples,
        &QualificationThresholds {
            minimum_runs: 8,
            minimum_success_bps: 10_000,
            minimum_accuracy_bps: 9_900,
            minimum_validation_bps: 10_000,
            maximum_p95_ms: 100,
            require_recovery_drill: true,
        },
    )
    .expect("scorecard");

    println!(
        "tma-qualification status=ok passed={} runs={} p95_ms={} fingerprint={}",
        scorecard.passed, scorecard.runs, scorecard.p95_ms, scorecard.fingerprint
    );
}
