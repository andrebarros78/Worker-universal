use std::{
    env,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tma_availability::{run_claim_matrix, verify_report};

fn temp_database(index: usize) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "tma-f11-soak-{}-{nonce}-{index}.sqlite3",
        std::process::id()
    ))
}
fn cleanup(db: &Path) {
    for path in [
        db.to_path_buf(),
        PathBuf::from(format!("{}-wal", db.display())),
        PathBuf::from(format!("{}-shm", db.display())),
    ] {
        let _ = std::fs::remove_file(path);
    }
}
fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() == 3 && args[0] == "matrix" {
        let workers = args[2].parse::<usize>().expect("workers");
        match run_claim_matrix(Path::new(&args[1]), workers, 64) {
            Ok(report) => {
                println!(
                    "F11_MATRIX workers={} missions={} unique={} duplicates={} lost={} errors={} events={} elapsed_ms={} integrity={}",
                    report.workers,
                    report.missions,
                    report.unique_claims,
                    report.duplicates,
                    report.lost,
                    report.errors.len(),
                    report.ledger_events,
                    report.elapsed_ms,
                    report.integrity
                );
                if let Err(e) = verify_report(&report) {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        return;
    }
    if args.len() == 2 && args[0] == "soak" {
        let seconds = args[1].parse::<u64>().expect("seconds");
        if seconds == 0 || seconds > 1800 {
            eprintln!("invalid soak duration");
            std::process::exit(2);
        }
        let started = Instant::now();
        let mut cycles = 0usize;
        let mut total_missions = 0usize;
        let mut workers_observed = [false; 4];
        while started.elapsed() < Duration::from_secs(seconds) {
            let workers = [1, 4, 8, 16][cycles % 4];
            let db = temp_database(cycles);
            let result = run_claim_matrix(&db, workers, 32);
            cleanup(&db);
            match result.and_then(|report| {
                verify_report(&report)?;
                Ok(report)
            }) {
                Ok(report) => {
                    workers_observed[cycles % 4] = true;
                    total_missions += report.missions;
                    cycles += 1;
                }
                Err(e) => {
                    eprintln!("F11_SOAK=FAIL cycle={cycles} workers={workers} error={e}");
                    std::process::exit(1);
                }
            }
        }
        println!(
            "F11_SOAK=PASS seconds={} cycles={} missions={} tiers_covered={} integrity=ok",
            started.elapsed().as_secs(),
            cycles,
            total_missions,
            workers_observed.iter().filter(|&&v| v).count()
        );
        if workers_observed.iter().any(|&v| !v) {
            std::process::exit(1);
        }
        return;
    }
    eprintln!("usage: tma-availability matrix <db> <workers> | soak <seconds>");
    std::process::exit(2);
}
