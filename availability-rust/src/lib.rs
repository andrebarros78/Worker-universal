use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tma_core::MissionState;
use tma_core::durable::{ClaimOutcome, DurableStore, MissionSpec};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcurrencyReport {
    pub workers: usize,
    pub missions: usize,
    pub unique_claims: usize,
    pub duplicates: usize,
    pub lost: usize,
    pub errors: Vec<String>,
    pub ledger_events: usize,
    pub elapsed_ms: u128,
    pub integrity: String,
}

pub fn run_claim_matrix(
    db: &std::path::Path,
    workers: usize,
    missions: usize,
) -> Result<ConcurrencyReport, String> {
    if workers == 0 || workers > 16 || missions == 0 || missions > 1024 {
        return Err("unsupported workload".into());
    }
    let store = DurableStore::open(db).map_err(|e| e.to_string())?;
    let start = Instant::now();
    for i in 0..missions {
        store
            .create_mission(
                &MissionSpec {
                    mission_id: format!("f11-{i:04}"),
                    task_id: format!("task-{i:04}"),
                    deadline_unix_ms: 1_000_000,
                    priority: (i % 5) as i64,
                    idempotency_key: format!("idem-f11-{i:04}"),
                    payload_hash: format!("payload-{i:04}"),
                },
                100,
            )
            .map_err(|e| e.to_string())?;
    }
    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let errors = Arc::new(Mutex::new(Vec::<String>::new()));
    let mut handles = Vec::new();
    for w in 0..workers {
        let db = db.to_path_buf();
        let seen = Arc::clone(&seen);
        let errors = Arc::clone(&errors);
        handles.push(thread::spawn(move || {
            let store = match DurableStore::open(db) {
                Ok(s) => s,
                Err(e) => {
                    errors.lock().unwrap().push(e.to_string());
                    return;
                }
            };
            let mut empty = 0;
            while empty < 2 {
                match store.claim_next(&format!("worker-{w}"), 1000, 50000) {
                    Ok(ClaimOutcome::Claimed(grant)) => {
                        empty = 0;
                        if let Err(e) = store.save_checkpoint(
                            &grant.mission_id,
                            &grant.worker_id,
                            grant.fencing_token,
                            &format!("f11_checkpoint_worker_{w}"),
                            1001,
                        ) {
                            errors.lock().unwrap().push(format!("checkpoint: {e}"));
                        }
                        seen.lock().unwrap().push(grant.mission_id);
                    }
                    Ok(ClaimOutcome::Empty) => {
                        empty += 1;
                        thread::sleep(Duration::from_millis(2));
                    }
                    Err(e) => {
                        errors.lock().unwrap().push(format!("claim: {e}"));
                        break;
                    }
                }
            }
        }));
    }
    for handle in handles {
        handle.join().map_err(|_| "worker panicked".to_string())?;
    }
    let seen = seen.lock().unwrap().clone();
    let unique = seen.iter().cloned().collect::<BTreeSet<_>>();
    let mut report = ConcurrencyReport {
        workers,
        missions,
        unique_claims: unique.len(),
        duplicates: seen.len().saturating_sub(unique.len()),
        lost: missions.saturating_sub(unique.len()),
        errors: errors.lock().unwrap().clone(),
        ledger_events: 0,
        elapsed_ms: start.elapsed().as_millis(),
        integrity: String::new(),
    };
    let verified = store.verify_ledger().map_err(|e| e.to_string())?;
    report.ledger_events = verified.events;
    report.integrity = store.integrity_check().map_err(|e| e.to_string())?;
    let q = store.queue_depth().map_err(|e| e.to_string())?;
    for id in &unique {
        let mission = store.mission(id).map_err(|e| e.to_string())?;
        if mission.state != MissionState::Running || mission.fence_seq != 1 {
            report.errors.push(format!("invalid lease projection {id}"));
        }
    }
    if q != missions {
        report.errors.push(format!("unexpected queue depth {q}"));
    }
    Ok(report)
}

pub fn verify_report(report: &ConcurrencyReport) -> Result<(), String> {
    if !report.errors.is_empty()
        || report.duplicates != 0
        || report.lost != 0
        || report.unique_claims != report.missions
        || report.integrity != "ok"
    {
        return Err(format!("F11_MATRIX_FAIL {report:?}"));
    }
    Ok(())
}
