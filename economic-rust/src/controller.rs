use std::collections::{BTreeMap, BTreeSet};
use tma_core::MissionState;
use tma_core::durable::DurableStore;
use tma_foundation::model::{LifecycleState, PromotionState, ReadinessState};
use tma_foundation::registry::CapabilityRegistry;

use crate::model::{
    DispatchResources, EconomicPolicy, IncomeReport, Opportunity, SchedulePlan, SuccessHistory,
};
use crate::scheduler::{observed_success, schedule};

/// Production-facing evaluation. The F02 registry, not adapter claims, decides
/// capability qualification and readiness. Pure schedule() is a test/what-if
/// function and confers no permission to execute work.
pub fn schedule_registered(
    opportunities: &[Opportunity],
    mission_capabilities: &BTreeMap<String, String>,
    registry: &CapabilityRegistry,
    policy: &EconomicPolicy,
    report: &IncomeReport,
    resources: &DispatchResources,
) -> SchedulePlan {
    let filtered = opportunities
        .iter()
        .map(|op| {
            let mut checked = op.clone();
            let authorized = mission_capabilities
                .get(&op.mission_id)
                .and_then(|capability_id| registry.get(capability_id))
                .is_some_and(|descriptor| {
                    descriptor.enabled
                        && descriptor.lifecycle == LifecycleState::Healthy
                        && descriptor.readiness == ReadinessState::Operational
                        && descriptor.promotion == PromotionState::Production
                });
            checked.qualified = op.qualified && authorized;
            checked
        })
        .collect::<Vec<_>>();
    schedule(
        &filtered,
        policy,
        report,
        resources.reserved_prior_cents,
        resources.spent_today_cents,
        resources.busy_slots,
        &resources.busy_resource_locks,
    )
}

/// Convert observed durable F05 terminal results to success probability.
/// Failed/nonterminal outcomes are not fabricated as successful.
pub fn observed_success_from_missions(
    store: &DurableStore,
    mission_ids: &[String],
) -> Result<SuccessHistory, String> {
    let mut completed = 0usize;
    let mut succeeded = 0usize;
    let unique = mission_ids.iter().collect::<BTreeSet<_>>();
    for id in unique {
        let mission = store.mission(id).map_err(|e| e.to_string())?;
        if mission.state.is_terminal() {
            completed += 1;
            if mission.state == MissionState::Succeeded {
                succeeded += 1;
            }
        }
    }
    observed_success(completed, succeeded)
}
