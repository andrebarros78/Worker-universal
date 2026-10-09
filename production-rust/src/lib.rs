use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use tma_core::{
    MissionState,
    durable::{ClaimOutcome, DurableStore, MissionSpec, ResultDisposition, ResultSubmission},
};
use tma_economic::{
    DispatchResources, EconomicEntry, EconomicLedger, EconomicPolicy, EntryKind, IncomeReport,
    Opportunity, RecordOutcome, Rejection, schedule_registered,
};
use tma_foundation::{
    model::{
        CapabilityDescriptor, CapabilityKind, IdempotencyClass, LifecycleState, PromotionState,
        ReadinessState, SideEffectClass, TransportKind, Version,
    },
    registry::CapabilityRegistry,
};
use tma_planner::{
    DeterministicPlanner, MissionRequest, Planner, persist_plan_checkpoint, recover_plan_checkpoint,
};
use tma_validation::{
    EvidenceItem, EvidenceValidator, ExecutorResult, IndependentValidator, ValidationContext,
    finalize_validated_success,
};

#[derive(Debug, Clone)]
pub struct RehearsalReport {
    pub mission_id: String,
    pub terminal_hash: String,
    pub ledger_events: usize,
    pub snapshots: usize,
    pub cost_cents: i64,
    pub accrued_revenue_cents: i64,
    pub payment_cents: i64,
    pub provenance: &'static str,
}

fn sha(input: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(input);
    format!("{:x}", h.finalize())
}
fn gate(error: impl std::fmt::Display) -> String {
    error.to_string()
}

