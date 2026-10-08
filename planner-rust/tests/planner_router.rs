use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tma_core::durable::{ClaimOutcome, DurableStore, MissionSpec};
use tma_foundation::model::{
    CapabilityDescriptor, CapabilityKind, IdempotencyClass, LifecycleState, PromotionState,
    ReadinessState, SideEffectClass, TransportKind, Version,
};
use tma_foundation::registry::CapabilityRegistry;
use tma_foundation::services::ScopePolicy;
use tma_planner::{
    CapabilityRequirement, DeterministicPlanner, ExecutionPlan, KnowledgeQuery, KnowledgeSource,
    LocalResearchProvider, LocalRuleReasoner, MissionClass, MissionClassifier, MissionRequest,
    PlanError, PlanStep, Planner, ReasonerOutput, ReasonerProvider, ReasonerRequest,
    ResearchProvider, RouteError, Router, TaskKind, canonical_plan, persist_plan_checkpoint,
    plan_fingerprint, recover_plan_checkpoint, validate_plan, validate_reasoner_output,
};

fn request(objective: &str) -> MissionRequest {
    MissionRequest {
        mission_id: "mission-001".to_string(),
        objective: objective.to_string(),
        deadline_budget_ms: 20_000,
        cost_budget_microunits: 10_000,
        granted_scopes: BTreeSet::new(),
        denied_capabilities: BTreeSet::new(),
        allow_llm: false,
        allow_degraded: false,
    }
}

fn transport(kind: CapabilityKind) -> TransportKind {
    match kind {
        CapabilityKind::Api => TransportKind::Api,
        CapabilityKind::Mcp => TransportKind::Mcp,
        CapabilityKind::Browser => TransportKind::Browser,
        CapabilityKind::Computer => TransportKind::Computer,
        CapabilityKind::Llm => TransportKind::Ai,
        _ => TransportKind::Native,
    }
}

fn descriptor(
    id: &str,
    kind: CapabilityKind,
    provider: &str,
    reliability: u16,
    cost: u64,
    p95: u64,
) -> CapabilityDescriptor {
    CapabilityDescriptor {
        id: id.to_string(),
        kind,
        provider: Some(provider.to_string()),
        contract_version: Version::new(1, 0, 0),
        dependencies: Vec::new(),
        permissions: Vec::new(),
        fallbacks: Vec::new(),
        lifecycle: LifecycleState::Healthy,
        readiness: ReadinessState::Operational,
        promotion: PromotionState::Production,
        side_effect: SideEffectClass::Read,
        idempotency: IdempotencyClass::Safe,
        max_concurrency: 8,
        resource_locks: Vec::new(),
        cost_microunits: cost,
        p95_ms: p95,
        reliability_bps: reliability,
        evidence_types: Vec::new(),
        transport: transport(kind),
        config_ref: None,
        enabled: true,
    }
}

fn single_step_plan(kind: CapabilityKind, provider_preference: Vec<String>) -> ExecutionPlan {
    ExecutionPlan {
        mission_id: "mission-001".to_string(),
        class: MissionClass::Deterministic,
        steps: vec![PlanStep {
            id: "step-1".to_string(),
            task_kind: TaskKind::Transform,
            depends_on: Vec::new(),
            requirement: CapabilityRequirement {
                kind,
                required_contract: Version::new(1, 0, 0),
                required_permissions: Vec::new(),
                required_evidence: Vec::new(),
                minimum_promotion: PromotionState::Tested,
                minimum_reliability_bps: 9_000,
                maximum_cost_microunits: Some(5_000),
                maximum_p95_ms: Some(5_000),
                provider_preference,
            },
            side_effect: SideEffectClass::Read,
            replay_safe: true,
            budget_cost_microunits: 5_000,
            budget_latency_ms: 5_000,
        }],
    }
}

fn db_path(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "tma-f06-{name}-{}-{nonce}.sqlite3",
        std::process::id()
    ))
}

fn cleanup_db(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}

