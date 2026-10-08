use std::collections::{BTreeMap, BTreeSet, VecDeque};

use sha2::{Digest, Sha256};
use tma_foundation::model::{CapabilityKind, SideEffectClass};

use crate::model::{ExecutionPlan, MissionRequest, TaskKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    InvalidRequest(String),
    DuplicateStep(String),
    MissingDependency { step: String, dependency: String },
    DependencyCycle,
    DeadlineBudgetExceeded { planned: u64, budget: u64 },
    CostBudgetExceeded { planned: u64, budget: u64 },
    MissingScope { step: String, scope: String },
    LlmNotAllowed(String),
    EmptyPlan,
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PlanError {}

pub fn validate_plan(
    request: &MissionRequest,
    plan: &ExecutionPlan,
) -> Result<Vec<String>, PlanError> {
    request
        .validate()
        .map_err(|message| PlanError::InvalidRequest(message.to_string()))?;
    if plan.mission_id != request.mission_id {
        return Err(PlanError::InvalidRequest("mission_id mismatch".to_string()));
    }
    if plan.steps.is_empty() {
        return Err(PlanError::EmptyPlan);
    }

    let mut ids = BTreeSet::new();
    for step in &plan.steps {
        if step.id.trim().is_empty() || !ids.insert(step.id.clone()) {
            return Err(PlanError::DuplicateStep(step.id.clone()));
        }
        if step.requirement.kind == CapabilityKind::Llm && !request.allow_llm {
            return Err(PlanError::LlmNotAllowed(step.id.clone()));
        }
        for scope in &step.requirement.required_permissions {
            if !request.granted_scopes.contains(scope) {
                return Err(PlanError::MissingScope {
                    step: step.id.clone(),
                    scope: scope.clone(),
                });
            }
        }
    }

    for step in &plan.steps {
        for dependency in &step.depends_on {
            if !ids.contains(dependency) {
                return Err(PlanError::MissingDependency {
                    step: step.id.clone(),
                    dependency: dependency.clone(),
                });
            }
        }
    }

    let cost = plan
        .steps
        .iter()
        .map(|step| step.budget_cost_microunits)
        .sum::<u64>();
    if cost > request.cost_budget_microunits {
        return Err(PlanError::CostBudgetExceeded {
            planned: cost,
            budget: request.cost_budget_microunits,
        });
    }

    let latency = plan
        .steps
        .iter()
        .map(|step| step.budget_latency_ms)
        .sum::<u64>();
    if latency > request.deadline_budget_ms {
        return Err(PlanError::DeadlineBudgetExceeded {
            planned: latency,
            budget: request.deadline_budget_ms,
        });
    }

    topological_order(plan)
}

pub fn topological_order(plan: &ExecutionPlan) -> Result<Vec<String>, PlanError> {
    let mut indegree = BTreeMap::<String, usize>::new();
    let mut children = BTreeMap::<String, Vec<String>>::new();

    for step in &plan.steps {
        indegree.insert(step.id.clone(), step.depends_on.len());
        for dependency in &step.depends_on {
            children
                .entry(dependency.clone())
                .or_default()
                .push(step.id.clone());
        }
    }

    let mut ready = VecDeque::new();
    for (id, degree) in &indegree {
        if *degree == 0 {
            ready.push_back(id.clone());
        }
    }

    let mut order = Vec::with_capacity(plan.steps.len());
    while let Some(id) = ready.pop_front() {
        order.push(id.clone());
        if let Some(next) = children.get(&id) {
            let mut sorted = next.clone();
            sorted.sort();
            for child in sorted {
                let degree = indegree.get_mut(&child).expect("known child");
                *degree -= 1;
                if *degree == 0 {
                    ready.push_back(child);
                }
            }
        }
    }

    if order.len() != plan.steps.len() {
        return Err(PlanError::DependencyCycle);
    }
    Ok(order)
}

pub fn canonical_plan(plan: &ExecutionPlan) -> String {
    let mut lines = vec![
        format!("mission={}", plan.mission_id),
        format!("class={:?}", plan.class),
    ];
    for step in &plan.steps {
        let mut dependencies = step.depends_on.clone();
        dependencies.sort();
        let mut permissions = step.requirement.required_permissions.clone();
        permissions.sort();
        let mut evidence = step.requirement.required_evidence.clone();
        evidence.sort();
        lines.push(format!(
            "step={}|kind={:?}|deps={}|cap={:?}|contract={}|permissions={}|evidence={}|promotion={:?}|reliability={}|max_cost={:?}|max_p95={:?}|providers={}|side_effect={:?}|replay_safe={}|budget_cost={}|budget_latency={}",
            step.id,
            step.task_kind,
            dependencies.join(","),
            step.requirement.kind,
            step.requirement.required_contract,
            permissions.join(","),
            evidence.join(","),
            step.requirement.minimum_promotion,
            step.requirement.minimum_reliability_bps,
            step.requirement.maximum_cost_microunits,
            step.requirement.maximum_p95_ms,
            step.requirement.provider_preference.join(","),
            step.side_effect,
            step.replay_safe,
            step.budget_cost_microunits,
            step.budget_latency_ms,
        ));
    }
    lines.join("\n")
}

pub fn plan_fingerprint(plan: &ExecutionPlan) -> String {
    let digest = Sha256::digest(canonical_plan(plan).as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepTemplate {
    pub id: String,
    pub task_kind: TaskKind,
    pub kind: CapabilityKind,
    pub dependencies: Vec<String>,
    pub evidence: Vec<String>,
    pub side_effect: SideEffectClass,
    pub replay_safe: bool,
    pub budget_cost_microunits: u64,
    pub budget_latency_ms: u64,
}

pub fn deterministic_step(template: StepTemplate) -> crate::model::PlanStep {
    crate::model::PlanStep {
        id: template.id,
        task_kind: template.task_kind,
        depends_on: template.dependencies,
        requirement: crate::model::CapabilityRequirement {
            kind: template.kind,
            required_contract: tma_foundation::model::Version::new(1, 0, 0),
            required_permissions: Vec::new(),
            required_evidence: template.evidence,
            minimum_promotion: tma_foundation::model::PromotionState::Tested,
            minimum_reliability_bps: 9_000,
            maximum_cost_microunits: Some(template.budget_cost_microunits),
            maximum_p95_ms: Some(template.budget_latency_ms),
            provider_preference: Vec::new(),
        },
        side_effect: template.side_effect,
        replay_safe: template.replay_safe,
        budget_cost_microunits: template.budget_cost_microunits,
        budget_latency_ms: template.budget_latency_ms,
    }
}
