use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use tma_core::MissionState;
use tma_core::durable::{ClaimOutcome, DurableStore, MissionSpec};

fn spec() -> MissionSpec {
    MissionSpec {
        mission_id: "probe-mission".to_string(),
        task_id: "probe-task".to_string(),
        deadline_unix_ms: 100_000,
        priority: 10,
        idempotency_key: "probe-idempotency".to_string(),
        payload_hash: "probe-payload".to_string(),
    }
}

fn remove_db(path: &str) {
    for candidate in [
        path.to_string(),
        format!("{path}-wal"),
        format!("{path}-shm"),
    ] {
        let _ = fs::remove_file(candidate);
    }
}

fn hold(db: &str, state: &str, ready: &str) -> Result<(), String> {
    remove_db(db);
    let store = DurableStore::open(db).map_err(|error| error.to_string())?;
    store
        .create_mission(&spec(), 1_000)
        .map_err(|error| error.to_string())?;

    let mut token = 0_i64;
    match state {
        "planned" => {}
        "running" | "validating" => {
            let grant = match store
                .claim_next("worker-old", 1_100, 100)
                .map_err(|error| error.to_string())?
            {
                ClaimOutcome::Claimed(grant) => grant,
                ClaimOutcome::Empty => return Err("expected lease".to_string()),
            };
            token = grant.fencing_token;
            store
                .save_checkpoint(
                    "probe-mission",
                    "worker-old",
                    token,
                    &format!("checkpoint={state}"),
                    1_110,
                )
                .map_err(|error| error.to_string())?;
            if state == "validating" {
                store
                    .transition_with_lease(
                        "probe-mission",
                        "worker-old",
                        token,
                        MissionState::Validating,
                        1_120,
                    )
                    .map_err(|error| error.to_string())?;
            }
        }
        other => return Err(format!("unsupported state: {other}")),
    }

    if let Some(parent) = Path::new(ready).parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(
        ready,
        format!(
            "READY state={state} token={token} pid={}\n",
            std::process::id()
        ),
    )
    .map_err(|error| error.to_string())?;

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

fn recover(db: &str, expected: &str) -> Result<(), String> {
    let store = DurableStore::open(db).map_err(|error| error.to_string())?;
    let report = store.reconcile(5_000).map_err(|error| error.to_string())?;
    let grant = match store
        .claim_next("worker-new", 5_001, 1_000)
        .map_err(|error| error.to_string())?
    {
        ClaimOutcome::Claimed(grant) => grant,
        ClaimOutcome::Empty => return Err("recovery queue empty".to_string()),
    };

    let expected_state = match expected {
        "planned" | "running" => MissionState::Running,
        "validating" => MissionState::Validating,
        other => return Err(format!("unsupported expected state: {other}")),
    };
    if grant.state != expected_state {
        return Err(format!(
            "state mismatch: expected {expected_state:?}, got {:?}",
            grant.state
        ));
    }

    if expected != "planned" && grant.checkpoint != format!("checkpoint={expected}") {
        return Err(format!(
            "checkpoint mismatch: expected checkpoint={expected}, got {}",
            grant.checkpoint
        ));
    }

    let ledger = store.verify_ledger().map_err(|error| error.to_string())?;
    let integrity = store.integrity_check().map_err(|error| error.to_string())?;
    if integrity != "ok" {
        return Err(format!("sqlite integrity failure: {integrity}"));
    }

    println!(
        "RECOVERED state={:?} token={} checkpoint={} expired_leases={} events={} snapshots={} integrity={}",
        grant.state,
        grant.fencing_token,
        grant.checkpoint,
        report.expired_leases,
        ledger.events,
        ledger.snapshots,
        integrity
    );
    Ok(())
}

fn inspect(db: &str) -> Result<(), String> {
    let store = DurableStore::open(db).map_err(|error| error.to_string())?;
    let mission = store
        .mission("probe-mission")
        .map_err(|error| error.to_string())?;
    let ledger = store.verify_ledger().map_err(|error| error.to_string())?;
    println!(
        "INSPECT state={:?} checkpoint={} fence={} events={} snapshots={} queue={}",
        mission.state,
        mission.checkpoint,
        mission.fence_seq,
        ledger.events,
        ledger.snapshots,
        store.queue_depth().map_err(|error| error.to_string())?
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("hold") if args.len() == 5 => hold(&args[2], &args[3], &args[4]),
        Some("recover") if args.len() == 4 => recover(&args[2], &args[3]),
        Some("inspect") if args.len() == 3 => inspect(&args[2]),
        _ => Err(
            "usage: tma-durable-probe hold <db> <planned|running|validating> <ready> | recover <db> <state> | inspect <db>"
                .to_string(),
        ),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
