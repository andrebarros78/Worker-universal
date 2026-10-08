use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tma_core::MissionState;
use tma_core::durable::{
    ClaimOutcome, DurableStore, MissionSpec, ResultDisposition, ResultSubmission, ValidationReceipt,
};
use tma_economic::{
    EconomicEntry, EconomicLedger, EconomicPolicy, EntryKind, IncomeReport, Opportunity,
    RecordOutcome, Rejection, observed_success, schedule, score,
};

fn db_path(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "tma-f10-{prefix}-{}-{nonce}.db",
        std::process::id()
    ))
}
fn cleanup(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}
fn entry(
    key: &str,
    mission: &str,
    kind: EntryKind,
    amount: i64,
    source: &str,
    time: i64,
) -> EconomicEntry {
    EconomicEntry {
        event_key: key.to_string(),
        mission_id: mission.to_string(),
        kind,
        amount_cents: amount,
        source_ref: source.to_string(),
        recorded_at_ms: time,
    }
}
fn core_with_validated_success(path: &Path) -> DurableStore {
    let core = DurableStore::open(path).unwrap();
    core.create_mission(
        &MissionSpec {
            mission_id: "m1".to_string(),
            task_id: "task1".to_string(),
            deadline_unix_ms: 1_000_000,
            priority: 1,
            idempotency_key: "idem-m1".to_string(),
            payload_hash: "work-hash".to_string(),
        },
        100,
    )
    .unwrap();
    let grant = match core.claim_next("executor", 200, 10000).unwrap() {
        ClaimOutcome::Claimed(grant) => grant,
        ClaimOutcome::Empty => panic!("mission expected"),
    };
    core.transition_with_lease(
        "m1",
        "executor",
        grant.fencing_token,
        MissionState::Validating,
        220,
    )
    .unwrap();
    core.record_validation_receipt(&ValidationReceipt {
        receipt_id: "receipt-m1".to_string(),
        mission_id: "m1".to_string(),
        validator_id: "independent-validator".to_string(),
        subject_payload_hash: "validated-result".to_string(),
        evidence_digest: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        accepted: true,
        created_at_ms: 230,
    })
    .unwrap();
    assert_eq!(
        core.submit_result(&ResultSubmission {
            result_id: "result-m1".to_string(),
            mission_id: "m1".to_string(),
            worker_id: "executor".to_string(),
            fencing_token: grant.fencing_token,
            target_state: MissionState::Succeeded,
            payload_hash: "validated-result".to_string(),
            validation_receipt_id: Some("receipt-m1".to_string()),
            created_at_ms: 240,
        })
        .unwrap(),
        ResultDisposition::Accepted
    );
    core
}
fn policy() -> EconomicPolicy {
    EconomicPolicy {
        now_ms: 1000,
        minimum_remaining_ms: 100,
        minimum_expected_profit_cents: 100,
        minimum_margin_bps: 1000,
        maximum_cost_per_mission_cents: 2000,
        maximum_daily_cost_cents: 5000,
        stop_loss_cents: 2500,
        max_concurrency: 4,
        available_slots: 4,
    }
}
fn opportunity(id: &str, reward: i64, cost: i64, p: u16, duration: u64) -> Opportunity {
    Opportunity {
        opportunity_id: id.to_string(),
        mission_id: format!("mission-{id}"),
        reward_cents: reward,
        estimated_cost_cents: cost,
        success_probability_bps: p,
        expected_duration_ms: duration,
        deadline_ms: 1000000,
        required_slots: 1,
        resource_locks: vec![],
        eligible: true,
        qualified: true,
    }
}
fn plan(
    opps: &[Opportunity],
    policy: &EconomicPolicy,
    report: &IncomeReport,
    spent_today: i64,
) -> tma_economic::SchedulePlan {
    schedule(opps, policy, report, 0, spent_today, 0, &[])
}