/// Only locally authorized reversible transformations. No external adapter or payment.
pub fn run_local_rehearsal(dir: &Path, source: &[u8], now: i64) -> Result<RehearsalReport, String> {
    if source.is_empty() || source.len() > 1_000_000 || !source.is_ascii() {
        return Err("F13_UNSUPPORTED_LOCAL_INPUT".into());
    }
    let spec = MissionSpec {
        mission_id: "f13-local-rehearsal".to_string(),
        task_id: "normalize-ascii-v1".into(),
        deadline_unix_ms: now + 120_000,
        priority: 1,
        idempotency_key: "f13-local-rehearsal-idempotent".into(),
        payload_hash: sha(source),
    };
    let store = DurableStore::open(dir.join("missions.sqlite3")).map_err(gate)?;
    store.create_mission(&spec, now).map_err(gate)?;
    let request = MissionRequest {
        mission_id: spec.mission_id.clone(),
        objective: "normalize deterministic local text".into(),
        deadline_budget_ms: 100_000,
        cost_budget_microunits: 100,
        granted_scopes: BTreeSet::new(),
        denied_capabilities: BTreeSet::new(),
        allow_llm: false,
        allow_degraded: false,
    };
    let plan = DeterministicPlanner::new().plan(&request).map_err(gate)?;
    if plan.steps.len() != 2 {
        return Err("F13_PLAN_INCOMPLETE".into());
    }

    let first = match store
        .claim_next("worker-primary", now + 1, 100)
        .map_err(gate)?
    {
        ClaimOutcome::Claimed(x) => x,
        ClaimOutcome::Empty => return Err("F13_CLAIM_MISSING".into()),
    };
    let saved = persist_plan_checkpoint(
        &store,
        &spec.mission_id,
        "worker-primary",
        first.fencing_token,
        &plan,
        now + 2,
    )
    .map_err(gate)?;
    drop(store); // simulate terminated worker and reopen durable authority

    let store = DurableStore::open(dir.join("missions.sqlite3")).map_err(gate)?;
    let recovered = recover_plan_checkpoint(&store, &spec.mission_id)?;
    if recovered != saved {
        return Err("F13_PLAN_RECOVERY_MISMATCH".into());
    }
    let result = store.reconcile(now + 102).map_err(gate)?;
    if result.expired_leases != 1 {
        return Err("F13_EXPIRED_LEASE_NOT_RECONCILED".into());
    }
    let new_lease = match store
        .claim_next("worker-recovered", now + 103, 5000)
        .map_err(gate)?
    {
        ClaimOutcome::Claimed(x) => x,
        ClaimOutcome::Empty => return Err("F13_RECOVERY_NOT_CLAIMED".into()),
    };
    if new_lease.fencing_token != first.fencing_token + 1 {
        return Err("F13_FENCE_NOT_INCREMENTED".into());
    }
    let stale = store
        .submit_result(&ResultSubmission {
            result_id: "f13-stale-result".into(),
            mission_id: spec.mission_id.clone(),
            worker_id: "worker-primary".into(),
            fencing_token: first.fencing_token,
            target_state: MissionState::Failed,
            payload_hash: "stale".into(),
            validation_receipt_id: None,
            created_at_ms: now + 104,
        })
        .map_err(gate)?;
    if !matches!(stale, ResultDisposition::Rejected { .. }) {
        return Err("F13_STALE_WORKER_NOT_DENIED".into());
    }

    let transformed = source
        .iter()
        .map(|b| b.to_ascii_uppercase())
        .collect::<Vec<_>>();
    // Materialize independently verifiable evidence as a new, durable local artifact.
    // Never replace a prior result or expose any network/paid-provider side effect.
    let output_path = dir.join("result.txt");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output_path)
        .map_err(gate)?;
    use std::io::Write;
    file.write_all(&transformed).map_err(gate)?;
    file.sync_all().map_err(gate)?;
    let disk_evidence = std::fs::read(&output_path).map_err(gate)?;
    if disk_evidence != transformed {
        return Err("F13_ON_DISK_EVIDENCE_MISMATCH".into());
    }
    let digest = sha(&disk_evidence);
    let exact = source
        .iter()
        .map(|b| b.to_ascii_uppercase())
        .collect::<Vec<_>>();
    if transformed != exact || sha(&exact) != digest {
        return Err("F13_INDEPENDENT_CONTENT_CHECK_FAILED".into());
    }
    store
        .transition_with_lease(
            &spec.mission_id,
            "worker-recovered",
            new_lease.fencing_token,
            MissionState::Validating,
            now + 105,
        )
        .map_err(gate)?;
    let execution = ExecutorResult {
        mission_id: spec.mission_id.clone(),
        result_id: "f13-result-1".into(),
        worker_id: "worker-recovered".into(),
        capability_id: "local.native".into(),
        capability_kind: CapabilityKind::Native,
        payload_hash: digest.clone(),
        declared_success: true,
        required_evidence: vec![],
        evidence: vec![EvidenceItem {
            kind: "result".into(),
            sha256: sha(&disk_evidence),
            bytes: disk_evidence.len() as u64,
            confidence_bps: None,
        }],
        confidence_bps: Some(10_000),
        completed_at_ms: now + 106,
    };
    let report = EvidenceValidator.validate(
        &execution,
        &ValidationContext {
            validator_id: "independent-validator".into(),
            now_ms: now + 107,
            deadline_unix_ms: spec.deadline_unix_ms,
            minimum_confidence_bps: 0,
        },
    );
    if !report.is_proven() {
        return Err("F13_F07_VALIDATION_FAILED".into());
    }
    let acceptance = finalize_validated_success(
        &store,
        &report,
        "worker-recovered",
        new_lease.fencing_token,
        now + 108,
    )
    .map_err(gate)?;
    if !matches!(acceptance, ResultDisposition::Accepted) {
        return Err("F13_RESULT_NOT_ACCEPTED".into());
    }
    let mission = store.mission(&spec.mission_id).map_err(gate)?;
    if mission.state != MissionState::Succeeded
        || mission.terminal_result_hash.as_deref() != Some(&digest)
    {
        return Err("F13_TERMINAL_MISMATCH".into());
    }
    let events = store.verify_ledger().map_err(gate)?;
    if store.integrity_check().map_err(gate)? != "ok" || events.snapshots < 3 {
        return Err("F13_F05_JOURNAL_INVALID".into());
    }

    // A succeeded *synthetic* local transformation is NOT a paid earning.
    let econ = EconomicLedger::open(dir.join("cost.sqlite3")).map_err(gate)?;
    let cost = econ.record_cost(&EconomicEntry {
        event_key: "f13-local-cost".into(),
        mission_id: spec.mission_id.clone(),
        kind: EntryKind::Cost,
        amount_cents: 1,
        source_ref: "synthetic-compute-budget".into(),
        recorded_at_ms: now + 109,
    })?;
    if cost != RecordOutcome::Recorded {
        return Err("F13_COST_NOT_RECORDED".into());
    }
    if econ.verify_chain()? != 1 || econ.integrity_check()? != "ok" {
        return Err("F13_FINANCIAL_JOURNAL_INVALID".into());
    }
    let income = econ.report()?;
    if income.verified_earnings_cents != 0 || income.received_payments_cents != 0 {
        return Err("F13_SYNTHETIC_REVENUE_INFLATED".into());
    }
    Ok(RehearsalReport {
        mission_id: spec.mission_id,
        terminal_hash: digest,
        ledger_events: events.events,
        snapshots: events.snapshots,
        cost_cents: income.costs_cents,
        accrued_revenue_cents: income.verified_earnings_cents,
        payment_cents: income.received_payments_cents,
        provenance: "synthetic_local",
    })
}

