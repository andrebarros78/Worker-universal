use std::collections::BTreeSet;

use tma_foundation::model::{
    CapabilityKind, IdempotencyClass, PromotionState, SideEffectClass, Version,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MissionClass {
    Deterministic,
    Document,
    Browser,
    Research,
    Composite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskKind {
    Transform,
    ExtractDocument,
    BrowserAction,
    Research,
    Reason,
    Validate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionRequest {
    pub mission_id: String,
    pub objective: String,
    pub deadline_budget_ms: u64,
    pub cost_budget_microunits: u64,
    pub granted_scopes: BTreeSet<String>,
    pub denied_capabilities: BTreeSet<String>,
    pub allow_llm: bool,
    pub allow_degraded: bool,
}

impl MissionRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.mission_id.trim().is_empty() {
            return Err("mission_id cannot be empty");
        }
        if self.objective.trim().is_empty() {
            return Err("objective cannot be empty");
        }
        if self.deadline_budget_ms == 0 {
            return Err("deadline budget must be positive");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityRequirement {
    pub kind: CapabilityKind,
    pub required_contract: Version,
    pub required_permissions: Vec<String>,
    pub required_evidence: Vec<String>,
    pub minimum_promotion: PromotionState,
    pub minimum_reliability_bps: u16,
    pub maximum_cost_microunits: Option<u64>,
    pub maximum_p95_ms: Option<u64>,
    pub provider_preference: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanStep {
    pub id: String,
    pub task_kind: TaskKind,
    pub depends_on: Vec<String>,
    pub requirement: CapabilityRequirement,
    pub side_effect: SideEffectClass,
    pub replay_safe: bool,
    pub budget_cost_microunits: u64,
    pub budget_latency_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub mission_id: String,
    pub class: MissionClass,
    pub steps: Vec<PlanStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedStep {
    pub step_id: String,
    pub capability_id: String,
    pub provider: Option<String>,
    pub fallback_chain: Vec<String>,
    pub side_effect: SideEffectClass,
    pub idempotency: IdempotencyClass,
    pub replay_safe: bool,
    pub resource_locks: Vec<String>,
    pub cost_microunits: u64,
    pub p95_ms: u64,
    pub reliability_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedPlan {
    pub mission_id: String,
    pub fingerprint: String,
    pub steps: Vec<RoutedStep>,
    pub total_cost_microunits: u64,
    pub total_p95_ms: u64,
}