#[test]
fn classifier_is_deterministic_across_supported_classes() {
    let classifier = MissionClassifier;
    assert_eq!(
        classifier.classify("transform these values"),
        MissionClass::Deterministic
    );
    assert_eq!(
        classifier.classify("extract OCR from invoice document"),
        MissionClass::Document
    );
    assert_eq!(
        classifier.classify("fill website form in browser"),
        MissionClass::Browser
    );
    assert_eq!(
        classifier.classify("pesquisa knowledge sobre o tema"),
        MissionClass::Research
    );
    assert_eq!(
        classifier.classify("research invoice and submit browser form"),
        MissionClass::Composite
    );
}

#[test]
fn deterministic_plan_works_without_llm_or_api() {
    let req = request("deterministic transform");
    let plan = DeterministicPlanner::new().plan(&req).unwrap();
    assert_eq!(plan.class, MissionClass::Deterministic);
    assert!(
        plan.steps
            .iter()
            .all(|step| step.requirement.kind == CapabilityKind::Native)
    );
    assert_eq!(
        validate_plan(&req, &plan).unwrap(),
        vec!["transform", "validate"]
    );
}

#[test]
fn research_plan_uses_local_reasoner_when_llm_is_disabled() {
    let req = request("research knowledge about local evidence");
    let plan = DeterministicPlanner::new().plan(&req).unwrap();
    assert_eq!(plan.class, MissionClass::Research);
    assert_eq!(plan.steps[0].requirement.kind, CapabilityKind::Research);
    assert_eq!(plan.steps[1].requirement.kind, CapabilityKind::Native);
}

#[test]
fn invalid_dependency_and_cycle_are_rejected() {
    let req = request("deterministic");
    let mut missing = DeterministicPlanner::new().plan(&req).unwrap();
    missing.steps[1].depends_on = vec!["does-not-exist".to_string()];
    assert!(matches!(
        validate_plan(&req, &missing),
        Err(PlanError::MissingDependency { .. })
    ));

    let mut cycle = DeterministicPlanner::new().plan(&req).unwrap();
    cycle.steps[0].depends_on = vec!["validate".to_string()];
    assert!(matches!(
        validate_plan(&req, &cycle),
        Err(PlanError::DependencyCycle)
    ));
}

#[test]
fn plan_budget_is_hard_bound() {
    let mut req = request("deterministic");
    req.deadline_budget_ms = 1;
    req.cost_budget_microunits = 1;
    assert!(matches!(
        DeterministicPlanner::new().plan(&req),
        Err(PlanError::DeadlineBudgetExceeded { .. }) | Err(PlanError::CostBudgetExceeded { .. })
    ));
}

#[test]
fn api_mcp_and_browser_selection_fixtures_route_correct_kinds() {
    let policy = ScopePolicy;

    for (kind, id) in [
        (CapabilityKind::Api, "api.fixture"),
        (CapabilityKind::Mcp, "mcp.fixture"),
        (CapabilityKind::Browser, "browser.fixture"),
    ] {
        let req = request("manual fixture");
        let plan = single_step_plan(kind, Vec::new());
        let mut registry = CapabilityRegistry::new();
        registry
            .register(descriptor(id, kind, "fixture", 9_900, 10, 100))
            .unwrap();
        let routed = Router::new(&registry, &policy).route(&req, &plan).unwrap();
        assert_eq!(routed.steps[0].capability_id, id);
    }
}

#[test]
fn provider_preference_overrides_otherwise_better_candidate() {
    let req = request("manual fixture");
    let plan = single_step_plan(
        CapabilityKind::Api,
        vec!["preferred".to_string(), "other".to_string()],
    );
    let mut registry = CapabilityRegistry::new();
    registry
        .register(descriptor(
            "api.other",
            CapabilityKind::Api,
            "other",
            10_000,
            1,
            1,
        ))
        .unwrap();
    registry
        .register(descriptor(
            "api.preferred",
            CapabilityKind::Api,
            "preferred",
            9_500,
            100,
            100,
        ))
        .unwrap();
    let routed = Router::new(&registry, &ScopePolicy)
        .route(&req, &plan)
        .unwrap();
    assert_eq!(routed.steps[0].capability_id, "api.preferred");
}