/// Actual F02 promotion gate. Never self-promote the adapter based on input metadata.
pub fn production_gate_denies_unqualified() -> Result<(), String> {
    let mut registry = CapabilityRegistry::new();
    registry.register(CapabilityDescriptor {
        id: "f13.browser.unqualified".into(),
        kind: CapabilityKind::Browser,
        provider: Some("simulated".into()),
        contract_version: Version::new(1, 0, 0),
        dependencies: vec![],
        permissions: vec![],
        fallbacks: vec![],
        lifecycle: LifecycleState::Healthy,
        readiness: ReadinessState::Operational,
        promotion: PromotionState::Experimental,
        side_effect: SideEffectClass::ExternalEffect,
        idempotency: IdempotencyClass::Unsafe,
        max_concurrency: 1,
        resource_locks: vec![],
        cost_microunits: 0,
        p95_ms: 100,
        reliability_bps: 10_000,
        evidence_types: vec!["result".into()],
        transport: TransportKind::Browser,
        config_ref: None,
        enabled: true,
    })?;
    let opportunity = Opportunity {
        opportunity_id: "simulated-platform".into(),
        mission_id: "fake-revenue".into(),
        reward_cents: 10_000,
        estimated_cost_cents: 100,
        success_probability_bps: 10_000,
        expected_duration_ms: 1000,
        deadline_ms: 100_000,
        required_slots: 1,
        resource_locks: vec![],
        eligible: true,
        qualified: true,
    };
    let plan = schedule_registered(
        &[opportunity],
        &BTreeMap::from([("fake-revenue".into(), "f13.browser.unqualified".into())]),
        &registry,
        &EconomicPolicy {
            now_ms: 1,
            minimum_remaining_ms: 0,
            minimum_expected_profit_cents: 1,
            minimum_margin_bps: 1,
            maximum_cost_per_mission_cents: 1000,
            maximum_daily_cost_cents: 2000,
            stop_loss_cents: 500,
            max_concurrency: 1,
            available_slots: 1,
        },
        &IncomeReport::default(),
        &DispatchResources::default(),
    );
    if !plan.selected.is_empty()
        || plan.rejected.len() != 1
        || plan.rejected[0].reason != Rejection::NotQualified
    {
        return Err("F13_UNQUALIFIED_PLATFORM_ALLOWED".into());
    }
    Ok(())
}
