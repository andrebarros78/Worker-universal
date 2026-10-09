use std::{env, fs};
use tma_production::{production_gate_denies_unqualified, run_local_rehearsal};
fn test_dir(s: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let p = env::temp_dir().join(format!("f13-{s}-{}-{nanos}", std::process::id()));
    fs::create_dir(&p).unwrap();
    p
}
#[test]
fn local_end_to_end_f05_f06_f07_f10_recovers_crash() {
    let root = test_dir("rehearsal");
    let result = run_local_rehearsal(&root, b"aBc\nnormalization\n", 1000).unwrap();
    assert!(result.ledger_events > 5);
    assert!(result.snapshots >= 3);
    assert_eq!(result.accrued_revenue_cents, 0);
    assert_eq!(result.payment_cents, 0);
    assert_eq!(result.cost_cents, 1);
    assert_eq!(result.provenance, "synthetic_local");
    let _ = fs::remove_dir_all(root);
}
#[test]
fn external_platform_unqualified_must_be_denied() {
    production_gate_denies_unqualified().unwrap();
}
#[test]
fn disallow_non_ascii_and_empty() {
    let root = test_dir("input-deny");
    assert!(run_local_rehearsal(&root, b"", 1000).is_err());
    assert!(run_local_rehearsal(&root, &[0xff], 1000).is_err());
    let _ = fs::remove_dir_all(root);
}