#[test]
fn deterministic_tie_break_is_reliability_cost_latency_then_id() {
    let req = request("manual fixture");
    let plan = single_step_plan(CapabilityKind::Api, Vec::new());
    let mut registry = CapabilityRegistry::new();
    for item in [
        descriptor("api.z", CapabilityKind::Api, "x", 9_900, 10, 100),
        descriptor("api.a", CapabilityKind::Api, "x", 9_900, 10, 100),
        descriptor("api.cheaper", CapabilityKind::Api, "x", 9_900, 5, 200),
        descriptor("api.reliable", CapabilityKind::Api, "x", 9_950, 100, 500),
    ] {
        registry.register(item).unwrap();
    }
    let routed = Router::new(&registry, &ScopePolicy)
        .route(&req, &plan)
        .unwrap();
    assert_eq!(routed.steps[0].capability_id, "api.reliable");
}

#[test]
fn failed_primary_routes_to_ready_fallback_candidate() {
    let req = request("manual fixture");
    let plan = single_step_plan(
        CapabilityKind::Api,
        vec!["primary".to_string(), "fallback".to_string()],
    );
    let mut registry = CapabilityRegistry::new();
    let mut primary = descriptor("api.primary", CapabilityKind::Api, "primary", 9_999, 1, 1);
    primary.lifecycle = LifecycleState::Failed;
    registry.register(primary).unwrap();
    registry
        .register(descriptor(
            "api.fallback",
            CapabilityKind::Api,
            "fallback",
            9_500,
            10,
            100,
        ))
        .unwrap();

    let routed = Router::new(&registry, &ScopePolicy)
        .route(&req, &plan)
        .unwrap();
    assert_eq!(routed.steps[0].capability_id, "api.fallback");
}

#[test]
fn missing_dependency_makes_capability_unroutable() {
    let req = request("manual fixture");
    let plan = single_step_plan(CapabilityKind::Api, Vec::new());
    let mut registry = CapabilityRegistry::new();
    let mut item = descriptor("api.dep", CapabilityKind::Api, "x", 9_900, 10, 100);
    item.dependencies = vec!["native.missing".to_string()];
    registry.register(item).unwrap();

    assert!(matches!(
        Router::new(&registry, &ScopePolicy).route(&req, &plan),
        Err(RouteError::NoCapability(_))
    ));
}

#[test]
fn policy_scope_is_enforced_even_when_registry_matches() {
    let req = request("manual fixture");
    let plan = single_step_plan(CapabilityKind::Api, Vec::new());
    let mut registry = CapabilityRegistry::new();
    let mut item = descriptor("api.secured", CapabilityKind::Api, "x", 9_900, 10, 100);
    item.permissions = vec!["network.external".to_string()];
    registry.register(item).unwrap();

    assert!(matches!(
        Router::new(&registry, &ScopePolicy).route(&req, &plan),
        Err(RouteError::PolicyDenied(_))
    ));
}

#[test]
fn vision_fixture_routes_f03_capability_and_propagates_metadata() {
    let req = request("invoice document OCR");
    let plan = DeterministicPlanner::new().plan(&req).unwrap();
    let mut registry = CapabilityRegistry::new();

    let mut vision = descriptor(
        "vision.ocr.tesseract",
        CapabilityKind::Vision,
        "tesseract",
        9_900,
        0,
        1_500,
    );
    vision.evidence_types = vec!["ocr_text".to_string(), "field_confidence".to_string()];
    vision.resource_locks = vec!["ocr.local".to_string()];
    registry.register(vision).unwrap();

    registry
        .register(descriptor(
            "native.validator",
            CapabilityKind::Native,
            "local",
            10_000,
            0,
            10,
        ))
        .unwrap();

    let routed = Router::new(&registry, &ScopePolicy)
        .route(&req, &plan)
        .unwrap();
    assert_eq!(routed.steps[0].capability_id, "vision.ocr.tesseract");
    assert_eq!(routed.steps[0].resource_locks, vec!["ocr.local"]);
    assert!(routed.steps[0].replay_safe);
}

#[test]
fn unsafe_capability_disables_replay_even_if_step_is_replay_safe() {
    let req = request("manual fixture");
    let plan = single_step_plan(CapabilityKind::Api, Vec::new());
    let mut registry = CapabilityRegistry::new();
    let mut item = descriptor("api.unsafe", CapabilityKind::Api, "x", 9_900, 10, 100);
    item.idempotency = IdempotencyClass::Unsafe;
    registry.register(item).unwrap();

    let routed = Router::new(&registry, &ScopePolicy)
        .route(&req, &plan)
        .unwrap();
    assert!(!routed.steps[0].replay_safe);
}

