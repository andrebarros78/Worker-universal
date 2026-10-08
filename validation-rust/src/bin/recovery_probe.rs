use std::fs;
use std::thread;
use std::time::Duration;

use tma_core::durable::{ClaimOutcome, DurableStore, MissionSpec};
use tma_foundation::model::{ErrorClass, SideEffectClass};
use tma_validation::{
    RecoveryAgent, RecoveryContext, persist_recovery_checkpoint, recover_recovery_checkpoint,
};

fn remove_db(path: &str) {
    for candidate in [
        path.to_string(),
        format!("{path}-wal"),
        format!("{path}-shm"),
    ] {
        let _ = fs::remove_file(candidate);
    }
}

fn hold(db: &str, ready: &str) -> Result<(), String> {
    remove_db(db);
    let store = DurableStore::open(db).map_err(|error| error.to_string())?;
    store
        .create_mission(
            &MissionSpec {
                mission_id: "recovery-probe".to_string(),
                task_id: "recovery-task".to_string(),
                deadline_unix_ms: 50_000,
                priority: 10,
                idempotency_key: "recovery-probe-idem".to_string(),
                payload_hash: "recovery-probe-payload".to_string(),
            },
            100,
        )
        .map_err(|error| error.to_string())?;
    let grant = match store
        .claim_next("worker-old", 200, 100)
        .map_err(|error| error.to_string())?
    {
        ClaimOutcome::Claimed(grant) => grant,
        ClaimOutcome::Empty => return Err("expected lease".to_string()),
    };
    let decision = RecoveryAgent.decide(&RecoveryContext {
        error_class: ErrorClass::WorkerCrash,
        attempt: 0,
        max_attempts: 3,
        remaining_ms: 10_000,
        minimum_retry_window_ms: 100,
        replay_safe: true,
        side_effect: SideEffectClass::Read,
        current_capability_id: "worker.local".to_string(),
        fallback_chain: Vec::new(),
    });
    let checkpoint = persist_recovery_checkpoint(
        &store,
        "recovery-probe",
        "worker-old",
        grant.fencing_token,
        &decision,
        220,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        ready,
        format!(
            "READY pid={} token={} checkpoint={}\n",
            std::process::id(),
            grant.fencing_token,
            checkpoint
        ),
    )
    .map_err(|error| error.to_string())?;
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

fn recover(db: &str) -> Result<(), String> {
    let store = DurableStore::open(db).map_err(|error| error.to_string())?;
    let report = store.reconcile(500).map_err(|error| error.to_string())?;
    let checkpoint =
        recover_recovery_checkpoint(&store, "recovery-probe").map_err(|error| error.to_string())?;
    let grant = match store
        .claim_next("worker-new", 501, 500)
        .map_err(|error| error.to_string())?
    {
        ClaimOutcome::Claimed(grant) => grant,
        ClaimOutcome::Empty => return Err("recovery queue empty".to_string()),
    };
    if grant.checkpoint != checkpoint {
        return Err("checkpoint mismatch after process restart".to_string());
    }
    if grant.fencing_token < 2 {
        return Err("fencing token did not advance".to_string());
    }
    if store.integrity_check().map_err(|error| error.to_string())? != "ok" {
        return Err("sqlite integrity failed".to_string());
    }
    store.verify_ledger().map_err(|error| error.to_string())?;
    println!(
        "F07_RECOVERY_RESTART=PASS expired_leases={} token={} checkpoint={}",
        report.expired_leases, grant.fencing_token, checkpoint
    );
    Ok(())
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let result = match args.as_slice() {
        [_, command, db, ready] if command == "hold" => hold(db, ready),
        [_, command, db] if command == "recover" => recover(db),
        _ => {
            Err("usage: tma-validation-recovery-probe hold <db> <ready> | recover <db>".to_string())
        }
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
