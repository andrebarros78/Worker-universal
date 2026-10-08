use std::collections::BTreeSet;

use sha2::{Digest, Sha256};
use tma_foundation::model::{CapabilityKind, ErrorClass};

use crate::model::{
    EvidenceCompleteness, EvidenceItem, ExecutorResult, ValidationContext, ValidationOutcome,
    ValidationReport,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofStrategy {
    pub required_evidence: Vec<String>,
    pub minimum_confidence_bps: u16,
}

pub trait IndependentValidator {
    fn validate(&self, result: &ExecutorResult, context: &ValidationContext) -> ValidationReport;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EvidenceValidator;

impl EvidenceValidator {
    pub fn strategy(kind: CapabilityKind) -> ProofStrategy {
        let required = match kind {
            CapabilityKind::Vision | CapabilityKind::Document => {
                vec!["ocr_text".to_string(), "field_confidence".to_string()]
            }
            CapabilityKind::Browser => vec!["dom".to_string(), "screenshot".to_string()],
            CapabilityKind::Research => vec!["sources".to_string()],
            CapabilityKind::Computer => vec!["screenshot".to_string()],
            CapabilityKind::Native
            | CapabilityKind::Api
            | CapabilityKind::Mcp
            | CapabilityKind::Llm
            | CapabilityKind::Storage
            | CapabilityKind::Notification => vec!["result".to_string()],
        };
        let minimum_confidence_bps = match kind {
            CapabilityKind::Vision | CapabilityKind::Document | CapabilityKind::Llm => 9_000,
            _ => 0,
        };
        ProofStrategy {
            required_evidence: required,
            minimum_confidence_bps,
        }
    }

    pub fn completeness(result: &ExecutorResult) -> EvidenceCompleteness {
        let strategy = Self::strategy(result.capability_kind);
        let mut required = strategy.required_evidence;
        required.extend(result.required_evidence.iter().cloned());
        required.sort();
        required.dedup();

        let mut valid_kinds = BTreeSet::new();
        let mut invalid = Vec::new();

        for item in &result.evidence {
            if evidence_item_valid(item) {
                valid_kinds.insert(item.kind.clone());
            } else {
                invalid.push(item.kind.clone());
            }
        }
        invalid.sort();
        invalid.dedup();

        let missing = required
            .iter()
            .filter(|kind| !valid_kinds.contains(*kind))
            .cloned()
            .collect::<Vec<_>>();

        EvidenceCompleteness {
            complete: missing.is_empty() && invalid.is_empty(),
            required,
            missing,
            invalid,
        }
    }
}

impl IndependentValidator for EvidenceValidator {
    fn validate(&self, result: &ExecutorResult, context: &ValidationContext) -> ValidationReport {
        let evidence = Self::completeness(result);
        let digest = evidence_digest(&result.evidence);
        let outcome =
            if context.validator_id.trim().is_empty() || context.validator_id == result.worker_id {
                ValidationOutcome::Rejected(ErrorClass::ValidationFailed)
            } else if context.now_ms >= context.deadline_unix_ms {
                ValidationOutcome::Rejected(ErrorClass::DeadlineRisk)
            } else if !result.declared_success || !evidence.complete {
                ValidationOutcome::Rejected(ErrorClass::ValidationFailed)
            } else {
                let strategy = Self::strategy(result.capability_kind);
                let minimum = context
                    .minimum_confidence_bps
                    .max(strategy.minimum_confidence_bps);
                match result.confidence_bps {
                    Some(confidence) if confidence < minimum => {
                        ValidationOutcome::Rejected(ErrorClass::LowConfidence)
                    }
                    None if minimum > 0 => ValidationOutcome::Rejected(ErrorClass::LowConfidence),
                    _ => ValidationOutcome::Proven,
                }
            };

        let fingerprint = validation_fingerprint(result, context, &evidence, &digest, &outcome);

        ValidationReport {
            mission_id: result.mission_id.clone(),
            result_id: result.result_id.clone(),
            validator_id: context.validator_id.clone(),
            executor_id: result.worker_id.clone(),
            payload_hash: result.payload_hash.clone(),
            evidence_digest: digest,
            evidence,
            outcome,
            fingerprint,
        }
    }
}

fn evidence_item_valid(item: &EvidenceItem) -> bool {
    !item.kind.trim().is_empty()
        && item.bytes > 0
        && item.sha256.len() == 64
        && item
            .sha256
            .chars()
            .all(|character| character.is_ascii_hexdigit())
        && item.confidence_bps.is_none_or(|value| value <= 10_000)
}

pub fn evidence_digest(evidence: &[EvidenceItem]) -> String {
    let mut canonical = evidence.to_vec();
    canonical.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then(left.sha256.cmp(&right.sha256))
            .then(left.bytes.cmp(&right.bytes))
            .then(left.confidence_bps.cmp(&right.confidence_bps))
    });
    let encoded = canonical
        .iter()
        .map(|item| {
            format!(
                "{}|{}|{}|{:?}",
                item.kind, item.sha256, item.bytes, item.confidence_bps
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    sha256_hex(encoded.as_bytes())
}

fn validation_fingerprint(
    result: &ExecutorResult,
    context: &ValidationContext,
    evidence: &EvidenceCompleteness,
    digest: &str,
    outcome: &ValidationOutcome,
) -> String {
    let canonical = format!(
        "mission={}|result={}|validator={}|executor={}|payload={}|digest={}|required={}|missing={}|invalid={}|outcome={}|now={}|deadline={}",
        result.mission_id,
        result.result_id,
        context.validator_id,
        result.worker_id,
        result.payload_hash,
        digest,
        evidence.required.join(","),
        evidence.missing.join(","),
        evidence.invalid.join(","),
        outcome_name(outcome),
        context.now_ms,
        context.deadline_unix_ms,
    );
    sha256_hex(canonical.as_bytes())
}

fn outcome_name(outcome: &ValidationOutcome) -> &'static str {
    match outcome {
        ValidationOutcome::Proven => "proven",
        ValidationOutcome::Rejected(ErrorClass::NetworkFailure) => "network_failure",
        ValidationOutcome::Rejected(ErrorClass::SessionExpired) => "session_expired",
        ValidationOutcome::Rejected(ErrorClass::ElementMoved) => "element_moved",
        ValidationOutcome::Rejected(ErrorClass::LayoutChanged) => "layout_changed",
        ValidationOutcome::Rejected(ErrorClass::RateLimit) => "rate_limit",
        ValidationOutcome::Rejected(ErrorClass::WorkerCrash) => "worker_crash",
        ValidationOutcome::Rejected(ErrorClass::ModelTimeout) => "model_timeout",
        ValidationOutcome::Rejected(ErrorClass::LowConfidence) => "low_confidence",
        ValidationOutcome::Rejected(ErrorClass::ValidationFailed) => "validation_failed",
        ValidationOutcome::Rejected(ErrorClass::DuplicateRisk) => "duplicate_risk",
        ValidationOutcome::Rejected(ErrorClass::DeadlineRisk) => "deadline_risk",
        ValidationOutcome::Rejected(ErrorClass::ExternalDependency) => "external_dependency",
        ValidationOutcome::Rejected(ErrorClass::PermissionDenied) => "permission_denied",
        ValidationOutcome::Rejected(ErrorClass::ContractMismatch) => "contract_mismatch",
        ValidationOutcome::Rejected(ErrorClass::Cancelled) => "cancelled",
        ValidationOutcome::Rejected(ErrorClass::Unknown) => "unknown",
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
