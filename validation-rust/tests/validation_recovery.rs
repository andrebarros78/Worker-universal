use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tma_core::MissionState;
use tma_core::durable::{
    ClaimOutcome, DurableStore, MissionSpec, ResultDisposition, ResultSubmission,
};
use tma_foundation::model::{CapabilityKind, ErrorClass, SideEffectClass};
use tma_validation::{
    EvidenceItem, EvidenceValidator, ExecutorResult, IndependentValidator, RecoveryAction,
    RecoveryAgent, RecoveryContext, RecoveryDisposition, ValidationContext, ValidationOutcome,
    error_class_name, finalize_validated_failure, finalize_validated_success,
    persist_recovery_checkpoint, recover_recovery_checkpoint,
};

fn sha(ch: char) -> String {
    std::iter::repeat_n(ch, 64).collect()
}

fn evidence(kind: &str) -> EvidenceItem {
    EvidenceItem {
        kind: kind.to_string(),
        sha256: sha('a'),
        bytes: 10,
        confidence_bps: None,
    }
}

fn executor_result(kind: CapabilityKind) -> ExecutorResult {
    ExecutorResult {
        mission_id: "mission-1".to_string(),
        result_id: "result-1".to_string(),
        worker_id: "executor-a".to_string(),
        capability_id: "capability-a".to_string(),
        capability_kind: kind,
        payload_hash: "payload-hash".to_string(),
        declared_success: true,
        required_evidence: Vec::new(),
        evidence: match kind {
            CapabilityKind::Vision | CapabilityKind::Document => {
                vec![evidence("ocr_text"), evidence("field_confidence")]
            }
            CapabilityKind::Browser => vec![evidence("dom"), evidence("screenshot")],
            CapabilityKind::Research => vec![evidence("sources")],
            CapabilityKind::Computer => vec![evidence("screenshot")],
            _ => vec![evidence("result")],
        },
        confidence_bps: match kind {
            CapabilityKind::Vision | CapabilityKind::Document | CapabilityKind::Llm => Some(9_500),
            _ => None,
        },
        completed_at_ms: 100,
    }
}

fn validation_context() -> ValidationContext {
    ValidationContext {
        validator_id: "validator-b".to_string(),
        now_ms: 200,
        deadline_unix_ms: 10_000,
        minimum_confidence_bps: 0,
    }
}

fn recovery_context(class: ErrorClass) -> RecoveryContext {
    RecoveryContext {
        error_class: class,
        attempt: 0,
        max_attempts: 3,
        remaining_ms: 10_000,
        minimum_retry_window_ms: 100,
        replay_safe: true,
        side_effect: SideEffectClass::Read,
        current_capability_id: "capability-a".to_string(),
        fallback_chain: vec!["capability-b".to_string()],
    }
}

fn db_path(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "tma-f07-{name}-{}-{nonce}.sqlite3",
        std::process::id()
    ))
}

fn cleanup(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}

fn spec(id: &str) -> MissionSpec {
    MissionSpec {
        mission_id: id.to_string(),
        task_id: format!("task-{id}"),
        deadline_unix_ms: 50_000,
        priority: 10,
        idempotency_key: format!("idem-{id}"),
        payload_hash: format!("payload-{id}"),
    }
}

#[test]
fn false_success_without_required_evidence_is_rejected() {
    let mut result = executor_result(CapabilityKind::Browser);
    result.evidence.retain(|item| item.kind != "screenshot");
    let report = EvidenceValidator.validate(&result, &validation_context());

    assert!(!report.is_proven());
    assert_eq!(
        report.outcome,
        ValidationOutcome::Rejected(ErrorClass::ValidationFailed)
    );
    assert_eq!(report.evidence.missing, vec!["screenshot"]);
}

#[test]
fn invalid_evidence_hash_and_empty_bytes_are_reported() {
    let mut result = executor_result(CapabilityKind::Native);
    result.evidence[0].sha256 = "not-a-hash".to_string();
    result.evidence[0].bytes = 0;
    let report = EvidenceValidator.validate(&result, &validation_context());

    assert!(!report.evidence.complete);
    assert_eq!(report.evidence.invalid, vec!["result"]);
    assert_eq!(report.evidence.missing, vec!["result"]);
}

#[test]
fn validator_must_be_independent_from_executor() {
    let result = executor_result(CapabilityKind::Native);
    let mut context = validation_context();
    context.validator_id = result.worker_id.clone();
    let report = EvidenceValidator.validate(&result, &context);
    assert_eq!(
        report.outcome,
        ValidationOutcome::Rejected(ErrorClass::ValidationFailed)
    );
}

