use std::collections::{BTreeSet, VecDeque};

use tma_foundation::model::{
    CapabilityDescriptor, CapabilityKind, IdempotencyClass, LifecycleState, PromotionState,
    ReadinessState, SideEffectClass, TransportKind, Version,
};
use tma_foundation::registry::CapabilityRegistry;
use tma_foundation::services::ScopePolicy;
use tma_planner::{DeterministicPlanner, MissionRequest, Planner, Router};

fn descriptor(id: &str, reliability: u16, cost: u64, p95: u64) -> CapabilityDescriptor {
    CapabilityDescriptor {
        id: id.to_string(),
        kind: CapabilityKind::Native,
        provider: Some("local".to_string()),
        contract_version: Version::new(1, 0, 0),
        dependencies: Vec::new(),
        permissions: Vec::new(),
        fallbacks: Vec::new(),
        lifecycle: LifecycleState::Healthy,
        readiness: ReadinessState::Operational,
        promotion: PromotionState::Production,
        side_effect: SideEffectClass::None,
        idempotency: IdempotencyClass::Safe,
        max_concurrency: 8,
        resource_locks: Vec::new(),
        cost_microunits: cost,
        p95_ms: p95,
        reliability_bps: reliability,
        evidence_types: Vec::new(),
        transport: TransportKind::Native,
        config_ref: None,
        enabled: true,
    }
}

fn main() {
    let mut args = std::env::args().skip(1).collect::<VecDeque<_>>();
    if args.pop_front().as_deref() != Some("self-test") {
        eprintln!("usage: tma-planner self-test");
        std::process::exit(2);
    }

    let request = MissionRequest {
        mission_id: "self-test".to_string(),
        objective: "deterministic transform".to_string(),
        deadline_budget_ms: 1_000,
        cost_budget_microunits: 10,
        granted_scopes: BTreeSet::new(),
        denied_capabilities: BTreeSet::new(),
        allow_llm: false,
        allow_degraded: false,
    };
    let plan = DeterministicPlanner::new().plan(&request).expect("plan");

    let mut registry = CapabilityRegistry::new();
    registry
        .register(descriptor("native.local", 10_000, 0, 10))
        .expect("register");
    let policy = ScopePolicy;
    let routed = Router::new(&registry, &policy)
        .route(&request, &plan)
        .expect("route");

    println!(
        "tma-planner status=ok class={:?} steps={} routed={} fingerprint={}",
        plan.class,
        plan.steps.len(),
        routed.steps.len(),
        routed.fingerprint
    );
}
