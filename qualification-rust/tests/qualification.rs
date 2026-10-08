use std::thread;
use std::time::Duration;

use tma_foundation::model::{
    CapabilityDescriptor, CapabilityKind, IdempotencyClass, LifecycleState, PromotionState,
    ReadinessState, SideEffectClass, TransportKind, Version,
};
use tma_foundation::registry::CapabilityRegistry;
use tma_qualification::{
    BenchmarkCase, BenchmarkExecutor, BenchmarkSample, QualificationAuthority,
    QualificationSession, QualificationThresholds, RegistrationState, RunMode, SyntheticHarness,
    WallClockHarness, build_scorecard, canonical_receipt, canonical_scorecard,
    sample_from_validation,
};
use tma_validation::{EvidenceCompleteness, ValidationOutcome, ValidationReport};

fn cases(count: usize, recover_index: Option<usize>) -> Vec<BenchmarkCase> {
    (0..count)
        .map(|index| BenchmarkCase {
            case_id: format!("case-{index:02}"),
            dataset_id: "dataset-a".to_string(),
            dataset_version: "1.0.0".to_string(),
            seed: 42,
            deadline_ms: 50,
            expected_accuracy_bps: 9_900,
            should_recover: recover_index == Some(index),
        })
        .collect()
}

fn thresholds(recovery: bool) -> QualificationThresholds {
    QualificationThresholds {
        minimum_runs: 8,
        minimum_success_bps: 9_500,
        minimum_accuracy_bps: 9_500,
        minimum_validation_bps: 10_000,
        maximum_p95_ms: 100,
        require_recovery_drill: recovery,
    }
}

fn descriptor(id: &str, promotion: PromotionState) -> CapabilityDescriptor {
    CapabilityDescriptor {
        id: id.to_string(),
        kind: CapabilityKind::Native,
        provider: Some("local".to_string()),
        contract_version: Version::new(1, 0, 0),
        dependencies: Vec::new(),
        permissions: Vec::new(),
        fallbacks: Vec::new(),
        lifecycle: LifecycleState::Healthy,
        readiness: ReadinessState::Operational,
        promotion,
        side_effect: SideEffectClass::None,
        idempotency: IdempotencyClass::Safe,
        max_concurrency: 8,
        resource_locks: Vec::new(),
        cost_microunits: 0,
        p95_ms: 10,
        reliability_bps: 10_000,
        evidence_types: vec!["result".to_string()],
        transport: TransportKind::Native,
        config_ref: None,
        enabled: true,
    }
}

#[derive(Clone)]
struct DeterministicExecutor {
    fail_index: Option<usize>,
    low_accuracy_index: Option<usize>,
    invalid_validation_index: Option<usize>,
    latency_base: u64,
}

impl BenchmarkExecutor for DeterministicExecutor {
    fn execute(&mut self, case: &BenchmarkCase) -> BenchmarkSample {
        let index = case
            .case_id
            .split('-')
            .next_back()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        BenchmarkSample {
            case_id: case.case_id.clone(),
            success: self.fail_index != Some(index),
            accuracy_bps: if self.low_accuracy_index == Some(index) {
                5_000
            } else {
                case.expected_accuracy_bps
            },
            validation_proven: self.invalid_validation_index != Some(index),
            latency_ms: self.latency_base + index as u64,
            recovered: case.should_recover,
            timed_out: false,
        }
    }
}

fn passing_scorecard(mode: RunMode, recovery: bool) -> tma_qualification::Scorecard {
    let suite = cases(8, recovery.then_some(0));
    let mut executor = DeterministicExecutor {
        fail_index: None,
        low_accuracy_index: None,
        invalid_validation_index: None,
        latency_base: 10,
    };
    let samples = SyntheticHarness.run(&suite, &mut executor);
    build_scorecard("native.cap", mode, &suite, &samples, &thresholds(recovery)).unwrap()
}