#[test]
fn declared_failure_is_never_promoted_to_proven() {
    let mut result = executor_result(CapabilityKind::Native);
    result.declared_success = false;
    let report = EvidenceValidator.validate(&result, &validation_context());
    assert_eq!(
        report.outcome,
        ValidationOutcome::Rejected(ErrorClass::ValidationFailed)
    );
}

#[test]
fn document_browser_research_and_native_strategies_prove_valid_evidence() {
    for kind in [
        CapabilityKind::Document,
        CapabilityKind::Vision,
        CapabilityKind::Browser,
        CapabilityKind::Research,
        CapabilityKind::Native,
    ] {
        let report = EvidenceValidator.validate(&executor_result(kind), &validation_context());
        assert!(report.is_proven(), "kind={kind:?} report={report:?}");
        assert!(report.evidence.complete);
    }
}

#[test]
fn low_confidence_is_rejected_independently() {
    let mut result = executor_result(CapabilityKind::Vision);
    result.confidence_bps = Some(8_999);
    let report = EvidenceValidator.validate(&result, &validation_context());
    assert_eq!(
        report.outcome,
        ValidationOutcome::Rejected(ErrorClass::LowConfidence)
    );
}

#[test]
fn expired_deadline_preempts_declared_success() {
    let result = executor_result(CapabilityKind::Native);
    let mut context = validation_context();
    context.now_ms = context.deadline_unix_ms;
    let report = EvidenceValidator.validate(&result, &context);
    assert_eq!(
        report.outcome,
        ValidationOutcome::Rejected(ErrorClass::DeadlineRisk)
    );
}

#[test]
fn extra_required_evidence_is_enforced() {
    let mut result = executor_result(CapabilityKind::Native);
    result.required_evidence.push("audit_log".to_string());
    let report = EvidenceValidator.validate(&result, &validation_context());
    assert_eq!(report.evidence.missing, vec!["audit_log"]);
    assert!(!report.is_proven());
}

#[test]
fn validation_fingerprint_is_stable_under_evidence_reordering() {
    let mut left = executor_result(CapabilityKind::Browser);
    let mut right = left.clone();
    right.evidence.reverse();

    let left_report = EvidenceValidator.validate(&left, &validation_context());
    let right_report = EvidenceValidator.validate(&right, &validation_context());
    assert_eq!(left_report.evidence_digest, right_report.evidence_digest);
    assert_eq!(left_report.fingerprint, right_report.fingerprint);

    left.payload_hash = "changed".to_string();
    let changed = EvidenceValidator.validate(&left, &validation_context());
    assert_ne!(left_report.fingerprint, changed.fingerprint);
}

#[test]
fn all_sixteen_error_classes_have_explicit_recovery_policy() {
    let cases = [
        (ErrorClass::NetworkFailure, "network_failure"),
        (ErrorClass::SessionExpired, "session_expired"),
        (ErrorClass::ElementMoved, "element_moved"),
        (ErrorClass::LayoutChanged, "layout_changed"),
        (ErrorClass::RateLimit, "rate_limit"),
        (ErrorClass::WorkerCrash, "worker_crash"),
        (ErrorClass::ModelTimeout, "model_timeout"),
        (ErrorClass::LowConfidence, "low_confidence"),
        (ErrorClass::ValidationFailed, "validation_failed"),
        (ErrorClass::DuplicateRisk, "duplicate_risk"),
        (ErrorClass::DeadlineRisk, "deadline_risk"),
        (ErrorClass::ExternalDependency, "external_dependency"),
        (ErrorClass::PermissionDenied, "permission_denied"),
        (ErrorClass::ContractMismatch, "contract_mismatch"),
        (ErrorClass::Cancelled, "cancelled"),
        (ErrorClass::Unknown, "unknown"),
    ];

    let mut names = BTreeSet::new();
    for (class, expected_name) in cases {
        assert_eq!(error_class_name(class), expected_name);
        assert!(names.insert(expected_name));
        let decision = RecoveryAgent.decide(&recovery_context(class));
        assert_eq!(decision.error_class, class);
        assert_eq!(decision.fingerprint.len(), 64);
    }
    assert_eq!(names.len(), 16);
}

