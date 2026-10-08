use tma_core::durable::{DurableResult, DurableStore};

use crate::model::ExecutionPlan;
use crate::plan::plan_fingerprint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanCheckpoint {
    pub fingerprint: String,
    pub step_count: usize,
}

impl PlanCheckpoint {
    pub fn encode(&self) -> String {
        format!(
            "planner_v1;fingerprint={};steps={}",
            self.fingerprint, self.step_count
        )
    }

    pub fn decode(value: &str) -> Result<Self, String> {
        let mut fingerprint = None;
        let mut step_count = None;
        for part in value.split(';') {
            if let Some(value) = part.strip_prefix("fingerprint=") {
                fingerprint = Some(value.to_string());
            } else if let Some(value) = part.strip_prefix("steps=") {
                step_count = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| "invalid step count".to_string())?,
                );
            }
        }
        let fingerprint = fingerprint.ok_or_else(|| "missing fingerprint".to_string())?;
        if fingerprint.len() != 64 || !fingerprint.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Err("invalid fingerprint".to_string());
        }
        Ok(Self {
            fingerprint,
            step_count: step_count.ok_or_else(|| "missing step count".to_string())?,
        })
    }
}

pub fn persist_plan_checkpoint(
    store: &DurableStore,
    mission_id: &str,
    worker_id: &str,
    fencing_token: i64,
    plan: &ExecutionPlan,
    now_ms: i64,
) -> DurableResult<PlanCheckpoint> {
    let checkpoint = PlanCheckpoint {
        fingerprint: plan_fingerprint(plan),
        step_count: plan.steps.len(),
    };
    store.save_checkpoint(
        mission_id,
        worker_id,
        fencing_token,
        &checkpoint.encode(),
        now_ms,
    )?;
    Ok(checkpoint)
}

pub fn recover_plan_checkpoint(
    store: &DurableStore,
    mission_id: &str,
) -> Result<PlanCheckpoint, String> {
    let record = store
        .mission(mission_id)
        .map_err(|error| error.to_string())?;
    PlanCheckpoint::decode(&record.checkpoint)
}
