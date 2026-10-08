use tma_economic::{
    EconomicLedger, EconomicPolicy, IncomeReport, Opportunity, dashboard_json, schedule,
};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() == 2 && args[0] == "report" {
        let ledger = EconomicLedger::open(&args[1]).expect("ledger");
        let integrity = ledger.integrity_check().expect("integrity");
        if integrity != "ok" {
            eprintln!("ledger_integrity_failed");
            std::process::exit(1);
        }
        ledger.verify_chain().expect("chain");
        println!("{}", dashboard_json(&ledger.report().expect("report")));
        return;
    }
    if args.as_slice() != ["self-test"] {
        eprintln!("usage: tma-economic self-test | report <ledger.sqlite3>");
        std::process::exit(2);
    }
    let opportunity = Opportunity {
        opportunity_id: "synthetic-a".to_string(),
        mission_id: "mission-a".to_string(),
        reward_cents: 5000,
        estimated_cost_cents: 1000,
        success_probability_bps: 8000,
        expected_duration_ms: 60000,
        deadline_ms: 1000000,
        required_slots: 1,
        resource_locks: vec![],
        eligible: true,
        qualified: true,
    };
    let policy = EconomicPolicy {
        now_ms: 0,
        minimum_remaining_ms: 1000,
        minimum_expected_profit_cents: 100,
        minimum_margin_bps: 1000,
        maximum_cost_per_mission_cents: 2000,
        maximum_daily_cost_cents: 3000,
        stop_loss_cents: 5000,
        max_concurrency: 2,
        available_slots: 2,
    };
    let outcome = schedule(
        &[opportunity],
        &policy,
        &IncomeReport::default(),
        0,
        0,
        0,
        &[],
    );
    println!(
        "tma-economic status=ok selected={} expected_profit_cents={} fingerprint={}",
        outcome.selected.len(),
        outcome
            .selected
            .first()
            .map_or(0, |x| x.expected_profit_cents),
        outcome.fingerprint
    );
}
