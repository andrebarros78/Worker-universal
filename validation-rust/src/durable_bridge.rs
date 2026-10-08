use tma_core::MissionState;
use tma_core::durable::{
    DurableResult, DurableStore, ResultDisposition, ResultSubmission, ValidationReceipt,
};

use crate::model::{RecoveryDecision, ValidationOutcome, ValidationReport};
use crate::recovery::error_class_name;

pub fn finalize_validated_success(
    store: &DurableStore,
    report: &ValidationReport,
    worker_id: &str,
    fencing_token: i64,
    now_ms: i64,
) -> DurableResult<ResultDisposition> {
    if !matches!(report.outcome, ValidationOutcome::Proven) {
        return Err(tma_core::durable::DurableError::InvalidInput(
            "validation report is not proven".to_string(),
        ));
    }

    let receipt_id = format!("validation-{}", report.fingerprint);
    store.record_validation_receipt(&ValidationReceipt {
        receipt_id: receipt_id.clone(),
        mission_id: report.mission_id.clone(),
        validator_id: report.validator_id.clone(),
        subject_payload_hash: report.payload_hash.clone(),
        evidence_digest: report.evidence_digest.clone(),
        accepted: true,
        created_at_ms: now_ms,
    })?;

    store.submit_result(&ResultSubmission {
        result_id: report.result_id.clone(),
        mission_id: report.mission_id.clone(),
        worker_id: worker_id.to_string(),
        fencing_token,
        target_state: MissionState::Succeeded,
        payload_hash: report.payload_hash.clone(),
        validation_receipt_id: Some(receipt_id),
        created_at_ms: now_ms,
    })
}

pub fn finalize_validated_failure(
    store: &DurableStore,
    report: &ValidationReport,
    worker_id: &str,
    fencing_token: i64,
    target: MissionState,
    now_ms: i64,
) -> DurableResult<ResultDisposition> {
    if !matches!(
        target,
        MissionState::Failed | MissionState::DeadlineExceeded
    ) {
        return Err(tma_core::durable::DurableError::InvalidInput(
            "failure finalization target must be failed or deadline_exceeded".to_string(),
        ));
    }
    if matches!(report.outcome, ValidationOutcome::Proven) {
        return Err(tma_core::durable::DurableError::InvalidInput(
            "proven validation cannot finalize as failure".to_string(),
        ));
    }

    store.submit_result(&ResultSubmission {
        result_id: report.result_id.clone(),
        mission_id: report.mission_id.clone(),
        worker_id: worker_id.to_string(),
        fencing_token,
        target_state: target,
        payload_hash: report.payload_hash.clone(),
        validation_receipt_id: None,
        created_at_ms: now_ms,
    })
}

pub fn persist_recovery_checkpoint(
    store: &DurableStore,
    mission_id: &str,
    worker_id: &str,
    fencing_token: i64,
    decision: &RecoveryDecision,
    now_ms: i64,
) -> DurableResult<String> {
    let checkpoint = format!(
        "recovery_v1;class={};attempt={};fingerprint={}",
        error_class_name(decision.error_class),
        decision.attempt,
        decision.fingerprint
    );
    store.save_checkpoint(mission_id, worker_id, fencing_token, &checkpoint, now_ms)?;
    Ok(checkpoint)
}

pub fn recover_recovery_checkpoint(
    store: &DurableStore,
    mission_id: &str,
) -> Result<String, String> {
    let mission = store
        .mission(mission_id)
        .map_err(|error| error.to_string())?;
    if !mission.checkpoint.starts_with("recovery_v1;") {
        return Err("recovery checkpoint missing".to_string());
    }
    Ok(mission.checkpoint)
}