#[test]
fn retryable_failure_classes_map_to_expected_actions() {
    let cases = [
        (ErrorClass::NetworkFailure, RecoveryAction::RetrySame),
        (ErrorClass::SessionExpired, RecoveryAction::RefreshSession),
        (ErrorClass::ElementMoved, RecoveryAction::RelocateElement),
        (
            ErrorClass::LayoutChanged,
            RecoveryAction::ReobserveAndReplan,
        ),
        (ErrorClass::RateLimit, RecoveryAction::Backoff),
        (ErrorClass::WorkerCrash, RecoveryAction::RestartWorker),
        (ErrorClass::ModelTimeout, RecoveryAction::TryFallback),
        (ErrorClass::LowConfidence, RecoveryAction::SelectiveReread),
        (
            ErrorClass::ValidationFailed,
            RecoveryAction::ReexecuteThenValidate,
        ),
        (
            ErrorClass::ExternalDependency,
            RecoveryAction::WaitExternalDependency,
        ),
    ];

    for (class, expected_action) in cases {
        let decision = RecoveryAgent.decide(&recovery_context(class));
        match decision.disposition {
            RecoveryDisposition::Retry { action, .. } => assert_eq!(action, expected_action),
            other => panic!("expected retry for {class:?}, got {other:?}"),
        }
    }
}

#[test]
fn nonretryable_classes_map_to_review_or_terminal_states() {
    let duplicate = RecoveryAgent.decide(&recovery_context(ErrorClass::DuplicateRisk));
    assert!(matches!(
        duplicate.disposition,
        RecoveryDisposition::NeedsReview {
            action: RecoveryAction::IndependentReview
        }
    ));

    let permission = RecoveryAgent.decide(&recovery_context(ErrorClass::PermissionDenied));
    assert!(matches!(
        permission.disposition,
        RecoveryDisposition::NeedsReview {
            action: RecoveryAction::PermissionReview
        }
    ));

    let contract = RecoveryAgent.decide(&recovery_context(ErrorClass::ContractMismatch));
    assert!(matches!(
        contract.disposition,
        RecoveryDisposition::TerminalFailed {
            action: RecoveryAction::ReplanContract
        }
    ));

    let deadline = RecoveryAgent.decide(&recovery_context(ErrorClass::DeadlineRisk));
    assert!(matches!(
        deadline.disposition,
        RecoveryDisposition::TerminalDeadline {
            action: RecoveryAction::DeadlineExceeded
        }
    ));

    let cancelled = RecoveryAgent.decide(&recovery_context(ErrorClass::Cancelled));
    assert!(matches!(
        cancelled.disposition,
        RecoveryDisposition::Cancelled {
            action: RecoveryAction::CancelTerminal
        }
    ));

    let unknown = RecoveryAgent.decide(&recovery_context(ErrorClass::Unknown));
    assert!(matches!(
        unknown.disposition,
        RecoveryDisposition::NeedsReview {
            action: RecoveryAction::ManualReview
        }
    ));
}

#[test]
fn retry_budget_and_deadline_window_are_hard_bounds() {
    let mut exhausted = recovery_context(ErrorClass::NetworkFailure);
    exhausted.attempt = 3;
    exhausted.max_attempts = 3;
    let decision = RecoveryAgent.decide(&exhausted);
    assert!(matches!(
        decision.disposition,
        RecoveryDisposition::TerminalFailed { .. }
    ));

    let mut too_late = recovery_context(ErrorClass::NetworkFailure);
    too_late.remaining_ms = 99;
    too_late.minimum_retry_window_ms = 100;
    let decision = RecoveryAgent.decide(&too_late);
    assert!(matches!(
        decision.disposition,
        RecoveryDisposition::TerminalDeadline { .. }
    ));
}

#[test]
fn unsafe_or_submit_side_effect_is_never_auto_replayed() {
    let mut unsafe_context = recovery_context(ErrorClass::NetworkFailure);
    unsafe_context.replay_safe = false;
    assert!(matches!(
        RecoveryAgent.decide(&unsafe_context).disposition,
        RecoveryDisposition::NeedsReview { .. }
    ));

    let mut submit_context = recovery_context(ErrorClass::NetworkFailure);
    submit_context.side_effect = SideEffectClass::Submit;
    assert!(matches!(
        RecoveryAgent.decide(&submit_context).disposition,
        RecoveryDisposition::NeedsReview { .. }
    ));

    let mut external_context = recovery_context(ErrorClass::NetworkFailure);
    external_context.side_effect = SideEffectClass::ExternalEffect;
    assert!(matches!(
        RecoveryAgent.decide(&external_context).disposition,
        RecoveryDisposition::NeedsReview { .. }
    ));
}

#[test]
fn model_timeout_consumes_next_fallback_capability() {
    let decision = RecoveryAgent.decide(&recovery_context(ErrorClass::ModelTimeout));
    match decision.disposition {
        RecoveryDisposition::Retry {
            action,
            next_capability_id,
            ..
        } => {
            assert_eq!(action, RecoveryAction::TryFallback);
            assert_eq!(next_capability_id.as_deref(), Some("capability-b"));
        }
        other => panic!("unexpected decision: {other:?}"),
    }
}

