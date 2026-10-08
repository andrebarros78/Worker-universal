pub mod durable;

use std::fmt;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionState {
    Received,
    Planned,
    Running,
    Validating,
    Succeeded,
    Failed,
    DeadlineExceeded,
}

impl MissionState {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            MissionState::Succeeded | MissionState::Failed | MissionState::DeadlineExceeded
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionError {
    pub from: MissionState,
    pub to: MissionState,
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "illegal transition: {:?} -> {:?}", self.from, self.to)
    }
}

impl std::error::Error for TransitionError {}

#[derive(Debug)]
pub struct MissionMachine {
    state: MissionState,
}

impl Default for MissionMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl MissionMachine {
    pub fn new() -> Self {
        Self {
            state: MissionState::Received,
        }
    }

    pub fn state(&self) -> MissionState {
        self.state
    }

    pub fn transition(&mut self, to: MissionState) -> Result<(), TransitionError> {
        if allowed(self.state, to) {
            self.state = to;
            Ok(())
        } else {
            Err(TransitionError {
                from: self.state,
                to,
            })
        }
    }
}

pub fn allowed(from: MissionState, to: MissionState) -> bool {
    use MissionState::*;
    matches!(
        (from, to),
        (Received, Planned)
            | (Planned, Running)
            | (Running, Validating)
            | (Running, Failed)
            | (Running, DeadlineExceeded)
            | (Validating, Succeeded)
            | (Validating, Failed)
            | (Validating, DeadlineExceeded)
    )
}

#[derive(Debug, Clone)]
pub struct DeadlineGuard {
    started: Instant,
    budget: Duration,
}

impl DeadlineGuard {
    pub fn new(deadline_ms: u64) -> Result<Self, &'static str> {
        if deadline_ms == 0 {
            return Err("deadline_ms must be positive");
        }
        Ok(Self {
            started: Instant::now(),
            budget: Duration::from_millis(deadline_ms),
        })
    }

    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    pub fn expired(&self) -> bool {
        self.elapsed() > self.budget
    }

    pub fn remaining(&self) -> Duration {
        self.budget.saturating_sub(self.elapsed())
    }
}

pub fn aggregate_confidence(values: &[f64]) -> Result<f64, &'static str> {
    if values.is_empty() {
        return Err("confidence set cannot be empty");
    }
    if values.iter().any(|value| !(0.0..=1.0).contains(value)) {
        return Err("confidence must be within [0,1]");
    }
    Ok(values.iter().sum::<f64>() / values.len() as f64)
}

pub fn accept_result(guard: &DeadlineGuard, confidence: f64, minimum_confidence: f64) -> bool {
    !guard.expired()
        && (0.0..=1.0).contains(&confidence)
        && (0.0..=1.0).contains(&minimum_confidence)
        && confidence >= minimum_confidence
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn state_machine_happy_path() {
        let mut machine = MissionMachine::new();
        machine.transition(MissionState::Planned).unwrap();
        machine.transition(MissionState::Running).unwrap();
        machine.transition(MissionState::Validating).unwrap();
        machine.transition(MissionState::Succeeded).unwrap();
        assert!(machine.state().is_terminal());
    }

    #[test]
    fn state_machine_rejects_illegal_transition() {
        let mut machine = MissionMachine::new();
        let error = machine.transition(MissionState::Succeeded).unwrap_err();
        assert_eq!(error.from, MissionState::Received);
        assert_eq!(machine.state(), MissionState::Received);
    }

    #[test]
    fn deadline_expires() {
        let guard = DeadlineGuard::new(5).unwrap();
        thread::sleep(Duration::from_millis(12));
        assert!(guard.expired());
        assert_eq!(guard.remaining(), Duration::ZERO);
    }

    #[test]
    fn zero_deadline_is_invalid() {
        assert!(DeadlineGuard::new(0).is_err());
    }

    #[test]
    fn confidence_is_bounded_and_aggregated() {
        let score = aggregate_confidence(&[0.9, 1.0, 0.8]).unwrap();
        assert!((score - 0.9).abs() < 1e-9);
        assert!(aggregate_confidence(&[1.1]).is_err());
    }

    #[test]
    fn result_gate_requires_time_and_confidence() {
        let guard = DeadlineGuard::new(100).unwrap();
        assert!(accept_result(&guard, 0.95, 0.90));
        assert!(!accept_result(&guard, 0.80, 0.90));
    }
}
