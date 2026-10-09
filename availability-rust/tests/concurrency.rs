use std::path::PathBuf;
use tma_availability::{run_claim_matrix, verify_report};
fn temp_path(w: usize) -> PathBuf {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("f11-matrix-{w}-{}-{n}.db", std::process::id()))
}
fn probe(workers: usize) {
    let path = temp_path(workers);
    let result = run_claim_matrix(&path, workers, 48).unwrap();
    verify_report(&result).unwrap();
    assert_eq!(result.duplicates, 0);
    assert_eq!(result.lost, 0);
    assert_eq!(result.unique_claims, 48);
}
#[test]
fn matrix_1() {
    probe(1)
}
#[test]
fn matrix_4() {
    probe(4)
}
#[test]
fn matrix_8() {
    probe(8)
}
#[test]
fn matrix_16() {
    probe(16)
}
