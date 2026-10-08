pub mod durable_bridge;
pub mod model;
pub mod recovery;
pub mod validator;

pub use durable_bridge::{
    finalize_validated_failure, finalize_validated_success, persist_recovery_checkpoint,
    recover_recovery_checkpoint,
};
pub use model::{
    EvidenceCompleteness, EvidenceItem, ExecutorResult, RecoveryAction, RecoveryContext,
    RecoveryDecision, RecoveryDisposition, ValidationContext, ValidationOutcome, ValidationReport,
};
pub use recovery::{RecoveryAgent, error_class_name};
pub use validator::{EvidenceValidator, IndependentValidator, ProofStrategy, evidence_digest};