#[test]
fn recovery_fingerprint_is_deterministic() {
    let context = recovery_context(ErrorClass::RateLimit);
    let a = RecoveryAgent.decide(&context);
    let b = RecoveryAgent.decide(&context);
    assert_eq!(a, b);

    let mut changed = context;
    changed.attempt = 1;
    assert_ne!(a.fingerprint, RecoveryAgent.decide(&changed).fingerprint);
}

#[test]
fn independent_validation_receipt_is_required_for_durable_success() {
    let path = db_path("success");
    let store = DurableStore::open(&path).unwrap();
    store.create_mission(&spec("m1"), 100).unwrap();
    let grant = match store.claim_next("executor-a", 200, 1_000).unwrap() {
        ClaimOutcome::Claimed(grant) => grant,
        ClaimOutcome::Empty => panic!("expected lease"),
    };
    store
        .transition_with_lease(
            "m1",
            "executor-a",
            grant.fencing_token,
            MissionState::Validating,
            220,
        )
        .unwrap();

    let direct = store
        .submit_result(&ResultSubmission {
            result_id: "direct-success".to_string(),
            mission_id: "m1".to_string(),
            worker_id: "executor-a".to_string(),
            fencing_token: grant.fencing_token,
            target_state: MissionState::Succeeded,
            payload_hash: "payload-hash".to_string(),
            validation_receipt_id: None,
            created_at_ms: 230,
        })
        .unwrap();
    assert_eq!(
        direct,
        ResultDisposition::Rejected {
            reason: "validation_receipt_required".to_string()
        }
    );

    let mut result = executor_result(CapabilityKind::Native);
    result.mission_id = "m1".to_string();
    result.result_id = "validated-success".to_string();
    let report = EvidenceValidator.validate(&result, &validation_context());
    assert!(report.is_proven());

    assert_eq!(
        finalize_validated_success(&store, &report, "executor-a", grant.fencing_token, 240,)
            .unwrap(),
        ResultDisposition::Accepted
    );
    assert_eq!(store.mission("m1").unwrap().state, MissionState::Succeeded);
    cleanup(&path);
}

#[test]
fn rejected_validation_can_terminalize_failure_but_not_success() {
    let path = db_path("failure");
    let store = DurableStore::open(&path).unwrap();
    store.create_mission(&spec("m1"), 100).unwrap();
    let grant = match store.claim_next("executor-a", 200, 1_000).unwrap() {
        ClaimOutcome::Claimed(grant) => grant,
        ClaimOutcome::Empty => panic!("expected lease"),
    };
    store
        .transition_with_lease(
            "m1",
            "executor-a",
            grant.fencing_token,
            MissionState::Validating,
            220,
        )
        .unwrap();

    let mut result = executor_result(CapabilityKind::Browser);
    result.mission_id = "m1".to_string();
    result.result_id = "validated-failure".to_string();
    result.evidence.clear();
    let report = EvidenceValidator.validate(&result, &validation_context());
    assert!(!report.is_proven());
    assert!(
        finalize_validated_success(&store, &report, "executor-a", grant.fencing_token, 230,)
            .is_err()
    );

    assert_eq!(
        finalize_validated_failure(
            &store,
            &report,
            "executor-a",
            grant.fencing_token,
            MissionState::Failed,
            240,
        )
        .unwrap(),
        ResultDisposition::Accepted
    );
    assert_eq!(store.mission("m1").unwrap().state, MissionState::Failed);
    cleanup(&path);
}

#[test]
fn recovery_checkpoint_survives_store_restart_and_reclaim() {
    let path = db_path("recovery-restart");
    let store = DurableStore::open(&path).unwrap();
    store.create_mission(&spec("m1"), 100).unwrap();
    let first = match store.claim_next("executor-a", 200, 100).unwrap() {
        ClaimOutcome::Claimed(grant) => grant,
        ClaimOutcome::Empty => panic!("expected lease"),
    };

    let decision = RecoveryAgent.decide(&recovery_context(ErrorClass::WorkerCrash));
    let checkpoint = persist_recovery_checkpoint(
        &store,
        "m1",
        "executor-a",
        first.fencing_token,
        &decision,
        220,
    )
    .unwrap();
    drop(store);

    let reopened = DurableStore::open(&path).unwrap();
    reopened.reconcile(500).unwrap();
    assert_eq!(
        recover_recovery_checkpoint(&reopened, "m1").unwrap(),
        checkpoint
    );
    let second = match reopened.claim_next("executor-b", 501, 500).unwrap() {
        ClaimOutcome::Claimed(grant) => grant,
        ClaimOutcome::Empty => panic!("expected recovered lease"),
    };
    assert!(second.fencing_token > first.fencing_token);
    assert_eq!(second.checkpoint, checkpoint);
    cleanup(&path);
}
