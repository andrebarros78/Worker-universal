use tma_foundation::model::{CapabilityKind, ErrorClass, SideEffectClass};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceItem {
    pub kind: String,
    pub sha256: String,
    pub bytes: u64,
    pub confidence_bps: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutorResult {
    pub mission_id: String,
    pub result_id: String,
    pub worker_id: String,
    pub capability_id: String,
    pub capability_kind: CapabilityKind,
    pub payload_hash: String,
    pub declared_success: bool,
    pub required_evidence: Vec<String>,
    pub evidence: Vec<EvidenceItem>,
    pub confidence_bps: Option<u16>,
    pub completed_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationContext {
    pub validator_id: String,
    pub now_ms: i64,
    pub deadline_unix_ms: i64,
    pub minimum_confidence_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceCompleteness {
    pub complete: bool,
    pub required: Vec<String>,
    pub missing: Vec<String>,
    pub invalid: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationOutcome {
    Proven,
    Rejected(ErrorClass),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    pub mission_id: String,
    pub result_id: String,
    pub validator_id: String,
    pub executor_id: String,
    pub payload_hash: String,
    pub evidence_digest: String,
    pub evidence: EvidenceCompleteness,
    pub outcome: ValidationOutcome,
    pub fingerprint: String,
}

impl ValidationReport {
    pub fn is_proven(&self) -> bool {
        matches!(self.outcome, ValidationOutcome::Proven)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    RetrySame,
    RefreshSession,
    RelocateElement,
    ReobserveAndReplan,
    Backoff,
    RestartWorker,
    TryFallback,
    SelectiveReread,
    ReexecuteThenValidate,
    IndependentReview,
    DeadlineExceeded,
    WaitExternalDependency,
    PermissionReview,
    ReplanContract,
    CancelTerminal,
    ManualReview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryDisposition {
    Retry {
        action: RecoveryAction,
        next_capability_id: Option<String>,
        delay_ms: u64,
    },
    NeedsReview {
        action: RecoveryAction,
    },
    TerminalFailed {
        action: RecoveryAction,
    },
    TerminalDeadline {
        action: RecoveryAction,
    },
    Cancelled {
        action: RecoveryAction,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryContext {
    pub error_class: ErrorClass,
    pub attempt: u16,
    pub max_attempts: u16,
    pub remaining_ms: u64,
    pub minimum_retry_window_ms: u64,
    pub replay_safe: bool,
    pub side_effect: SideEffectClass,
    pub current_capability_id: String,
    pub fallback_chain: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryDecision {
    pub error_class: ErrorClass,
    pub disposition: RecoveryDisposition,
    pub attempt: u16,
    pub fingerprint: String,
}
