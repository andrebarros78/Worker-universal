use std::{env, fs, path::PathBuf, process};
use tma_production::{production_gate_denies_unqualified, run_local_rehearsal};
fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 3 || args[0] != "rehearsal" {
        eprintln!("usage: tma-production rehearsal <new-rehearsal-directory> <local-input-file>");
        process::exit(2);
    }
    let base = PathBuf::from(&args[1]);
    if base.exists() {
        eprintln!("F13_REHEARSAL=DENIED reason=TARGET_EXISTS");
        process::exit(3);
    }
    let input = match fs::read(&args[2]) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("F13_REHEARSAL=DENIED reason=INPUT_UNAVAILABLE");
            process::exit(3)
        }
    };
    if let Err(e) = fs::create_dir(&base) {
        eprintln!("F13_REHEARSAL=DENIED reason={e}");
        process::exit(3)
    }
    if let Err(e) = production_gate_denies_unqualified() {
        eprintln!("F13_PRODUCTION_GATE=FAIL {e}");
        process::exit(1);
    }
    match run_local_rehearsal(&base, &input, 1000) {
        Ok(r) => {
            println!(
                "F13_LOCAL_E2E=PASS mission={} provenance={} ledger_events={} snapshots={} cost_cents={} recorded_revenue_cents={} payment_cents={} evidence_sha256={} artifact=result.txt",
                r.mission_id,
                r.provenance,
                r.ledger_events,
                r.snapshots,
                r.cost_cents,
                r.accrued_revenue_cents,
                r.payment_cents,
                r.terminal_hash
            );
            println!("F13_PRODUCTION_GATE=BLOCKED reason=EXTERNAL_ADAPTER_NOT_QUALIFIED");
            println!("F13_MISSION_PROVEN=NOT_YET");
        }
        Err(e) => {
            eprintln!("F13_LOCAL_E2E=FAIL reason={e}");
            process::exit(1);
        }
    }
}
