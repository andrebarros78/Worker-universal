use std::collections::BTreeSet;

use tma_foundation::model::{IdempotencyClass, ResolveRequest};
use tma_foundation::registry::CapabilityRegistry;
use tma_foundation::services::{PolicyContext, PolicyEngine};

use crate::model::{ExecutionPlan, MissionRequest, RoutedPlan, RoutedStep};
use crate::plan::{plan_fingerprint, topological_order};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteError {
    InvalidPlan(String),
    NoCapability(String),
    PolicyDenied(String),
    DeadlineBudgetExceeded { routed: u64, budget: u64 },
    CostBudgetExceeded { routed: u64, budget: u64 },
}

impl std::fmt::Display for RouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for RouteError {}

pub struct Router<'a> {
    registry: &'a CapabilityRegistry,
    policy: &'a dyn PolicyEngine,
}

impl<'a> Router<'a> {
    pub fn new(registry: &'a CapabilityRegistry, policy: &'a dyn PolicyEngine) -> Self {
        Self { registry, policy }
    }

    pub fn route(
        &self,
        request: &MissionRequest,
        plan: &ExecutionPlan,
    ) -> Result<RoutedPlan, RouteError> {
        let order =
            topological_order(plan).map_err(|error| RouteError::InvalidPlan(error.to_string()))?;
        let context = PolicyContext {
            granted_scopes: request.granted_scopes.clone(),
            denied_capabilities: request.denied_capabilities.clone(),
        };

        let mut routed = Vec::with_capacity(plan.steps.len());
        let mut total_cost = 0_u64;
        let mut total_latency = 0_u64;

        for step_id in order {
            let step = plan
                .steps
                .iter()
                .find(|candidate| candidate.id == step_id)
                .ok_or_else(|| RouteError::InvalidPlan(step_id.clone()))?;

            let resolve = ResolveRequest {
                kind: Some(step.requirement.kind),
                required_contract: step.requirement.required_contract,
                required_permissions: step.requirement.required_permissions.clone(),
                required_evidence: step.requirement.required_evidence.clone(),
                minimum_promotion: step.requirement.minimum_promotion,
                minimum_reliability_bps: step.requirement.minimum_reliability_bps,
                maximum_cost_microunits: step.requirement.maximum_cost_microunits,
                maximum_p95_ms: step.requirement.maximum_p95_ms,
                provider_preference: step.requirement.provider_preference.clone(),
                allow_degraded: request.allow_degraded,
            };

            let candidates = self.registry.resolve(&resolve);
            if candidates.is_empty() {
                return Err(RouteError::NoCapability(step.id.clone()));
            }

            let mut permitted = Vec::new();
            for candidate in candidates {
                let decision = self.policy.evaluate(&context, candidate);
                if decision.allowed {
                    permitted.push(candidate);
                }
            }
            if permitted.is_empty() {
                return Err(RouteError::PolicyDenied(step.id.clone()));
            }

            let selected = permitted[0];
            let mut fallback_chain = Vec::new();
            let mut seen = BTreeSet::new();
            seen.insert(selected.id.clone());

            for candidate in permitted.iter().skip(1) {
                if seen.insert(candidate.id.clone()) {
                    fallback_chain.push(candidate.id.clone());
                }
            }
            for fallback in self
                .registry
                .fallback_chain(&selected.id, request.allow_degraded)
            {
                if seen.insert(fallback.id.clone()) {
                    fallback_chain.push(fallback.id.clone());
                }
            }

            total_cost = total_cost.saturating_add(selected.cost_microunits);
            total_latency = total_latency.saturating_add(selected.p95_ms);

            routed.push(RoutedStep {
                step_id: step.id.clone(),
                capability_id: selected.id.clone(),
                provider: selected.provider.clone(),
                fallback_chain,
                side_effect: selected.side_effect,
                idempotency: selected.idempotency,
                replay_safe: step.replay_safe && selected.idempotency != IdempotencyClass::Unsafe,
                resource_locks: selected.resource_locks.clone(),
                cost_microunits: selected.cost_microunits,
                p95_ms: selected.p95_ms,
                reliability_bps: selected.reliability_bps,
            });
        }

        if total_cost > request.cost_budget_microunits {
            return Err(RouteError::CostBudgetExceeded {
                routed: total_cost,
                budget: request.cost_budget_microunits,
            });
        }
        if total_latency > request.deadline_budget_ms {
            return Err(RouteError::DeadlineBudgetExceeded {
                routed: total_latency,
                budget: request.deadline_budget_ms,
            });
        }

        Ok(RoutedPlan {
            mission_id: plan.mission_id.clone(),
            fingerprint: plan_fingerprint(plan),
            steps: routed,
            total_cost_microunits: total_cost,
            total_p95_ms: total_latency,
        })
    }
}
