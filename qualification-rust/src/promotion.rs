use sha2::{Digest, Sha256};
use tma_foundation::model::PromotionState;
use tma_foundation::registry::CapabilityRegistry;

use crate::model::{
    QualificationReceipt, QualificationSession, RegistrationState, RunMode, Scorecard,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct QualificationAuthority;

impl QualificationAuthority {
    pub fn training_receipt(
        &self,
        session: &QualificationSession,
        scorecard: &Scorecard,
        current: PromotionState,
    ) -> QualificationReceipt {
        let accepted = current == PromotionState::Experimental
            && scorecard.mode == RunMode::Training
            && scorecard.passed
            && session.training_scorecard.as_deref() == Some(scorecard.fingerprint.as_str());
        receipt(
            session,
            scorecard,
            current,
            PromotionState::Tested,
            accepted,
            if accepted {
                "training_passed"
            } else {
                "training_gate_failed"
            },
        )
    }

    pub fn benchmark_receipt(
        &self,
        session: &QualificationSession,
        scorecard: &Scorecard,
        current: PromotionState,
    ) -> QualificationReceipt {
        let accepted = current == PromotionState::Tested
            && scorecard.mode == RunMode::Benchmark
            && scorecard.passed
            && session.registration_state == RegistrationState::Registered
            && session.benchmark_scorecard.as_deref() == Some(scorecard.fingerprint.as_str());
        receipt(
            session,
            scorecard,
            current,
            PromotionState::Qualified,
            accepted,
            if accepted {
                "benchmark_qualified"
            } else {
                "benchmark_gate_failed"
            },
        )
    }

    pub fn production_receipt(
        &self,
        session: &QualificationSession,
        scorecard: &Scorecard,
        current: PromotionState,
    ) -> QualificationReceipt {
        let accepted = current == PromotionState::Qualified
            && scorecard.mode == RunMode::Benchmark
            && scorecard.passed
            && session.registration_state == RegistrationState::ProductionReady
            && session.recovery_drill_passed
            && session.post_registration_scorecard.as_deref()
                == Some(scorecard.fingerprint.as_str());
        receipt(
            session,
            scorecard,
            current,
            PromotionState::Production,
            accepted,
            if accepted {
                "production_ready"
            } else {
                "production_gate_failed"
            },
        )
    }

    pub fn apply_receipt(
        &self,
        registry: &mut CapabilityRegistry,
        receipt: &QualificationReceipt,
    ) -> Result<(), String> {
        if !receipt.accepted {
            return Err("qualification receipt rejected".to_string());
        }
        let current = registry
            .get(&receipt.capability_id)
            .ok_or_else(|| format!("unknown capability: {}", receipt.capability_id))?
            .promotion;
        if current == receipt.to {
            return Ok(());
        }
        if current != receipt.from {
            return Err(format!(
                "stale qualification receipt: expected {:?}, current {:?}",
                receipt.from, current
            ));
        }
        registry.set_promotion(&receipt.capability_id, receipt.to)
    }
}

pub fn canonical_receipt(receipt: &QualificationReceipt) -> String {
    format!(
        "receipt={}|capability={}|from={:?}|to={:?}|registration={:?}|scorecard={}|accepted={}|reason={}",
        receipt.receipt_id,
        receipt.capability_id,
        receipt.from,
        receipt.to,
        receipt.registration_state,
        receipt.scorecard_fingerprint,
        receipt.accepted,
        receipt.reason
    )
}

fn receipt(
    session: &QualificationSession,
    scorecard: &Scorecard,
    from: PromotionState,
    to: PromotionState,
    accepted: bool,
    reason: &str,
) -> QualificationReceipt {
    let material = format!(
        "capability={}|from={from:?}|to={to:?}|registration={:?}|scorecard={}|accepted={accepted}|reason={reason}",
        session.capability_id, session.registration_state, scorecard.fingerprint
    );
    let digest = Sha256::digest(material.as_bytes());
    let receipt_id = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    QualificationReceipt {
        receipt_id,
        capability_id: session.capability_id.clone(),
        from,
        to,
        registration_state: session.registration_state,
        scorecard_fingerprint: scorecard.fingerprint.clone(),
        accepted,
        reason: reason.to_string(),
    }
}