#[test]
fn fixed_point_expected_value_and_margin_are_deterministic() {
    let c = score(&opportunity("a", 5000, 1000, 8000, 60000)).unwrap();
    assert_eq!(c.expected_revenue_cents, 4000);
    assert_eq!(c.expected_profit_cents, 3000);
    assert_eq!(c.margin_bps, 7500);
    assert_eq!(c.profit_per_hour_cents, 180000);
}
#[test]
fn unknown_history_is_not_fabricated_as_success() {
    assert_eq!(observed_success(0, 0).unwrap().probability_bps, 0);
    assert_eq!(observed_success(4, 3).unwrap().probability_bps, 7500);
    assert!(observed_success(2, 3).is_err());
}
#[test]
fn negative_ev_mission_is_rejected_before_execution() {
    let p = plan(
        &[opportunity("loss", 100, 900, 5000, 1000)],
        &policy(),
        &IncomeReport::default(),
        0,
    );
    assert!(p.selected.is_empty());
    assert_eq!(p.rejected[0].reason, Rejection::NegativeValue);
}
#[test]
fn cost_spike_changes_priority_without_floats() {
    let a = opportunity("a", 5000, 1000, 9000, 60000);
    let b = opportunity("b", 5000, 1200, 9000, 60000);
    let ranked = plan(
        &[a.clone(), b.clone()],
        &policy(),
        &IncomeReport::default(),
        0,
    );
    assert_eq!(ranked.selected[0].opportunity_id, "a");
    let mut spike = a;
    spike.estimated_cost_cents = 1900;
    let reranked = plan(&[spike, b], &policy(), &IncomeReport::default(), 0);
    assert_eq!(reranked.selected[0].opportunity_id, "b");
}
#[test]
fn slots_and_resource_locks_bound_parallel_selection() {
    let mut a = opportunity("a", 5000, 100, 10000, 1000);
    let mut b = opportunity("b", 4000, 100, 10000, 1000);
    let c = opportunity("c", 3000, 100, 10000, 1000);
    a.resource_locks = vec!["browser-session".to_string()];
    b.resource_locks = vec!["browser-session".to_string()];
    let result = plan(&[a, b, c], &policy(), &IncomeReport::default(), 0);
    assert_eq!(result.selected.len(), 2);
    assert_eq!(result.rejected[0].reason, Rejection::ResourceLock);
    assert_eq!(result.allocated_slots, 2);
    let limited = EconomicPolicy {
        max_concurrency: 1,
        available_slots: 1,
        ..policy()
    };
    assert_eq!(
        plan(
            &[
                opportunity("d", 5000, 100, 10000, 1000),
                opportunity("e", 4000, 100, 10000, 1000)
            ],
            &limited,
            &IncomeReport::default(),
            0
        )
        .selected
        .len(),
        1
    );
}
#[test]
fn daily_spending_plus_reserved_budget_is_hard_capped() {
    let cap = EconomicPolicy {
        maximum_daily_cost_cents: 3000,
        ..policy()
    };
    let ops = [
        opportunity("a", 5000, 2000, 10000, 1000),
        opportunity("b", 4000, 1800, 10000, 1000),
    ];
    let p = plan(&ops, &cap, &IncomeReport::default(), 1200);
    assert_eq!(p.selected.len(), 1);
    assert_eq!(p.selected[0].opportunity_id, "b");
    assert_eq!(p.rejected[0].reason, Rejection::DailyBudget);
}
#[test]
fn stop_loss_blocks_even_profitable_new_opportunities() {
    let report = IncomeReport {
        costs_cents: 4000,
        accrued_profit_cents: -3000,
        entries: 1,
        ..Default::default()
    };
    let p = plan(
        &[opportunity("a", 20000, 100, 10000, 1000)],
        &policy(),
        &report,
        4000,
    );
    assert_eq!(p.rejected[0].reason, Rejection::StopLoss);
}
#[test]
fn deadline_eligibility_and_qualification_are_hard_gates() {
    let mut op = opportunity("a", 5000, 10, 10000, 1000);
    op.eligible = false;
    assert_eq!(
        plan(&[op.clone()], &policy(), &IncomeReport::default(), 0).rejected[0].reason,
        Rejection::Ineligible
    );
    op.eligible = true;
    op.qualified = false;
    assert_eq!(
        plan(&[op.clone()], &policy(), &IncomeReport::default(), 0).rejected[0].reason,
        Rejection::NotQualified
    );
    op.qualified = true;
    op.deadline_ms = 1001;
    assert_eq!(
        plan(&[op], &policy(), &IncomeReport::default(), 0).rejected[0].reason,
        Rejection::Deadline
    );
}
#[test]
fn minimum_margin_mission_cost_cap_and_duplicate_ids_are_enforced() {
    let p = policy();
    assert_eq!(
        plan(
            &[opportunity("margin", 10000, 1900, 2000, 1000)],
            &p,
            &IncomeReport::default(),
            0
        )
        .rejected[0]
            .reason,
        Rejection::Margin
    );
    assert_eq!(
        plan(
            &[opportunity("cap", 10000, 2100, 10000, 1000)],
            &p,
            &IncomeReport::default(),
            0
        )
        .rejected[0]
            .reason,
        Rejection::CostCap
    );
    let op = opportunity("a", 5000, 100, 10000, 1000);
    let result = plan(&[op.clone(), op], &p, &IncomeReport::default(), 0);
    assert_eq!(result.selected.len(), 1);
    assert_eq!(result.rejected[0].reason, Rejection::Duplicate);
}
#[test]
fn deterministic_ties_and_input_order_do_not_change_fingerprint() {
    let a = opportunity("a", 5000, 100, 10000, 1000);
    let b = opportunity("b", 5000, 100, 10000, 1000);
    let left = plan(
        &[a.clone(), b.clone()],
        &policy(),
        &IncomeReport::default(),
        0,
    );
    let right = plan(&[b, a], &policy(), &IncomeReport::default(), 0);
    assert_eq!(left, right);
    assert_eq!(left.fingerprint.len(), 64);
}
#[test]
fn invalid_probability_money_and_duration_are_rejected() {
    assert_eq!(
        score(&opportunity("a", 5000, -1, 10000, 1000)).unwrap_err(),
        Rejection::Invalid
    );
    assert_eq!(
        score(&opportunity("a", 5000, 0, 10001, 1000)).unwrap_err(),
        Rejection::Invalid
    );
    assert_eq!(
        score(&opportunity("a", 5000, 0, 10000, 0)).unwrap_err(),
        Rejection::Invalid
    );
}
#[test]
fn only_f05_validated_success_can_accrue_verified_earnings() {
    let econ_path = db_path("revenue");
    let core_path = db_path("core");
    let ledger = EconomicLedger::open(&econ_path).unwrap();
    let empty = DurableStore::open(&core_path).unwrap();
    let earn = entry(
        "earned-1",
        "m1",
        EntryKind::VerifiedEarning,
        10000,
        "validated-result",
        300,
    );
    assert!(ledger.record_verified_earning(&empty, &earn).is_err());
    drop(empty);
    let core = core_with_validated_success(&core_path);
    let mut forged = earn.clone();
    forged.source_ref = "forged".to_string();
    assert!(ledger.record_verified_earning(&core, &forged).is_err());
    assert_eq!(
        ledger.record_verified_earning(&core, &earn).unwrap(),
        RecordOutcome::Recorded
    );
    assert_eq!(
        ledger.record_verified_earning(&core, &earn).unwrap(),
        RecordOutcome::Duplicate
    );
    let mut conflicting = earn;
    conflicting.amount_cents += 1;
    assert!(ledger.record_verified_earning(&core, &conflicting).is_err());
    assert_eq!(ledger.report().unwrap().verified_earnings_cents, 10000);
    cleanup(&econ_path);
    cleanup(&core_path);
}
#[test]
fn earned_revenue_cash_and_cost_are_separate_and_reopen_safe() {
    let econ_path = db_path("cash");
    let core_path = db_path("cash-core");
    let core = core_with_validated_success(&core_path);
    let ledger = EconomicLedger::open(&econ_path).unwrap();
    ledger
        .record_verified_earning(
            &core,
            &entry(
                "earned",
                "m1",
                EntryKind::VerifiedEarning,
                10000,
                "validated-result",
                300,
            ),
        )
        .unwrap();
    ledger
        .record_cost(&entry(
            "cost",
            "m1",
            EntryKind::Cost,
            1500,
            "invoice-1",
            400,
        ))
        .unwrap();
    ledger
        .record_payment(&entry(
            "paid",
            "m1",
            EntryKind::PaymentReceived,
            4000,
            "settlement-1",
            500,
        ))
        .unwrap();
    let report = ledger.report().unwrap();
    assert_eq!(report.verified_earnings_cents, 10000);
    assert_eq!(report.received_payments_cents, 4000);
    assert_eq!(report.costs_cents, 1500);
    assert_eq!(report.accrued_profit_cents, 8500);
    assert_eq!(report.cash_profit_cents, 2500);
    assert_eq!(report.accounts_receivable_cents, 6000);
    assert_eq!(ledger.verify_chain().unwrap(), 3);
    drop(ledger);
    let reopened = EconomicLedger::open(&econ_path).unwrap();
    assert_eq!(reopened.report().unwrap(), report);
    assert_eq!(reopened.integrity_check().unwrap(), "ok");
    cleanup(&econ_path);
    cleanup(&core_path);
}
#[test]
fn payment_cannot_exceed_verified_earnings_or_be_double_counted() {
    let path = db_path("payments");
    let core_path = db_path("payment-core");
    let core = core_with_validated_success(&core_path);
    let ledger = EconomicLedger::open(&path).unwrap();
    let paid = entry(
        "p1",
        "m1",
        EntryKind::PaymentReceived,
        5000,
        "bank-reference",
        400,
    );
    assert!(ledger.record_payment(&paid).is_err());
    ledger
        .record_verified_earning(
            &core,
            &entry(
                "earn",
                "m1",
                EntryKind::VerifiedEarning,
                5000,
                "validated-result",
                300,
            ),
        )
        .unwrap();
    assert_eq!(
        ledger.record_payment(&paid).unwrap(),
        RecordOutcome::Recorded
    );
    assert_eq!(
        ledger.record_payment(&paid).unwrap(),
        RecordOutcome::Duplicate
    );
    assert!(
        ledger
            .record_payment(&entry(
                "p2",
                "m1",
                EntryKind::PaymentReceived,
                1,
                "bank-ref-2",
                500
            ))
            .is_err()
    );
    assert_eq!(ledger.report().unwrap().received_payments_cents, 5000);
    cleanup(&path);
    cleanup(&core_path);
}
#[test]
fn costs_since_respects_day_boundary_without_wiping_historical_costs() {
    let path = db_path("daily");
    let ledger = EconomicLedger::open(&path).unwrap();
    ledger
        .record_cost(&entry(
            "yesterday",
            "m1",
            EntryKind::Cost,
            200,
            "billing",
            100,
        ))
        .unwrap();
    ledger
        .record_cost(&entry(
            "today",
            "m1",
            EntryKind::Cost,
            500,
            "billing",
            90000,
        ))
        .unwrap();
    assert_eq!(ledger.costs_since(86400).unwrap(), 500);
    assert_eq!(ledger.report().unwrap().costs_cents, 700);
    cleanup(&path);
}
#[test]
fn ledger_is_append_only_and_hash_chain_detects_tampering() {
    let path = db_path("append");
    let ledger = EconomicLedger::open(&path).unwrap();
    ledger
        .record_cost(&entry("cost", "m1", EntryKind::Cost, 300, "receipt", 100))
        .unwrap();
    assert_eq!(ledger.verify_chain().unwrap(), 1);
    let raw = rusqlite::Connection::open(&path).unwrap();
    assert!(
        raw.execute(
            "UPDATE economic_events SET amount_cents=5 WHERE event_key='cost'",
            []
        )
        .is_err()
    );
    assert!(
        raw.execute("DELETE FROM economic_events WHERE event_key='cost'", [])
            .is_err()
    );
    raw.execute_batch("DROP TRIGGER economic_events_no_update")
        .unwrap();
    raw.execute(
        "UPDATE economic_events SET amount_cents=5 WHERE event_key='cost'",
        [],
    )
    .unwrap();
    assert!(ledger.verify_chain().is_err());
    drop(raw);
    cleanup(&path);
}
#[test]
fn invalid_entry_and_cross_kind_api_are_rejected() {
    let path = db_path("invalid");
    let ledger = EconomicLedger::open(&path).unwrap();
    assert!(
        ledger
            .record_cost(&entry("bad", "m", EntryKind::Cost, -1, "receipt", 10))
            .is_err()
    );
    assert!(
        ledger
            .record_cost(&entry(
                "wrong",
                "m",
                EntryKind::PaymentReceived,
                1,
                "receipt",
                10
            ))
            .is_err()
    );
    assert!(
        ledger
            .record_payment(&entry("wrong", "m", EntryKind::Cost, 1, "receipt", 10))
            .is_err()
    );
    assert_eq!(ledger.report().unwrap().entries, 0);
    cleanup(&path);
}

