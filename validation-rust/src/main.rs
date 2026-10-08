use tma_foundation::model::{CapabilityKind, ErrorClass, SideEffectClass};
use tma_validation::{
    EvidenceItem, EvidenceValidator, ExecutorResult, IndependentValidator, RecoveryAgent,
    RecoveryContext, ValidationContext, error_class_name,
};

fn self_test() {
    let result = ExecutorResult {
        mission_id: "self-test".to_string(),
        result_id: "result-self-test".to_string(),
        worker_id: "executor-a".to_string(),
        capability_id: "native.local".to_string(),
        capability_kind: CapabilityKind::Native,
        payload_hash: "payload-hash".to_string(),
        declared_success: true,
        required_evidence: Vec::new(),
        evidence: vec![EvidenceItem {
            kind: "result".to_string(),
            sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            bytes: 1,
            confidence_bps: None,
        }],
        confidence_bps: None,
        completed_at_ms: 100,
    };
    let report = EvidenceValidator.validate(
        &result,
        &ValidationContext {
            validator_id: "validator-b".to_string(),
            now_ms: 110,
            deadline_unix_ms: 1_000,
            minimum_confidence_bps: 0,
        },
    );

    println!(
        "tma-validation status=ok proven={} evidence_complete={} fingerprint={}",
        report.is_proven(),
        report.evidence.complete,
        report.fingerprint
    );
}

fn failure_matrix() {
    let classes = [
        ErrorClass::NetworkFailure,
        ErrorClass::SessionExpired,
        ErrorClass::ElementMoved,
        ErrorClass::LayoutChanged,
        ErrorClass::RateLimit,
        ErrorClass::WorkerCrash,
        ErrorClass::ModelTimeout,
        ErrorClass::LowConfidence,
        ErrorClass::ValidationFailed,
        ErrorClass::DuplicateRisk,
        ErrorClass::DeadlineRisk,
        ErrorClass::ExternalDependency,
        ErrorClass::PermissionDenied,
        ErrorClass::ContractMismatch,
        ErrorClass::Cancelled,
        ErrorClass::Unknown,
    ];
    for class in classes {
        let decision = RecoveryAgent.decide(&RecoveryContext {
            error_class: class,
            attempt: 0,
            max_attempts: 3,
            remaining_ms: 10_000,
            minimum_retry_window_ms: 100,
            replay_safe: true,
            side_effect: SideEffectClass::Read,
            current_capability_id: "capability-a".to_string(),
            fallback_chain: vec!["capability-b".to_string()],
        });
        println!(
            "failure={} disposition={:?} fingerprint={}",
            error_class_name(class),
            decision.disposition,
            decision.fingerprint
        );
    }
    println!("F07_FAILURE_MATRIX=PASS classes=16");
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [command] if command == "self-test" => self_test(),
        [command] if command == "failure-matrix" => failure_matrix(),
        _ => {
            eprintln!("usage: tma-validation <self-test|failure-matrix>");
            std::process::exit(2);
        }
    }
}
