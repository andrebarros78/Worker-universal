use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tma_core::MissionState;
use tma_core::durable::DurableStore;

use crate::model::{EconomicEntry, EntryKind, IncomeReport, RecordOutcome};

pub struct EconomicLedger {
    path: PathBuf,
}

impl EconomicLedger {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let store = Self { path };
        let conn = store.connect()?;
        let current_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        if current_mode.to_lowercase() != "wal" {
            conn.execute_batch("PRAGMA journal_mode=WAL;")
                .map_err(|e| e.to_string())?;
        }
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS economic_events (
                 seq INTEGER PRIMARY KEY AUTOINCREMENT,
                 event_key TEXT NOT NULL UNIQUE,
                 mission_id TEXT NOT NULL,
                 kind TEXT NOT NULL CHECK(kind IN ('verified_earning','payment_received','cost')),
                 amount_cents INTEGER NOT NULL CHECK(amount_cents > 0),
                 source_ref TEXT NOT NULL,
                 recorded_at_ms INTEGER NOT NULL,
                 prev_hash TEXT NOT NULL,
                 event_hash TEXT NOT NULL
             );
             CREATE UNIQUE INDEX IF NOT EXISTS single_earning_per_mission
             ON economic_events(mission_id) WHERE kind='verified_earning';
             CREATE UNIQUE INDEX IF NOT EXISTS single_payment_per_settlement
             ON economic_events(source_ref) WHERE kind='payment_received';
             CREATE TRIGGER IF NOT EXISTS economic_events_no_update
             BEFORE UPDATE ON economic_events BEGIN
                 SELECT RAISE(ABORT,'economic_events_append_only');
             END;
             CREATE TRIGGER IF NOT EXISTS economic_events_no_delete
             BEFORE DELETE ON economic_events BEGIN
                 SELECT RAISE(ABORT,'economic_events_append_only');
             END;",
        )
        .map_err(|e| e.to_string())?;
        Ok(store)
    }

    fn connect(&self) -> Result<Connection, String> {
        let conn = Connection::open(&self.path).map_err(|e| e.to_string())?;
        conn.busy_timeout(Duration::from_secs(10))
            .map_err(|e| e.to_string())?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;",
        )
        .map_err(|e| e.to_string())?;
        Ok(conn)
    }

    pub fn record_verified_earning(
        &self,
        core: &DurableStore,
        entry: &EconomicEntry,
    ) -> Result<RecordOutcome, String> {
        if entry.kind != EntryKind::VerifiedEarning {
            return Err("verified earning API requires earning kind".to_string());
        }
        let mission = core.mission(&entry.mission_id).map_err(|e| e.to_string())?;
        if mission.state != MissionState::Succeeded {
            return Err("F05 mission is not independently validated Succeeded".to_string());
        }
        if mission.terminal_result_hash.as_deref() != Some(entry.source_ref.as_str()) {
            return Err("earning source must bind to F05 terminal_result_hash".to_string());
        }
        self.record(entry)
    }

    pub fn record_cost(&self, entry: &EconomicEntry) -> Result<RecordOutcome, String> {
        if entry.kind != EntryKind::Cost {
            return Err("cost API requires cost kind".to_string());
        }
        self.record(entry)
    }

    pub fn record_payment(&self, entry: &EconomicEntry) -> Result<RecordOutcome, String> {
        if entry.kind != EntryKind::PaymentReceived {
            return Err("payment API requires payment kind".to_string());
        }
        if entry.source_ref.trim().is_empty() {
            return Err("payment requires external settlement reference".to_string());
        }
        self.record(entry)
    }

    fn record(&self, entry: &EconomicEntry) -> Result<RecordOutcome, String> {
        if entry.event_key.trim().is_empty()
            || entry.mission_id.trim().is_empty()
            || entry.source_ref.trim().is_empty()
            || entry.amount_cents <= 0
            || entry.recorded_at_ms < 0
        {
            return Err("invalid economic entry".to_string());
        }
        let mut conn = self.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|e| e.to_string())?;
        let existing = tx
            .query_row(
                "SELECT mission_id,kind,amount_cents,source_ref,recorded_at_ms
             FROM economic_events WHERE event_key=?1",
                params![entry.event_key],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some((mission, kind, amount, source, recorded)) = existing {
            if mission == entry.mission_id
                && kind == entry.kind.as_str()
                && amount == entry.amount_cents
                && source == entry.source_ref
                && recorded == entry.recorded_at_ms
            {
                tx.commit().map_err(|e| e.to_string())?;
                return Ok(RecordOutcome::Duplicate);
            }
            return Err("economic event_key conflict".to_string());
        }

        if entry.kind == EntryKind::PaymentReceived {
            let (earned, paid): (i64, i64) = tx
                .query_row(
                    "SELECT
                    COALESCE(SUM(CASE WHEN kind='verified_earning' THEN amount_cents ELSE 0 END),0),
                    COALESCE(SUM(CASE WHEN kind='payment_received' THEN amount_cents ELSE 0 END),0)
                 FROM economic_events WHERE mission_id=?1",
                    params![entry.mission_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(|e| e.to_string())?;
            let outstanding = i128::from(earned) - i128::from(paid);
            if outstanding < i128::from(entry.amount_cents) {
                return Err("payment would exceed verified earned receivable".to_string());
            }
        }

        // Revenue and costs are not the same event: cost spikes never inflate payment.
        let prev: String = tx
            .query_row(
                "SELECT event_hash FROM economic_events ORDER BY seq DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "GENESIS".to_string());
        let hash = entry_hash(&prev, entry);
        tx.execute(
            "INSERT INTO economic_events(
                event_key,mission_id,kind,amount_cents,source_ref,recorded_at_ms,prev_hash,event_hash
            ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![entry.event_key,entry.mission_id,entry.kind.as_str(),entry.amount_cents,
                entry.source_ref,entry.recorded_at_ms,prev,hash],
        ).map_err(|e|e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(RecordOutcome::Recorded)
    }

    pub fn verify_chain(&self) -> Result<usize, String> {
        let conn = self.connect()?;
        let mut stmt=conn.prepare(
            "SELECT event_key,mission_id,kind,amount_cents,source_ref,recorded_at_ms,prev_hash,event_hash
             FROM economic_events ORDER BY seq"
        ).map_err(|e|e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut expected_prev = "GENESIS".to_string();
        let mut count = 0usize;
        for value in rows {
            let (event_key, mission_id, kind, amount, source, recorded, prev, hash) =
                value.map_err(|e| e.to_string())?;
            if prev != expected_prev {
                return Err(format!(
                    "economic hash chain previous mismatch at {event_key}"
                ));
            }
            let kind = match kind.as_str() {
                "verified_earning" => EntryKind::VerifiedEarning,
                "payment_received" => EntryKind::PaymentReceived,
                "cost" => EntryKind::Cost,
                _ => return Err("economic kind corruption".to_string()),
            };
            let expected = entry_hash(
                &prev,
                &EconomicEntry {
                    event_key,
                    mission_id,
                    kind,
                    amount_cents: amount,
                    source_ref: source,
                    recorded_at_ms: recorded,
                },
            );
            if expected != hash {
                return Err("economic hash chain tampered".to_string());
            }
            expected_prev = hash;
            count += 1;
        }
        Ok(count)
    }

    pub fn report(&self) -> Result<IncomeReport, String> {
        let conn = self.connect()?;
        let (earned, paid, cost, count): (i64, i64, i64, i64) = conn
            .query_row(
                "SELECT
                COALESCE(SUM(CASE WHEN kind='verified_earning' THEN amount_cents ELSE 0 END),0),
                COALESCE(SUM(CASE WHEN kind='payment_received' THEN amount_cents ELSE 0 END),0),
                COALESCE(SUM(CASE WHEN kind='cost' THEN amount_cents ELSE 0 END),0),
                COUNT(*)
             FROM economic_events",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|e| e.to_string())?;
        fn safe(v: i128) -> Result<i64, String> {
            i64::try_from(v).map_err(|_| "accounting overflow".to_string())
        }
        Ok(IncomeReport {
            verified_earnings_cents: earned,
            received_payments_cents: paid,
            costs_cents: cost,
            accrued_profit_cents: safe(i128::from(earned) - i128::from(cost))?,
            cash_profit_cents: safe(i128::from(paid) - i128::from(cost))?,
            accounts_receivable_cents: safe(i128::from(earned) - i128::from(paid))?,
            entries: usize::try_from(count).map_err(|_| "accounting entry overflow".to_string())?,
        })
    }

    pub fn costs_since(&self, since_ms: i64) -> Result<i64, String> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT COALESCE(SUM(amount_cents),0) FROM economic_events
             WHERE kind='cost' AND recorded_at_ms>=?1",
            params![since_ms],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())
    }

    pub fn integrity_check(&self) -> Result<String, String> {
        let conn = self.connect()?;
        conn.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())
    }
}

fn entry_hash(prev: &str, entry: &EconomicEntry) -> String {
    let canonical = format!(
        "{prev:?}|{:?}|{:?}|{}|{}|{:?}|{}",
        entry.event_key,
        entry.mission_id,
        entry.kind.as_str(),
        entry.amount_cents,
        entry.recorded_at_ms,
        entry.source_ref
    );
    let digest = Sha256::digest(canonical.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
