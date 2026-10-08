use sha2::{Digest, Sha256};
use tma_foundation::model::{ErrorClass, SideEffectClass};

use crate::model::{RecoveryAction, RecoveryContext, RecoveryDecision, RecoveryDisposition};

#[derive(Debug, Default, Clone, Copy)]
pub struct RecoveryAgent;

impl RecoveryAgent {
    pub fn decide(&self, context: &RecoveryContext) -> RecoveryDecision {
        let disposition = decide_disposition(context);
        let fingerprint = recovery_fingerprint(context, &disposition);
        RecoveryDecision {
            error_class: context.error_class,
            disposition,
            attempt: context.attempt,
            fingerprint,
        }
    }
}

fn decide_disposition(context: &RecoveryContext) -> RecoveryDisposition {
    if context.error_class == ErrorClass::DeadlineRisk
        || context.remaining_ms < context.minimum_retry_window_ms
    {
        return RecoveryDisposition::TerminalDeadline {
            action: RecoveryAction::DeadlineExceeded,
        };
    }

    if context.error_class == ErrorClass::Cancelled {
        return RecoveryDisposition::Cancelled {
            action: RecoveryAction::CancelTerminal,
        };
    }

    if context.attempt >= context.max_attempts {
        return RecoveryDisposition::TerminalFailed {
            action: RecoveryAction::ManualReview,
        };
    }

    if !context.replay_safe
        || matches!(
            context.side_effect,
            SideEffectClass::Submit | SideEffectClass::ExternalEffect
        )
    {
        return RecoveryDisposition::NeedsReview {
            action: RecoveryAction::IndependentReview,
        };
    }

    let fallback = context.fallback_chain.first().cloned();

    match context.error_class {
        ErrorClass::NetworkFailure => retry(RecoveryAction::RetrySame, None, 250),
        ErrorClass::SessionExpired => retry(RecoveryAction::RefreshSession, None, 0),
        ErrorClass::ElementMoved => retry(RecoveryAction::RelocateElement, None, 0),
        ErrorClass::LayoutChanged => retry(RecoveryAction::ReobserveAndReplan, None, 0),
        ErrorClass::RateLimit => retry(
            RecoveryAction::Backoff,
            None,
            bounded_backoff(context.attempt),
        ),
        ErrorClass::WorkerCrash => retry(RecoveryAction::RestartWorker, None, 0),
        ErrorClass::ModelTimeout => retry(RecoveryAction::TryFallback, fallback, 0),
        ErrorClass::LowConfidence => retry(RecoveryAction::SelectiveReread, fallback, 0),
        ErrorClass::ValidationFailed => retry(RecoveryAction::ReexecuteThenValidate, fallback, 0),
        ErrorClass::DuplicateRisk => RecoveryDisposition::NeedsReview {
            action: RecoveryAction::IndependentReview,
        },
        ErrorClass::DeadlineRisk => RecoveryDisposition::TerminalDeadline {
            action: RecoveryAction::DeadlineExceeded,
        },
        ErrorClass::ExternalDependency => retry(
            RecoveryAction::WaitExternalDependency,
            None,
            bounded_backoff(context.attempt),
        ),
        ErrorClass::PermissionDenied => RecoveryDisposition::NeedsReview {
            action: RecoveryAction::PermissionReview,
        },
        ErrorClass::ContractMismatch => RecoveryDisposition::TerminalFailed {
            action: RecoveryAction::ReplanContract,
        },
        ErrorClass::Cancelled => RecoveryDisposition::Cancelled {
            action: RecoveryAction::CancelTerminal,
        },
        ErrorClass::Unknown => RecoveryDisposition::NeedsReview {
            action: RecoveryAction::ManualReview,
        },
    }
}

fn retry(
    action: RecoveryAction,
    next_capability_id: Option<String>,
    delay_ms: u64,
) -> RecoveryDisposition {
    RecoveryDisposition::Retry {
        action,
        next_capability_id,
        delay_ms,
    }
}

fn bounded_backoff(attempt: u16) -> u64 {
    let exponent = u32::from(attempt.min(6));
    250_u64.saturating_mul(1_u64 << exponent).min(15_000)
}

pub fn error_class_name(class: ErrorClass) -> &'static str {
    match class {
        ErrorClass::NetworkFailure => "network_failure",
        ErrorClass::SessionExpired => "session_expired",
        ErrorClass::ElementMoved => "element_moved",
        ErrorClass::LayoutChanged => "layout_changed",
        ErrorClass::RateLimit => "rate_limit",
        ErrorClass::WorkerCrash => "worker_crash",
        ErrorClass::ModelTimeout => "model_timeout",
        ErrorClass::LowConfidence => "low_confidence",
        ErrorClass::ValidationFailed => "validation_failed",
        ErrorClass::DuplicateRisk => "duplicate_risk",
        ErrorClass::DeadlineRisk => "deadline_risk",
        ErrorClass::ExternalDependency => "external_dependency",
        ErrorClass::PermissionDenied => "permission_denied",
        ErrorClass::ContractMismatch => "contract_mismatch",
        ErrorClass::Cancelled => "cancelled",
        ErrorClass::Unknown => "unknown",
    }
}

fn action_name(action: RecoveryAction) -> &'static str {
    match action {
        RecoveryAction::RetrySame => "retry_same",
        RecoveryAction::RefreshSession => "refresh_session",
        RecoveryAction::RelocateElement => "relocate_element",
        RecoveryAction::ReobserveAndReplan => "reobserve_replan",
        RecoveryAction::Backoff => "backoff",
        RecoveryAction::RestartWorker => "restart_worker",
        RecoveryAction::TryFallback => "try_fallback",
        RecoveryAction::SelectiveReread => "selective_reread",
        RecoveryAction::ReexecuteThenValidate => "reexecute_validate",
        RecoveryAction::IndependentReview => "independent_review",
        RecoveryAction::DeadlineExceeded => "deadline_exceeded",
        RecoveryAction::WaitExternalDependency => "wait_external",
        RecoveryAction::PermissionReview => "permission_review",
        RecoveryAction::ReplanContract => "replan_contract",
        RecoveryAction::CancelTerminal => "cancel_terminal",
        RecoveryAction::ManualReview => "manual_review",
    }
}

fn disposition_name(disposition: &RecoveryDisposition) -> String {
    match disposition {
        RecoveryDisposition::Retry {
            action,
            next_capability_id,
            delay_ms,
        } => format!(
            "retry:{}:{}:{}",
            action_name(*action),
            next_capability_id.as_deref().unwrap_or(""),
            delay_ms
        ),
        RecoveryDisposition::NeedsReview { action } => {
            format!("review:{}", action_name(*action))
        }
        RecoveryDisposition::TerminalFailed { action } => {
            format!("failed:{}", action_name(*action))
        }
        RecoveryDisposition::TerminalDeadline { action } => {
            format!("deadline:{}", action_name(*action))
        }
        RecoveryDisposition::Cancelled { action } => {
            format!("cancelled:{}", action_name(*action))
        }
    }
}

fn recovery_fingerprint(context: &RecoveryContext, disposition: &RecoveryDisposition) -> String {
    let canonical = format!(
        "class={}|attempt={}|max={}|remaining={}|window={}|replay={}|side_effect={:?}|capability={}|fallbacks={}|decision={}",
        error_class_name(context.error_class),
        context.attempt,
        context.max_attempts,
        context.remaining_ms,
        context.minimum_retry_window_ms,
        context.replay_safe,
        context.side_effect,
        context.current_capability_id,
        context.fallback_chain.join(","),
        disposition_name(disposition)
    );
    let digest = Sha256::digest(canonical.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
