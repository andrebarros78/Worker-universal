use std::collections::BTreeMap;

use tma_core::MissionState;
use tma_core::durable::{
    ClaimOutcome, DurableStore, MissionSpec, ResultSubmission, ValidationReceipt,
};
use tma_economic::{
    DispatchResources, EconomicPolicy, IncomeReport, Opportunity, Rejection, dashboard_json,
    observed_success_from_missions, schedule_registered,
};
use tma_foundation::model::{
    CapabilityDescriptor, CapabilityKind, IdempotencyClass, LifecycleState, PromotionState,
    ReadinessState, SideEffectClass, TransportKind, Version,
};
use tma_foundation::registry::CapabilityRegistry;

fn descriptor(id: &str) -> CapabilityDescriptor {
    CapabilityDescriptor {
        id: id.to_string(),
        kind: CapabilityKind::Native,
        provider: Some("local".to_string()),
        contract_version: Version::new(1, 0, 0),
        dependencies: vec![],
        permissions: vec![],
        fallbacks: vec![],
        lifecycle: LifecycleState::Healthy,
        readiness: ReadinessState::Operational,
        promotion: PromotionState::Experimental,
        side_effect: SideEffectClass::None,
        idempotency: IdempotencyClass::Safe,
        max_concurrency: 4,
        resource_locks: vec![],
        cost_microunits: 0,
        p95_ms: 10,
        reliability_bps: 10000,
        evidence_types: vec!["result".to_string()],
        transport: TransportKind::Native,
        config_ref: None,
        enabled: true,
    }
}
fn opportunity() -> Opportunity {
    Opportunity {
        opportunity_id: "a".to_string(),
        mission_id: "mission-a".to_string(),
        reward_cents: 10000,
        estimated_cost_cents: 1000,
        success_probability_bps: 9000,
        expected_duration_ms: 60000,
        deadline_ms: 100000,
        required_slots: 1,
        resource_locks: vec![],
        eligible: true,
        qualified: true,
    }
}
fn policy() -> EconomicPolicy {
    EconomicPolicy {
        now_ms: 0,
        minimum_remaining_ms: 1000,
        minimum_expected_profit_cents: 100,
        minimum_margin_bps: 1000,
        maximum_cost_per_mission_cents: 2000,
        maximum_daily_cost_cents: 10000,
        stop_loss_cents: 5000,
        max_concurrency: 4,
        available_slots: 4,
    }
}
#[test]
fn f02_registry_experimental_adapter_cannot_self_qualify_via_input_flag() {
    let mut registry = CapabilityRegistry::new();
    registry.register(descriptor("native.work")).unwrap();
    let mapping = BTreeMap::from([("mission-a".to_string(), "native.work".to_string())]);
    let opp = opportunity();
    let result = schedule_registered(
        std::slice::from_ref(&opp),
        &mapping,
        &registry,
        &policy(),
        &IncomeReport::default(),
        &DispatchResources::default(),
    );
    assert_eq!(result.selected.len(), 0);
    assert_eq!(result.rejected[0].reason, Rejection::NotQualified);
    for stage in [
        PromotionState::Tested,
        PromotionState::Qualified,
        PromotionState::Production,
    ] {
        registry.set_promotion("native.work", stage).unwrap();
    }
    let result = schedule_registered(
        std::slice::from_ref(&opp),
        &mapping,
        &registry,
        &policy(),
        &IncomeReport::default(),
        &DispatchResources::default(),
    );
    assert_eq!(result.selected.len(), 1);
    let mut unavailable = descriptor("native.other");
    unavailable.promotion = PromotionState::Experimental;
    registry.register(unavailable).unwrap();
    let denied = schedule_registered(
        &[opp],
        &BTreeMap::new(),
        &registry,
        &policy(),
        &IncomeReport::default(),
        &DispatchResources::default(),
    );
    assert_eq!(denied.rejected[0].reason, Rejection::NotQualified);
}
#[test]
fn income_dashboard_is_machine_readable_and_separates_cash_from_accrual() {
    let report = IncomeReport {
        verified_earnings_cents: 10000,
        received_payments_cents: 3000,
        costs_cents: 500,
        accrued_profit_cents: 9500,
        cash_profit_cents: 2500,
        accounts_receivable_cents: 7000,
        entries: 3,
    };
    let json = dashboard_json(&report);
    assert!(json.contains("\"verified_earnings\":10000"));
    assert!(json.contains("\"cash_profit\":2500"));
    assert!(json.contains("\"accounts_receivable\":7000"));
    assert!(json.starts_with("{\"currency\":\"BRL\""));
}
#[test]
fn observed_success_is_derived_from_durable_terminal_state_and_deduplicates_ids() {
    let path = std::env::temp_dir().join(format!(
        "tma-f10-observed-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let store = DurableStore::open(&path).unwrap();
    store
        .create_mission(
            &MissionSpec {
                mission_id: "m1".to_string(),
                task_id: "t1".to_string(),
                deadline_unix_ms: 100000,
                priority: 1,
                idempotency_key: "idem1".to_string(),
                payload_hash: "payload".to_string(),
            },
            100,
        )
        .unwrap();
    store
        .create_mission(
            &MissionSpec {
                mission_id: "m2".to_string(),
                task_id: "t2".to_string(),
                deadline_unix_ms: 100000,
                priority: 1,
                idempotency_key: "idem2".to_string(),
                payload_hash: "payload".to_string(),
            },
            110,
        )
        .unwrap();
    let grant = match store.claim_next("worker", 200, 10000).unwrap() {
        ClaimOutcome::Claimed(value) => value,
        ClaimOutcome::Empty => panic!("expected"),
    };
    store
        .transition_with_lease(
            "m1",
            "worker",
            grant.fencing_token,
            MissionState::Validating,
            220,
        )
        .unwrap();
    store
        .record_validation_receipt(&ValidationReceipt {
            receipt_id: "receipt-m1".to_string(),
            mission_id: "m1".to_string(),
            validator_id: "independent".to_string(),
            subject_payload_hash: "validated".to_string(),
            evidence_digest: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            accepted: true,
            created_at_ms: 230,
        })
        .unwrap();
    store
        .submit_result(&ResultSubmission {
            result_id: "result-m1".to_string(),
            mission_id: "m1".to_string(),
            worker_id: "worker".to_string(),
            fencing_token: grant.fencing_token,
            target_state: MissionState::Succeeded,
            payload_hash: "validated".to_string(),
            validation_receipt_id: Some("receipt-m1".to_string()),
            created_at_ms: 250,
        })
        .unwrap();
    let stats = observed_success_from_missions(
        &store,
        &["m1".to_string(), "m1".to_string(), "m2".to_string()],
    )
    .unwrap();
    assert_eq!(stats.completed, 1);
    assert_eq!(stats.succeeded, 1);
    assert_eq!(stats.probability_bps, 10000);
    drop(store);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}