#[test]
fn synthetic_harness_is_reproducible_five_times() {
    let suite = cases(8, Some(0));
    let gate = thresholds(true);
    let mut fingerprints = Vec::new();

    for _ in 0..5 {
        let mut executor = DeterministicExecutor {
            fail_index: None,
            low_accuracy_index: None,
            invalid_validation_index: None,
            latency_base: 10,
        };
        let samples = SyntheticHarness.run(&suite, &mut executor);
        let score =
            build_scorecard("native.cap", RunMode::Benchmark, &suite, &samples, &gate).unwrap();
        assert!(score.passed);
        fingerprints.push(score.fingerprint);
    }

    assert!(fingerprints.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn wall_clock_harness_enforces_case_deadline() {
    let case = BenchmarkCase {
        case_id: "wall".to_string(),
        dataset_id: "d".to_string(),
        dataset_version: "1".to_string(),
        seed: 1,
        deadline_ms: 1,
        expected_accuracy_bps: 10_000,
        should_recover: false,
    };
    let sample = WallClockHarness.run_case(&case, || {
        thread::sleep(Duration::from_millis(5));
        (true, 10_000, true, false)
    });
    assert!(sample.timed_out);
    assert!(!sample.success);
    assert!(!sample.validation_proven);
}

#[test]
fn scorecard_computes_percentiles_and_rates() {
    let suite = cases(8, None);
    let mut executor = DeterministicExecutor {
        fail_index: None,
        low_accuracy_index: None,
        invalid_validation_index: None,
        latency_base: 10,
    };
    let samples = SyntheticHarness.run(&suite, &mut executor);
    let score = build_scorecard(
        "native.cap",
        RunMode::Benchmark,
        &suite,
        &samples,
        &thresholds(false),
    )
    .unwrap();

    assert_eq!(score.runs, 8);
    assert_eq!(score.success_bps, 10_000);
    assert_eq!(score.accuracy_bps, 9_900);
    assert_eq!(score.validation_bps, 10_000);
    assert_eq!(score.p50_ms, 13);
    assert_eq!(score.p95_ms, 17);
    assert_eq!(score.p99_ms, 17);
    assert!(score.passed);
}

#[test]
fn minimum_run_count_is_enforced() {
    let suite = cases(4, None);
    let mut executor = DeterministicExecutor {
        fail_index: None,
        low_accuracy_index: None,
        invalid_validation_index: None,
        latency_base: 10,
    };
    let samples = SyntheticHarness.run(&suite, &mut executor);
    let mut gate = thresholds(false);
    gate.minimum_runs = 8;
    assert!(build_scorecard("native.cap", RunMode::Training, &suite, &samples, &gate).is_err());
}

#[test]
fn success_accuracy_validation_and_latency_thresholds_fail_independently() {
    let suite = cases(8, None);

    let mut fail_success = DeterministicExecutor {
        fail_index: Some(0),
        low_accuracy_index: None,
        invalid_validation_index: None,
        latency_base: 10,
    };
    let samples = SyntheticHarness.run(&suite, &mut fail_success);
    let mut gate = thresholds(false);
    gate.minimum_success_bps = 10_000;
    let score = build_scorecard("native.cap", RunMode::Benchmark, &suite, &samples, &gate).unwrap();
    assert!(score.failures.contains(&"success_rate".to_string()));

    let mut low_accuracy = DeterministicExecutor {
        fail_index: None,
        low_accuracy_index: Some(0),
        invalid_validation_index: None,
        latency_base: 10,
    };
    let samples = SyntheticHarness.run(&suite, &mut low_accuracy);
    let score = build_scorecard(
        "native.cap",
        RunMode::Benchmark,
        &suite,
        &samples,
        &thresholds(false),
    )
    .unwrap();
    assert!(score.failures.contains(&"accuracy".to_string()));

    let mut invalid_validation = DeterministicExecutor {
        fail_index: None,
        low_accuracy_index: None,
        invalid_validation_index: Some(0),
        latency_base: 10,
    };
    let samples = SyntheticHarness.run(&suite, &mut invalid_validation);
    let score = build_scorecard(
        "native.cap",
        RunMode::Benchmark,
        &suite,
        &samples,
        &thresholds(false),
    )
    .unwrap();
    assert!(score.failures.contains(&"validation".to_string()));

    let mut slow = DeterministicExecutor {
        fail_index: None,
        low_accuracy_index: None,
        invalid_validation_index: None,
        latency_base: 200,
    };
    let samples = SyntheticHarness.run(&suite, &mut slow);
    let score = build_scorecard(
        "native.cap",
        RunMode::Benchmark,
        &suite,
        &samples,
        &thresholds(false),
    )
    .unwrap();
    assert!(score.failures.contains(&"p95_latency".to_string()));
}

#[test]
fn recovery_drill_is_a_hard_gate_when_required() {
    let suite = cases(8, Some(0));
    let mut executor = DeterministicExecutor {
        fail_index: None,
        low_accuracy_index: None,
        invalid_validation_index: None,
        latency_base: 10,
    };
    let mut samples = SyntheticHarness.run(&suite, &mut executor);
    samples[0].recovered = false;
    let score = build_scorecard(
        "native.cap",
        RunMode::Benchmark,
        &suite,
        &samples,
        &thresholds(true),
    )
    .unwrap();
    assert!(!score.passed);
    assert!(score.failures.contains(&"recovery_drill".to_string()));
}

#[test]
fn dataset_version_and_seed_are_bound() {
    let mut suite = cases(8, None);
    suite[7].seed = 43;
    let mut executor = DeterministicExecutor {
        fail_index: None,
        low_accuracy_index: None,
        invalid_validation_index: None,
        latency_base: 10,
    };
    let samples = SyntheticHarness.run(&suite, &mut executor);
    assert!(
        build_scorecard(
            "native.cap",
            RunMode::Benchmark,
            &suite,
            &samples,
            &thresholds(false)
        )
        .is_err()
    );
}

#[test]
fn scorecard_canonical_serialization_is_stable() {
    let left = passing_scorecard(RunMode::Training, false);
    let right = passing_scorecard(RunMode::Training, false);
    assert_eq!(left.fingerprint, right.fingerprint);
    assert_eq!(canonical_scorecard(&left), canonical_scorecard(&right));
}

#[test]
fn registration_lifecycle_requires_order() {
    let training = passing_scorecard(RunMode::Training, false);
    let benchmark = passing_scorecard(RunMode::Benchmark, false);
    let post = passing_scorecard(RunMode::Benchmark, true);
    let mut session = QualificationSession::new("native.cap");

    assert!(session.enter_post_registration().is_err());
    assert!(session.register(&benchmark).is_err());

    session.record_training(&training).unwrap();
    session.register(&benchmark).unwrap();
    assert_eq!(session.registration_state, RegistrationState::Registered);

    session.enter_post_registration().unwrap();
    assert_eq!(
        session.registration_state,
        RegistrationState::PostRegistration
    );

    assert!(session.record_post_registration(&post, false).is_err());
    session.record_post_registration(&post, true).unwrap();
    assert_eq!(
        session.registration_state,
        RegistrationState::ProductionReady
    );
}

#[test]
fn failed_scorecard_cannot_enter_registration() {
    let mut score = passing_scorecard(RunMode::Training, false);
    score.passed = false;
    let mut session = QualificationSession::new("native.cap");
    assert!(session.record_training(&score).is_err());
}

#[test]
fn scorecard_capability_is_bound_to_session() {
    let mut score = passing_scorecard(RunMode::Training, false);
    score.capability_id = "native.other".to_string();
    let mut session = QualificationSession::new("native.cap");
    assert!(session.record_training(&score).is_err());
}

#[test]
fn training_benchmark_and_production_promotions_are_sequential() {
    let training = passing_scorecard(RunMode::Training, false);
    let benchmark = passing_scorecard(RunMode::Benchmark, false);
    let post = passing_scorecard(RunMode::Benchmark, true);
    let authority = QualificationAuthority;
    let mut session = QualificationSession::new("native.cap");
    let mut registry = CapabilityRegistry::new();
    registry
        .register(descriptor("native.cap", PromotionState::Experimental))
        .unwrap();

    session.record_training(&training).unwrap();
    let training_receipt = authority.training_receipt(
        &session,
        &training,
        registry.get("native.cap").unwrap().promotion,
    );
    assert!(training_receipt.accepted);
    authority
        .apply_receipt(&mut registry, &training_receipt)
        .unwrap();
    assert_eq!(
        registry.get("native.cap").unwrap().promotion,
        PromotionState::Tested
    );

    session.register(&benchmark).unwrap();
    let benchmark_receipt = authority.benchmark_receipt(
        &session,
        &benchmark,
        registry.get("native.cap").unwrap().promotion,
    );
    assert!(benchmark_receipt.accepted);
    authority
        .apply_receipt(&mut registry, &benchmark_receipt)
        .unwrap();
    assert_eq!(
        registry.get("native.cap").unwrap().promotion,
        PromotionState::Qualified
    );

    session.enter_post_registration().unwrap();
    session.record_post_registration(&post, true).unwrap();
    let production_receipt = authority.production_receipt(
        &session,
        &post,
        registry.get("native.cap").unwrap().promotion,
    );
    assert!(production_receipt.accepted);
    authority
        .apply_receipt(&mut registry, &production_receipt)
        .unwrap();
    assert_eq!(
        registry.get("native.cap").unwrap().promotion,
        PromotionState::Production
    );
}

#[test]
fn registry_rejects_direct_experimental_to_production() {
    let mut registry = CapabilityRegistry::new();
    registry
        .register(descriptor("native.cap", PromotionState::Experimental))
        .unwrap();
    assert!(
        registry
            .set_promotion("native.cap", PromotionState::Production)
            .is_err()
    );
}

#[test]
fn demotion_remains_available_for_safety() {
    let mut registry = CapabilityRegistry::new();
    registry
        .register(descriptor("native.cap", PromotionState::Production))
        .unwrap();
    registry
        .set_promotion("native.cap", PromotionState::Tested)
        .unwrap();
    assert_eq!(
        registry.get("native.cap").unwrap().promotion,
        PromotionState::Tested
    );
}

#[test]
fn stale_receipt_is_rejected() {
    let training = passing_scorecard(RunMode::Training, false);
    let mut session = QualificationSession::new("native.cap");
    session.record_training(&training).unwrap();
    let authority = QualificationAuthority;
    let receipt = authority.training_receipt(&session, &training, PromotionState::Experimental);
    let mut registry = CapabilityRegistry::new();
    registry
        .register(descriptor("native.cap", PromotionState::Tested))
        .unwrap();
    assert!(authority.apply_receipt(&mut registry, &receipt).is_ok());

    registry
        .set_promotion("native.cap", PromotionState::Qualified)
        .unwrap();
    assert!(authority.apply_receipt(&mut registry, &receipt).is_err());
}

#[test]
fn rejected_receipt_cannot_promote() {
    let score = passing_scorecard(RunMode::Training, false);
    let session = QualificationSession::new("native.cap");
    let authority = QualificationAuthority;
    let receipt = authority.training_receipt(&session, &score, PromotionState::Experimental);
    assert!(!receipt.accepted);
    let mut registry = CapabilityRegistry::new();
    registry
        .register(descriptor("native.cap", PromotionState::Experimental))
        .unwrap();
    assert!(authority.apply_receipt(&mut registry, &receipt).is_err());
}

#[test]
fn production_receipt_requires_production_ready_and_recovery() {
    let post = passing_scorecard(RunMode::Benchmark, true);
    let session = QualificationSession::new("native.cap");
    let receipt =
        QualificationAuthority.production_receipt(&session, &post, PromotionState::Qualified);
    assert!(!receipt.accepted);
}

#[test]
fn receipt_serialization_and_id_are_deterministic() {
    let training = passing_scorecard(RunMode::Training, false);
    let mut session = QualificationSession::new("native.cap");
    session.record_training(&training).unwrap();
    let authority = QualificationAuthority;
    let a = authority.training_receipt(&session, &training, PromotionState::Experimental);
    let b = authority.training_receipt(&session, &training, PromotionState::Experimental);
    assert_eq!(a, b);
    assert_eq!(canonical_receipt(&a), canonical_receipt(&b));
    assert_eq!(a.receipt_id.len(), 64);
}

#[test]
fn benchmark_sample_is_bound_to_f07_validation_report() {
    let case = BenchmarkCase {
        case_id: "validated".to_string(),
        dataset_id: "d".to_string(),
        dataset_version: "1".to_string(),
        seed: 7,
        deadline_ms: 100,
        expected_accuracy_bps: 9_900,
        should_recover: false,
    };
    let report = ValidationReport {
        mission_id: "m1".to_string(),
        result_id: "r1".to_string(),
        validator_id: "validator-b".to_string(),
        executor_id: "executor-a".to_string(),
        payload_hash: "hash".to_string(),
        evidence_digest: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        evidence: EvidenceCompleteness {
            complete: true,
            required: vec!["result".to_string()],
            missing: Vec::new(),
            invalid: Vec::new(),
        },
        outcome: ValidationOutcome::Proven,
        fingerprint: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
    };
    let sample = sample_from_validation(&case, &report, 10, 9_900, false);
    assert!(sample.success);
    assert!(sample.validation_proven);

    let mut rejected = report;
    rejected.outcome =
        ValidationOutcome::Rejected(tma_foundation::model::ErrorClass::ValidationFailed);
    let sample = sample_from_validation(&case, &rejected, 10, 9_900, false);
    assert!(!sample.success);
    assert!(!sample.validation_proven);
}
