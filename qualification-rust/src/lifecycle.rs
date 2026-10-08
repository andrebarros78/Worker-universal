use crate::model::{QualificationSession, RegistrationState, Scorecard};

impl QualificationSession {
    pub fn new(capability_id: impl Into<String>) -> Self {
        Self {
            capability_id: capability_id.into(),
            registration_state: RegistrationState::PreRegistration,
            training_scorecard: None,
            benchmark_scorecard: None,
            post_registration_scorecard: None,
            recovery_drill_passed: false,
        }
    }

    pub fn record_training(&mut self, scorecard: &Scorecard) -> Result<(), String> {
        self.ensure_capability(scorecard)?;
        if self.registration_state != RegistrationState::PreRegistration {
            return Err("training only allowed before registration".to_string());
        }
        if !scorecard.passed {
            return Err("training scorecard failed".to_string());
        }
        self.training_scorecard = Some(scorecard.fingerprint.clone());
        Ok(())
    }

    pub fn register(&mut self, scorecard: &Scorecard) -> Result<(), String> {
        self.ensure_capability(scorecard)?;
        if self.registration_state != RegistrationState::PreRegistration {
            return Err("registration state mismatch".to_string());
        }
        if self.training_scorecard.is_none() || !scorecard.passed {
            return Err("training and passing benchmark required".to_string());
        }
        self.benchmark_scorecard = Some(scorecard.fingerprint.clone());
        self.registration_state = RegistrationState::Registered;
        Ok(())
    }

    pub fn enter_post_registration(&mut self) -> Result<(), String> {
        if self.registration_state != RegistrationState::Registered {
            return Err("must be registered first".to_string());
        }
        self.registration_state = RegistrationState::PostRegistration;
        Ok(())
    }

    pub fn record_post_registration(
        &mut self,
        scorecard: &Scorecard,
        recovery_drill_passed: bool,
    ) -> Result<(), String> {
        self.ensure_capability(scorecard)?;
        if self.registration_state != RegistrationState::PostRegistration {
            return Err("post-registration state required".to_string());
        }
        if !scorecard.passed {
            return Err("post-registration scorecard failed".to_string());
        }
        if !recovery_drill_passed {
            return Err("recovery drill required".to_string());
        }
        self.post_registration_scorecard = Some(scorecard.fingerprint.clone());
        self.recovery_drill_passed = true;
        self.registration_state = RegistrationState::ProductionReady;
        Ok(())
    }

    fn ensure_capability(&self, scorecard: &Scorecard) -> Result<(), String> {
        if scorecard.capability_id != self.capability_id {
            return Err("scorecard capability mismatch".to_string());
        }
        Ok(())
    }
}