#[test]
fn local_research_is_bounded_and_local_reasoner_output_is_validated() {
    let mut research = LocalResearchProvider::default();
    research.insert(KnowledgeSource {
        source_id: "s2".to_string(),
        title: "Rust durable runtime".to_string(),
        excerpt: "SQLite WAL persists mission state.".to_string(),
    });
    research.insert(KnowledgeSource {
        source_id: "s1".to_string(),
        title: "Rust planner".to_string(),
        excerpt: "Deterministic routing uses capability health.".to_string(),
    });
    research.insert(KnowledgeSource {
        source_id: "s3".to_string(),
        title: "Unrelated".to_string(),
        excerpt: "Nothing relevant.".to_string(),
    });

    let query = KnowledgeQuery {
        query_id: "q1".to_string(),
        question: "Rust durable planner".to_string(),
        max_sources: 1,
        deadline_ms: 1_000,
        cost_budget_microunits: 0,
    };
    let result = research.research(&query).unwrap();
    assert_eq!(result.sources.len(), 1);
    assert_eq!(result.cost_microunits, 0);

    let reasoner_request = ReasonerRequest {
        request_id: "r1".to_string(),
        instruction: "Summarize evidence".to_string(),
        evidence: result.sources.clone(),
        deadline_ms: 1_000,
        cost_budget_microunits: 0,
    };
    let output = LocalRuleReasoner.reason(&reasoner_request).unwrap();
    validate_reasoner_output(&reasoner_request, &output).unwrap();

    let invalid = ReasonerOutput {
        provider_id: "external".to_string(),
        text: "unsupported claim".to_string(),
        evidence_ids: vec!["invented".to_string()],
        cost_microunits: 0,
    };
    assert!(validate_reasoner_output(&reasoner_request, &invalid).is_err());
}

#[test]
fn canonical_plan_fingerprint_is_stable() {
    let req = request("deterministic transform");
    let plan_a = DeterministicPlanner::new().plan(&req).unwrap();
    let plan_b = DeterministicPlanner::new().plan(&req).unwrap();
    assert_eq!(canonical_plan(&plan_a), canonical_plan(&plan_b));
    assert_eq!(plan_fingerprint(&plan_a), plan_fingerprint(&plan_b));
    assert_eq!(plan_fingerprint(&plan_a).len(), 64);
}

#[test]
fn durable_checkpoint_survives_restart_and_new_lease() {
    let path = db_path("restart");
    let store = DurableStore::open(&path).unwrap();
    store
        .create_mission(
            &MissionSpec {
                mission_id: "mission-001".to_string(),
                task_id: "planner-task".to_string(),
                deadline_unix_ms: 50_000,
                priority: 10,
                idempotency_key: "planner-idem".to_string(),
                payload_hash: "planner-payload".to_string(),
            },
            100,
        )
        .unwrap();

    let grant = match store.claim_next("planner-a", 200, 100).unwrap() {
        ClaimOutcome::Claimed(grant) => grant,
        ClaimOutcome::Empty => panic!("expected lease"),
    };
    let req = request("deterministic transform");
    let plan = DeterministicPlanner::new().plan(&req).unwrap();
    let checkpoint = persist_plan_checkpoint(
        &store,
        "mission-001",
        "planner-a",
        grant.fencing_token,
        &plan,
        220,
    )
    .unwrap();
    drop(store);

    let reopened = DurableStore::open(&path).unwrap();
    reopened.reconcile(500).unwrap();
    let recovered = recover_plan_checkpoint(&reopened, "mission-001").unwrap();
    assert_eq!(recovered, checkpoint);

    let second = match reopened.claim_next("planner-b", 501, 500).unwrap() {
        ClaimOutcome::Claimed(grant) => grant,
        ClaimOutcome::Empty => panic!("expected recovered lease"),
    };
    assert!(second.fencing_token > grant.fencing_token);
    assert_eq!(second.checkpoint, checkpoint.encode());

    cleanup_db(&path);
}
