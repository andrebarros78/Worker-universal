use tma_foundation::model::{CapabilityKind, SideEffectClass};

use crate::model::{ExecutionPlan, MissionClass, MissionRequest, TaskKind};
use crate::plan::{PlanError, StepTemplate, deterministic_step, validate_plan};

pub trait Planner {
    fn plan(&self, request: &MissionRequest) -> Result<ExecutionPlan, PlanError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MissionClassifier;

impl MissionClassifier {
    pub fn classify(&self, objective: &str) -> MissionClass {
        let text = objective.to_lowercase();
        let words = text
            .split(|ch: char| !ch.is_alphanumeric())
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>();
        let has_word = |value: &str| words.contains(&value);
        let has_prefix = |prefix: &str| words.iter().any(|word| word.starts_with(prefix));

        let document = ["invoice", "document", "ocr", "nota", "documento"]
            .iter()
            .any(|token| has_word(token));
        let browser = has_word("browser")
            || has_word("website")
            || has_word("form")
            || has_word("navegador")
            || has_prefix("formul");
        let research = ["research", "pesquisa", "investigar", "lookup", "knowledge"]
            .iter()
            .any(|token| has_word(token));

        let count = usize::from(document) + usize::from(browser) + usize::from(research);
        if count > 1 {
            MissionClass::Composite
        } else if document {
            MissionClass::Document
        } else if browser {
            MissionClass::Browser
        } else if research {
            MissionClass::Research
        } else {
            MissionClass::Deterministic
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicPlanner {
    classifier: MissionClassifier,
}

impl DeterministicPlanner {
    pub fn new() -> Self {
        Self::default()
    }

    fn split_budget(total: u64, parts: usize) -> u64 {
        if parts == 0 {
            total
        } else {
            (total / parts as u64).max(1)
        }
    }
}

impl Planner for DeterministicPlanner {
    fn plan(&self, request: &MissionRequest) -> Result<ExecutionPlan, PlanError> {
        request
            .validate()
            .map_err(|message| PlanError::InvalidRequest(message.to_string()))?;
        let class = self.classifier.classify(&request.objective);
        let step_count = match class {
            MissionClass::Deterministic => 2,
            MissionClass::Document => 2,
            MissionClass::Browser => 2,
            MissionClass::Research => 2,
            MissionClass::Composite => 4,
        };
        let step_cost = Self::split_budget(request.cost_budget_microunits, step_count);
        let step_latency = Self::split_budget(request.deadline_budget_ms, step_count);

        let steps = match class {
            MissionClass::Deterministic => vec![
                deterministic_step(StepTemplate {
                    id: "transform".to_string(),
                    task_kind: TaskKind::Transform,
                    kind: CapabilityKind::Native,
                    dependencies: vec![],
                    evidence: vec![],
                    side_effect: SideEffectClass::None,
                    replay_safe: true,
                    budget_cost_microunits: step_cost,
                    budget_latency_ms: step_latency,
                }),
                deterministic_step(StepTemplate {
                    id: "validate".to_string(),
                    task_kind: TaskKind::Validate,
                    kind: CapabilityKind::Native,
                    dependencies: vec!["transform".to_string()],
                    evidence: vec![],
                    side_effect: SideEffectClass::Read,
                    replay_safe: true,
                    budget_cost_microunits: step_cost,
                    budget_latency_ms: step_latency,
                }),
            ],
            MissionClass::Document => vec![
                deterministic_step(StepTemplate {
                    id: "extract-document".to_string(),
                    task_kind: TaskKind::ExtractDocument,
                    kind: CapabilityKind::Vision,
                    dependencies: vec![],
                    evidence: vec!["ocr_text".to_string(), "field_confidence".to_string()],
                    side_effect: SideEffectClass::Read,
                    replay_safe: true,
                    budget_cost_microunits: step_cost,
                    budget_latency_ms: step_latency,
                }),
                deterministic_step(StepTemplate {
                    id: "validate".to_string(),
                    task_kind: TaskKind::Validate,
                    kind: CapabilityKind::Native,
                    dependencies: vec!["extract-document".to_string()],
                    evidence: vec![],
                    side_effect: SideEffectClass::Read,
                    replay_safe: true,
                    budget_cost_microunits: step_cost,
                    budget_latency_ms: step_latency,
                }),
            ],
            MissionClass::Browser => vec![
                deterministic_step(StepTemplate {
                    id: "browser-action".to_string(),
                    task_kind: TaskKind::BrowserAction,
                    kind: CapabilityKind::Browser,
                    dependencies: vec![],
                    evidence: vec!["dom".to_string(), "screenshot".to_string()],
                    side_effect: SideEffectClass::ExternalEffect,
                    replay_safe: false,
                    budget_cost_microunits: step_cost,
                    budget_latency_ms: step_latency,
                }),
                deterministic_step(StepTemplate {
                    id: "validate".to_string(),
                    task_kind: TaskKind::Validate,
                    kind: CapabilityKind::Native,
                    dependencies: vec!["browser-action".to_string()],
                    evidence: vec![],
                    side_effect: SideEffectClass::Read,
                    replay_safe: true,
                    budget_cost_microunits: step_cost,
                    budget_latency_ms: step_latency,
                }),
            ],
            MissionClass::Research => vec![
                deterministic_step(StepTemplate {
                    id: "research".to_string(),
                    task_kind: TaskKind::Research,
                    kind: CapabilityKind::Research,
                    dependencies: vec![],
                    evidence: vec!["sources".to_string()],
                    side_effect: SideEffectClass::Read,
                    replay_safe: true,
                    budget_cost_microunits: step_cost,
                    budget_latency_ms: step_latency,
                }),
                deterministic_step(StepTemplate {
                    id: "reason".to_string(),
                    task_kind: TaskKind::Reason,
                    kind: if request.allow_llm {
                        CapabilityKind::Llm
                    } else {
                        CapabilityKind::Native
                    },
                    dependencies: vec!["research".to_string()],
                    evidence: vec![],
                    side_effect: SideEffectClass::None,
                    replay_safe: true,
                    budget_cost_microunits: step_cost,
                    budget_latency_ms: step_latency,
                }),
            ],
            MissionClass::Composite => vec![
                deterministic_step(StepTemplate {
                    id: "extract-document".to_string(),
                    task_kind: TaskKind::ExtractDocument,
                    kind: CapabilityKind::Vision,
                    dependencies: vec![],
                    evidence: vec!["ocr_text".to_string()],
                    side_effect: SideEffectClass::Read,
                    replay_safe: true,
                    budget_cost_microunits: step_cost,
                    budget_latency_ms: step_latency,
                }),
                deterministic_step(StepTemplate {
                    id: "research".to_string(),
                    task_kind: TaskKind::Research,
                    kind: CapabilityKind::Research,
                    dependencies: vec![],
                    evidence: vec!["sources".to_string()],
                    side_effect: SideEffectClass::Read,
                    replay_safe: true,
                    budget_cost_microunits: step_cost,
                    budget_latency_ms: step_latency,
                }),
                deterministic_step(StepTemplate {
                    id: "browser-action".to_string(),
                    task_kind: TaskKind::BrowserAction,
                    kind: CapabilityKind::Browser,
                    dependencies: vec!["extract-document".to_string(), "research".to_string()],
                    evidence: vec!["dom".to_string(), "screenshot".to_string()],
                    side_effect: SideEffectClass::ExternalEffect,
                    replay_safe: false,
                    budget_cost_microunits: step_cost,
                    budget_latency_ms: step_latency,
                }),
                deterministic_step(StepTemplate {
                    id: "validate".to_string(),
                    task_kind: TaskKind::Validate,
                    kind: CapabilityKind::Native,
                    dependencies: vec!["browser-action".to_string()],
                    evidence: vec![],
                    side_effect: SideEffectClass::Read,
                    replay_safe: true,
                    budget_cost_microunits: step_cost,
                    budget_latency_ms: step_latency,
                }),
            ],
        };

        let plan = ExecutionPlan {
            mission_id: request.mission_id.clone(),
            class,
            steps,
        };
        validate_plan(request, &plan)?;
        Ok(plan)
    }
}
