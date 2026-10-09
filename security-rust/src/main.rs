use std::{env, path::Path, process};
use tma_core::durable::DurableStore;

fn verify(path: &str) -> Result<(), String> {
    let restored = Path::new(path);
    if !restored.is_file() {
        return Err("F12_RESTORED_DATABASE_MISSING".to_string());
    }
    let bytes = restored.metadata().map_err(|e| e.to_string())?.len();
    if bytes < 4096 {
        return Err("F12_RESTORED_DATABASE_EMPTY".to_string());
    }
    let store = DurableStore::open(restored).map_err(|e| e.to_string())?;
    let integrity = store.integrity_check().map_err(|e| e.to_string())?;
    if integrity != "ok" {
        return Err("F12_SQLITE_INTEGRITY_FAILED".into());
    }
    let ledger = store.verify_ledger().map_err(|e| e.to_string())?;
    let queue = store.queue_depth().map_err(|e| e.to_string())?;
    if ledger.events == 0 || ledger.snapshots == 0 || queue == 0 {
        return Err("F12_RESTORED_F05_MISSION_EVIDENCE_EMPTY".to_string());
    }
    let p = store.pragmas().map_err(|e| e.to_string())?;
    if p.journal_mode != "wal" || p.foreign_keys != 1 {
        return Err("F12_SQLITE_PRAGMAS_UNSAFE".into());
    }
    println!(
        "F12_RESTORED_F05_VERIFY=PASS events={} snapshots={} queue={} sqlite=ok wal=on foreign_keys=on",
        ledger.events, ledger.snapshots, queue
    );
    Ok(())
}
fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 || args[0] != "verify" {
        eprintln!("usage: tma-security-verify verify <restored-f05-sqlite>");
        process::exit(2);
    }
    if let Err(e) = verify(&args[1]) {
        eprintln!("F12_RESTORED_F05_VERIFY=FAIL reason={e}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::verify;
    use std::{env, fs};
    use tma_core::durable::DurableStore;

    #[test]
    fn missing_snapshot_does_not_get_autocreated() {
        let path = env::temp_dir().join(format!(
            "f12-missing-{}-snapshot.sqlite3",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        assert!(verify(path.to_str().unwrap()).is_err());
        assert!(!path.exists());
    }

    #[test]
    fn valid_but_empty_f05_database_is_not_a_restored_mission() {
        let path =
            env::temp_dir().join(format!("f12-empty-{}-snapshot.sqlite3", std::process::id()));
        let _ = fs::remove_file(&path);
        let source = DurableStore::open(&path).unwrap();
        drop(source);
        assert!(verify(path.to_str().unwrap()).is_err());
        for suffix in ["", "-wal", "-shm"] {
            let _ = fs::remove_file(format!("{}{}", path.display(), suffix));
        }
    }
}