#[test]
fn one_f05_mission_cannot_generate_multiple_verified_earnings() {
    let path = db_path("once");
    let core_path = db_path("once-core");
    let core = core_with_validated_success(&core_path);
    let ledger = EconomicLedger::open(&path).unwrap();
    assert_eq!(
        ledger
            .record_verified_earning(
                &core,
                &entry(
                    "earning-1",
                    "m1",
                    EntryKind::VerifiedEarning,
                    10000,
                    "validated-result",
                    300
                )
            )
            .unwrap(),
        RecordOutcome::Recorded
    );
    assert!(
        ledger
            .record_verified_earning(
                &core,
                &entry(
                    "earning-2",
                    "m1",
                    EntryKind::VerifiedEarning,
                    10000,
                    "validated-result",
                    301
                )
            )
            .is_err()
    );
    assert_eq!(ledger.report().unwrap().verified_earnings_cents, 10000);
    cleanup(&path);
    cleanup(&core_path);
}

#[test]
fn two_payment_keys_cannot_replay_same_external_settlement_ref() {
    let path = db_path("settlement");
    let core_path = db_path("settlement-core");
    let core = core_with_validated_success(&core_path);
    let ledger = EconomicLedger::open(&path).unwrap();
    ledger
        .record_verified_earning(
            &core,
            &entry(
                "earning",
                "m1",
                EntryKind::VerifiedEarning,
                10000,
                "validated-result",
                300,
            ),
        )
        .unwrap();
    ledger
        .record_payment(&entry(
            "payment-1",
            "m1",
            EntryKind::PaymentReceived,
            2000,
            "bank-settle-1",
            400,
        ))
        .unwrap();
    assert!(
        ledger
            .record_payment(&entry(
                "payment-2",
                "m1",
                EntryKind::PaymentReceived,
                2000,
                "bank-settle-1",
                401
            ))
            .is_err()
    );
    assert_eq!(ledger.report().unwrap().received_payments_cents, 2000);
    cleanup(&path);
    cleanup(&core_path);
}

#[test]
fn sixteen_parallel_writers_cannot_duplicate_single_cost_event() {
    let path = db_path("parallel");
    // Prepare the durable database once; concurrent workers only write events.
    let initialized = EconomicLedger::open(&path).unwrap();
    assert_eq!(initialized.integrity_check().unwrap(), "ok");
    drop(initialized);
    let mut handles = Vec::new();
    for _ in 0..16 {
        let path = path.clone();
        handles.push(std::thread::spawn(move || {
            let ledger = EconomicLedger::open(&path).unwrap();
            ledger
                .record_cost(&entry(
                    "cost-once",
                    "m1",
                    EntryKind::Cost,
                    100,
                    "invoice-1",
                    100,
                ))
                .unwrap()
        }));
    }
    let outcomes = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes
            .iter()
            .filter(|value| **value == RecordOutcome::Recorded)
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|value| **value == RecordOutcome::Duplicate)
            .count(),
        15
    );
    let ledger = EconomicLedger::open(&path).unwrap();
    assert_eq!(ledger.report().unwrap().costs_cents, 100);
    assert_eq!(ledger.verify_chain().unwrap(), 1);
    cleanup(&path);
}
