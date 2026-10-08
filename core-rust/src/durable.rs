use crate::{MissionState, TransitionError, allowed};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub type DurableResult<T> = Result<T, DurableError>;

#[derive(Debug)]
pub enum DurableError {
    Sql(rusqlite::Error),
    InvalidInput(String),
    InvalidState(String),
    MissionNotFound(String),
    IdempotencyConflict(String),
    LeaseMissing(String),
    WrongOwner,
    StaleFence { expected: i64, actual: i64 },
    LeaseExpired,
    ResultIdConflict(String),
    ValidationReceiptConflict(String),
    IllegalTransition(TransitionError),
    LedgerCorrupt(String),
}

impl fmt::Display for DurableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sql(error) => write!(f, "sqlite error: {error}"),
            Self::InvalidInput(message) => write!(f, "invalid input: {message}"),
            Self::InvalidState(state) => write!(f, "invalid persisted mission state: {state}"),
            Self::MissionNotFound(id) => write!(f, "mission not found: {id}"),
            Self::IdempotencyConflict(key) => write!(f, "idempotency conflict: {key}"),
            Self::LeaseMissing(id) => write!(f, "lease missing: {id}"),
            Self::WrongOwner => write!(f, "lease owner mismatch"),
            Self::StaleFence { expected, actual } => {
                write!(f, "stale fencing token: expected {expected}, got {actual}")
            }
            Self::LeaseExpired => write!(f, "lease expired"),
            Self::ResultIdConflict(id) => write!(f, "result id conflict: {id}"),
            Self::ValidationReceiptConflict(id) => {
                write!(f, "validation receipt conflict: {id}")
            }
            Self::IllegalTransition(error) => write!(f, "{error}"),
            Self::LedgerCorrupt(message) => write!(f, "ledger corrupt: {message}"),
        }
    }
}

impl std::error::Error for DurableError {}

impl From<rusqlite::Error> for DurableError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sql(value)
    }
}

impl From<TransitionError> for DurableError {
    fn from(value: TransitionError) -> Self {
        Self::IllegalTransition(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionSpec {
    pub mission_id: String,
    pub task_id: String,
    pub deadline_unix_ms: i64,
    pub priority: i64,
    pub idempotency_key: String,
    pub payload_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionRecord {
    pub mission_id: String,
    pub task_id: String,
    pub state: MissionState,
    pub deadline_unix_ms: i64,
    pub priority: i64,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub checkpoint: String,
    pub fence_seq: i64,
    pub terminal_result_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateOutcome {
    Created(String),
    Existing(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseGrant {
    pub mission_id: String,
    pub task_id: String,
    pub state: MissionState,
    pub checkpoint: String,
    pub worker_id: String,
    pub fencing_token: i64,
    pub lease_until_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimOutcome {
    Claimed(LeaseGrant),
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultSubmission {
    pub result_id: String,
    pub mission_id: String,
    pub worker_id: String,
    pub fencing_token: i64,
    pub target_state: MissionState,
    pub payload_hash: String,
    pub validation_receipt_id: Option<String>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReceipt {
    pub receipt_id: String,
    pub mission_id: String,
    pub validator_id: String,
    pub subject_payload_hash: String,
    pub evidence_digest: String,
    pub accepted: bool,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultDisposition {
    Accepted,
    Duplicate {
        accepted: bool,
        reason: Option<String>,
    },
    Rejected {
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReconcileReport {
    pub expired_leases: usize,
    pub requeued: usize,
    pub terminal_queue_removed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LedgerReport {
    pub missions: usize,
    pub events: usize,
    pub snapshots: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PragmaReport {
    pub journal_mode: String,
    pub synchronous: i64,
    pub foreign_keys: i64,
}

#[derive(Debug, Clone)]
pub struct DurableStore {
    path: PathBuf,
}

impl DurableStore {
    pub fn open(path: impl AsRef<Path>) -> DurableResult<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                DurableError::InvalidInput(format!("cannot create database directory: {error}"))
            })?;
        }
        let store = Self { path };
        store.initialize()?;
        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn connect(&self) -> DurableResult<Connection> {
        let conn = Connection::open(&self.path)?;
        conn.busy_timeout(Duration::from_secs(10))?;
        conn.execute_batch(
            "PRAGMA foreign_keys=ON;
             PRAGMA journal_mode=WAL;
             PRAGMA synchronous=FULL;",
        )?;
        Ok(conn)
    }

    fn initialize(&self) -> DurableResult<()> {
        let conn = self.connect()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS missions (
                mission_id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                state TEXT NOT NULL,
                deadline_unix_ms INTEGER NOT NULL,
                priority INTEGER NOT NULL,
                idempotency_key TEXT NOT NULL UNIQUE,
                payload_hash TEXT NOT NULL,
                checkpoint TEXT NOT NULL DEFAULT '',
                fence_seq INTEGER NOT NULL DEFAULT 0,
                terminal_result_hash TEXT,
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS queue (
                mission_id TEXT PRIMARY KEY REFERENCES missions(mission_id) ON DELETE CASCADE,
                available_at_ms INTEGER NOT NULL,
                enqueued_at_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS leases (
                mission_id TEXT PRIMARY KEY REFERENCES missions(mission_id) ON DELETE CASCADE,
                owner TEXT NOT NULL,
                fencing_token INTEGER NOT NULL,
                lease_until_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS mission_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mission_id TEXT NOT NULL REFERENCES missions(mission_id) ON DELETE RESTRICT,
                ordinal INTEGER NOT NULL,
                kind TEXT NOT NULL,
                payload TEXT NOT NULL,
                prev_hash TEXT NOT NULL,
                event_hash TEXT NOT NULL UNIQUE,
                created_at_ms INTEGER NOT NULL,
                UNIQUE(mission_id, ordinal)
            );

            CREATE TABLE IF NOT EXISTS mission_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mission_id TEXT NOT NULL REFERENCES missions(mission_id) ON DELETE RESTRICT,
                state TEXT NOT NULL,
                checkpoint TEXT NOT NULL,
                event_id INTEGER NOT NULL REFERENCES mission_events(id) ON DELETE RESTRICT,
                created_at_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS validation_receipts (
                receipt_id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL REFERENCES missions(mission_id) ON DELETE RESTRICT,
                validator_id TEXT NOT NULL,
                subject_payload_hash TEXT NOT NULL,
                evidence_digest TEXT NOT NULL,
                accepted INTEGER NOT NULL CHECK(accepted IN (0,1)),
                created_at_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS results (
                result_id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL REFERENCES missions(mission_id) ON DELETE RESTRICT,
                worker_id TEXT NOT NULL,
                fencing_token INTEGER NOT NULL,
                target_state TEXT NOT NULL,
                payload_hash TEXT NOT NULL,
                accepted INTEGER NOT NULL CHECK(accepted IN (0,1)),
                reason TEXT,
                created_at_ms INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_queue_available
                ON queue(available_at_ms, enqueued_at_ms);
            CREATE INDEX IF NOT EXISTS idx_events_mission
                ON mission_events(mission_id, ordinal);
            CREATE INDEX IF NOT EXISTS idx_snapshots_mission
                ON mission_snapshots(mission_id, id);

            CREATE TRIGGER IF NOT EXISTS mission_events_no_update
            BEFORE UPDATE ON mission_events
            BEGIN
                SELECT RAISE(ABORT, 'mission_events_append_only');
            END;

            CREATE TRIGGER IF NOT EXISTS mission_events_no_delete
            BEFORE DELETE ON mission_events
            BEGIN
                SELECT RAISE(ABORT, 'mission_events_append_only');
            END;

            CREATE TRIGGER IF NOT EXISTS mission_snapshots_no_update
            BEFORE UPDATE ON mission_snapshots
            BEGIN
                SELECT RAISE(ABORT, 'mission_snapshots_append_only');
            END;

            CREATE TRIGGER IF NOT EXISTS mission_snapshots_no_delete
            BEFORE DELETE ON mission_snapshots
            BEGIN
                SELECT RAISE(ABORT, 'mission_snapshots_append_only');
            END;

            CREATE TRIGGER IF NOT EXISTS validation_receipts_no_update
            BEFORE UPDATE ON validation_receipts
            BEGIN
                SELECT RAISE(ABORT, 'validation_receipts_append_only');
            END;

            CREATE TRIGGER IF NOT EXISTS validation_receipts_no_delete
            BEFORE DELETE ON validation_receipts
            BEGIN
                SELECT RAISE(ABORT, 'validation_receipts_append_only');
            END;
            ",
        )?;
        Ok(())
    }

    pub fn pragmas(&self) -> DurableResult<PragmaReport> {
        let conn = self.connect()?;
        let journal_mode =
            conn.query_row("PRAGMA journal_mode", [], |row| row.get::<_, String>(0))?;
        let synchronous = conn.query_row("PRAGMA synchronous", [], |row| row.get::<_, i64>(0))?;
        let foreign_keys = conn.query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))?;
        Ok(PragmaReport {
            journal_mode,
            synchronous,
            foreign_keys,
        })
    }

    pub fn integrity_check(&self) -> DurableResult<String> {
        let conn = self.connect()?;
        Ok(conn.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))?)
    }

    pub fn create_mission(&self, spec: &MissionSpec, now_ms: i64) -> DurableResult<CreateOutcome> {
        validate_spec(spec)?;
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

        let existing = tx
            .query_row(
                "SELECT mission_id, task_id, deadline_unix_ms, priority, payload_hash
                 FROM missions WHERE idempotency_key=?1",
                params![spec.idempotency_key],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()?;

        if let Some((mission_id, task_id, deadline, priority, payload_hash)) = existing {
            if mission_id == spec.mission_id
                && task_id == spec.task_id
                && deadline == spec.deadline_unix_ms
                && priority == spec.priority
                && payload_hash == spec.payload_hash
            {
                tx.commit()?;
                return Ok(CreateOutcome::Existing(mission_id));
            }
            return Err(DurableError::IdempotencyConflict(
                spec.idempotency_key.clone(),
            ));
        }

        tx.execute(
            "INSERT INTO missions(
                mission_id, task_id, state, deadline_unix_ms, priority,
                idempotency_key, payload_hash, checkpoint, fence_seq,
                created_at_ms, updated_at_ms
             ) VALUES (?1,?2,'received',?3,?4,?5,?6,'',0,?7,?7)",
            params![
                spec.mission_id,
                spec.task_id,
                spec.deadline_unix_ms,
                spec.priority,
                spec.idempotency_key,
                spec.payload_hash,
                now_ms
            ],
        )?;

        let created_event = append_event_tx(
            &tx,
            &spec.mission_id,
            "mission_created",
            &format!(
                "task_id={};payload_hash={}",
                spec.task_id, spec.payload_hash
            ),
            now_ms,
        )?;
        append_snapshot_tx(
            &tx,
            &spec.mission_id,
            MissionState::Received,
            "",
            created_event,
            now_ms,
        )?;

        transition_projection_tx(
            &tx,
            &spec.mission_id,
            MissionState::Received,
            MissionState::Planned,
            "",
            now_ms,
        )?;

        tx.execute(
            "INSERT INTO queue(mission_id, available_at_ms, enqueued_at_ms)
             VALUES (?1,?2,?2)",
            params![spec.mission_id, now_ms],
        )?;

        tx.commit()?;
        Ok(CreateOutcome::Created(spec.mission_id.clone()))
    }

    pub fn mission(&self, mission_id: &str) -> DurableResult<MissionRecord> {
        let conn = self.connect()?;
        mission_from_conn(&conn, mission_id)
    }

    pub fn queue_depth(&self) -> DurableResult<usize> {
        let conn = self.connect()?;
        let count = conn.query_row("SELECT COUNT(*) FROM queue", [], |row| row.get::<_, i64>(0))?;
        Ok(count as usize)
    }

    pub fn event_count(&self, mission_id: &str) -> DurableResult<usize> {
        let conn = self.connect()?;
        let count = conn.query_row(
            "SELECT COUNT(*) FROM mission_events WHERE mission_id=?1",
            params![mission_id],
            |row| row.get::<_, i64>(0),
        )?;
        Ok(count as usize)
    }

    pub fn snapshot_count(&self, mission_id: &str) -> DurableResult<usize> {
        let conn = self.connect()?;
        let count = conn.query_row(
            "SELECT COUNT(*) FROM mission_snapshots WHERE mission_id=?1",
            params![mission_id],
            |row| row.get::<_, i64>(0),
        )?;
        Ok(count as usize)
    }

    pub fn claim_next(
        &self,
        worker_id: &str,
        now_ms: i64,
        lease_ms: i64,
    ) -> DurableResult<ClaimOutcome> {
        if worker_id.trim().is_empty() || lease_ms <= 0 {
            return Err(DurableError::InvalidInput(
                "worker_id must be non-empty and lease_ms positive".to_string(),
            ));
        }

        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

        let candidate = tx
            .query_row(
                "SELECT m.mission_id, m.task_id, m.state, m.checkpoint, m.fence_seq
                 FROM queue q
                 JOIN missions m ON m.mission_id=q.mission_id
                 LEFT JOIN leases l ON l.mission_id=m.mission_id
                 WHERE q.available_at_ms<=?1
                   AND m.state NOT IN ('succeeded','failed','deadline_exceeded')
                   AND (l.mission_id IS NULL OR l.lease_until_ms<=?1)
                 ORDER BY m.priority DESC, q.enqueued_at_ms ASC, m.mission_id ASC
                 LIMIT 1",
                params![now_ms],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                },
            )
            .optional()?;

        let Some((mission_id, task_id, state_raw, checkpoint, fence_seq)) = candidate else {
            tx.commit()?;
            return Ok(ClaimOutcome::Empty);
        };

        let mut state = state_from_str(&state_raw)?;
        if state == MissionState::Received {
            return Err(DurableError::LedgerCorrupt(format!(
                "queued mission {mission_id} persisted in received state"
            )));
        }

        let token = fence_seq + 1;
        let lease_until_ms = now_ms
            .checked_add(lease_ms)
            .ok_or_else(|| DurableError::InvalidInput("lease overflow".to_string()))?;

        tx.execute(
            "UPDATE missions SET fence_seq=?2, updated_at_ms=?3 WHERE mission_id=?1",
            params![mission_id, token, now_ms],
        )?;

        tx.execute(
            "INSERT INTO leases(mission_id,owner,fencing_token,lease_until_ms)
             VALUES (?1,?2,?3,?4)
             ON CONFLICT(mission_id) DO UPDATE SET
               owner=excluded.owner,
               fencing_token=excluded.fencing_token,
               lease_until_ms=excluded.lease_until_ms",
            params![mission_id, worker_id, token, lease_until_ms],
        )?;

        if state == MissionState::Planned {
            transition_projection_tx(
                &tx,
                &mission_id,
                MissionState::Planned,
                MissionState::Running,
                &checkpoint,
                now_ms,
            )?;
            state = MissionState::Running;
        }

        append_event_tx(
            &tx,
            &mission_id,
            "lease_acquired",
            &format!("owner={worker_id};token={token};until={lease_until_ms}"),
            now_ms,
        )?;

        tx.commit()?;
        Ok(ClaimOutcome::Claimed(LeaseGrant {
            mission_id,
            task_id,
            state,
            checkpoint,
            worker_id: worker_id.to_string(),
            fencing_token: token,
            lease_until_ms,
        }))
    }

    pub fn renew_lease(
        &self,
        mission_id: &str,
        worker_id: &str,
        token: i64,
        now_ms: i64,
        lease_ms: i64,
    ) -> DurableResult<i64> {
        if lease_ms <= 0 {
            return Err(DurableError::InvalidInput(
                "lease_ms must be positive".to_string(),
            ));
        }
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_active_lease_tx(&tx, mission_id, worker_id, token, now_ms)?;
        let until = now_ms
            .checked_add(lease_ms)
            .ok_or_else(|| DurableError::InvalidInput("lease overflow".to_string()))?;
        tx.execute(
            "UPDATE leases SET lease_until_ms=?2 WHERE mission_id=?1",
            params![mission_id, until],
        )?;
        append_event_tx(
            &tx,
            mission_id,
            "lease_renewed",
            &format!("owner={worker_id};token={token};until={until}"),
            now_ms,
        )?;
        tx.commit()?;
        Ok(until)
    }

    pub fn save_checkpoint(
        &self,
        mission_id: &str,
        worker_id: &str,
        token: i64,
        checkpoint: &str,
        now_ms: i64,
    ) -> DurableResult<()> {
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_active_lease_tx(&tx, mission_id, worker_id, token, now_ms)?;
        let current = mission_from_tx(&tx, mission_id)?;
        tx.execute(
            "UPDATE missions SET checkpoint=?2, updated_at_ms=?3 WHERE mission_id=?1",
            params![mission_id, checkpoint, now_ms],
        )?;
        let event_id = append_event_tx(
            &tx,
            mission_id,
            "checkpoint_saved",
            &format!("bytes={}", checkpoint.len()),
            now_ms,
        )?;
        append_snapshot_tx(&tx, mission_id, current.state, checkpoint, event_id, now_ms)?;
        tx.commit()?;
        Ok(())
    }

    pub fn transition_with_lease(
        &self,
        mission_id: &str,
        worker_id: &str,
        token: i64,
        to: MissionState,
        now_ms: i64,
    ) -> DurableResult<MissionState> {
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_active_lease_tx(&tx, mission_id, worker_id, token, now_ms)?;
        let current = mission_from_tx(&tx, mission_id)?;
        transition_projection_tx(
            &tx,
            mission_id,
            current.state,
            to,
            &current.checkpoint,
            now_ms,
        )?;
        if to.is_terminal() {
            tx.execute("DELETE FROM queue WHERE mission_id=?1", params![mission_id])?;
            tx.execute(
                "DELETE FROM leases WHERE mission_id=?1",
                params![mission_id],
            )?;
        }
        tx.commit()?;
        Ok(to)
    }

    pub fn record_validation_receipt(&self, receipt: &ValidationReceipt) -> DurableResult<()> {
        if receipt.receipt_id.trim().is_empty()
            || receipt.mission_id.trim().is_empty()
            || receipt.validator_id.trim().is_empty()
            || receipt.subject_payload_hash.trim().is_empty()
            || receipt.evidence_digest.len() != 64
            || !receipt
                .evidence_digest
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            return Err(DurableError::InvalidInput(
                "invalid validation receipt".to_string(),
            ));
        }

        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = mission_from_tx(&tx, &receipt.mission_id)?;
        if current.state != MissionState::Validating {
            return Err(DurableError::InvalidInput(
                "validation receipt requires validating mission".to_string(),
            ));
        }

        let lease_owner = tx
            .query_row(
                "SELECT owner FROM leases WHERE mission_id=?1",
                params![receipt.mission_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or_else(|| DurableError::LeaseMissing(receipt.mission_id.clone()))?;

        if lease_owner == receipt.validator_id {
            return Err(DurableError::InvalidInput(
                "validator must be independent from lease owner".to_string(),
            ));
        }

        let existing = tx
            .query_row(
                "SELECT mission_id,validator_id,subject_payload_hash,evidence_digest,accepted
                 FROM validation_receipts WHERE receipt_id=?1",
                params![receipt.receipt_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                },
            )
            .optional()?;

        if let Some(existing) = existing {
            let same = existing.0 == receipt.mission_id
                && existing.1 == receipt.validator_id
                && existing.2 == receipt.subject_payload_hash
                && existing.3 == receipt.evidence_digest
                && existing.4 == i64::from(receipt.accepted);
            if same {
                tx.commit()?;
                return Ok(());
            }
            return Err(DurableError::ValidationReceiptConflict(
                receipt.receipt_id.clone(),
            ));
        }

        tx.execute(
            "INSERT INTO validation_receipts(
                receipt_id,mission_id,validator_id,subject_payload_hash,
                evidence_digest,accepted,created_at_ms
             ) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                receipt.receipt_id,
                receipt.mission_id,
                receipt.validator_id,
                receipt.subject_payload_hash,
                receipt.evidence_digest,
                i64::from(receipt.accepted),
                receipt.created_at_ms
            ],
        )?;
        append_event_tx(
            &tx,
            &receipt.mission_id,
            "validation_receipt_recorded",
            &format!(
                "receipt_id={};validator_id={};payload_hash={};evidence_digest={};accepted={}",
                receipt.receipt_id,
                receipt.validator_id,
                receipt.subject_payload_hash,
                receipt.evidence_digest,
                receipt.accepted
            ),
            receipt.created_at_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn submit_result(&self, submission: &ResultSubmission) -> DurableResult<ResultDisposition> {
        let result_id = submission.result_id.as_str();
        let mission_id = submission.mission_id.as_str();
        let worker_id = submission.worker_id.as_str();
        let token = submission.fencing_token;
        let target = submission.target_state;
        let payload_hash = submission.payload_hash.as_str();
        let now_ms = submission.created_at_ms;

        if result_id.trim().is_empty() || payload_hash.trim().is_empty() || !target.is_terminal() {
            return Err(DurableError::InvalidInput(
                "result_id/payload_hash required and target must be terminal".to_string(),
            ));
        }
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

        let existing = tx
            .query_row(
                "SELECT mission_id,target_state,payload_hash,accepted,reason
                 FROM results WHERE result_id=?1",
                params![result_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                },
            )
            .optional()?;

        if let Some((existing_mission, existing_target, existing_hash, accepted, reason)) = existing
        {
            if existing_mission != mission_id
                || existing_target != state_to_str(target)
                || existing_hash != payload_hash
            {
                return Err(DurableError::ResultIdConflict(result_id.to_string()));
            }
            tx.commit()?;
            return Ok(ResultDisposition::Duplicate {
                accepted: accepted == 1,
                reason,
            });
        }

        let lease_check = verify_active_lease_tx(&tx, mission_id, worker_id, token, now_ms);
        if let Err(error) = lease_check {
            let reason = lease_rejection_reason(&error).unwrap_or_else(|| error.to_string());
            tx.execute(
                "INSERT INTO results(
                    result_id,mission_id,worker_id,fencing_token,target_state,
                    payload_hash,accepted,reason,created_at_ms
                 ) VALUES (?1,?2,?3,?4,?5,?6,0,?7,?8)",
                params![
                    result_id,
                    mission_id,
                    worker_id,
                    token,
                    state_to_str(target),
                    payload_hash,
                    reason,
                    now_ms
                ],
            )?;
            append_event_tx(
                &tx,
                mission_id,
                "result_rejected",
                &format!("result_id={result_id};reason={reason};token={token}"),
                now_ms,
            )?;
            tx.commit()?;
            return Ok(ResultDisposition::Rejected { reason });
        }

        let current = mission_from_tx(&tx, mission_id)?;
        if !allowed(current.state, target) {
            let reason = "illegal_transition".to_string();
            tx.execute(
                "INSERT INTO results(
                    result_id,mission_id,worker_id,fencing_token,target_state,
                    payload_hash,accepted,reason,created_at_ms
                 ) VALUES (?1,?2,?3,?4,?5,?6,0,?7,?8)",
                params![
                    result_id,
                    mission_id,
                    worker_id,
                    token,
                    state_to_str(target),
                    payload_hash,
                    reason,
                    now_ms
                ],
            )?;
            append_event_tx(
                &tx,
                mission_id,
                "result_rejected",
                &format!("result_id={result_id};reason={reason};token={token}"),
                now_ms,
            )?;
            tx.commit()?;
            return Ok(ResultDisposition::Rejected { reason });
        }

        let validation_receipt_id = if target == MissionState::Succeeded {
            let Some(receipt_id) = submission.validation_receipt_id.as_deref() else {
                let reason = "validation_receipt_required".to_string();
                tx.execute(
                    "INSERT INTO results(
                        result_id,mission_id,worker_id,fencing_token,target_state,
                        payload_hash,accepted,reason,created_at_ms
                     ) VALUES (?1,?2,?3,?4,?5,?6,0,?7,?8)",
                    params![
                        result_id,
                        mission_id,
                        worker_id,
                        token,
                        state_to_str(target),
                        payload_hash,
                        reason,
                        now_ms
                    ],
                )?;
                append_event_tx(
                    &tx,
                    mission_id,
                    "result_rejected",
                    &format!("result_id={result_id};reason={reason};token={token}"),
                    now_ms,
                )?;
                tx.commit()?;
                return Ok(ResultDisposition::Rejected { reason });
            };

            let accepted = tx.query_row(
                "SELECT COUNT(*) FROM validation_receipts
                     WHERE receipt_id=?1
                       AND mission_id=?2
                       AND subject_payload_hash=?3
                       AND accepted=1",
                params![receipt_id, mission_id, payload_hash],
                |row| row.get::<_, i64>(0),
            )?;
            if accepted != 1 {
                let reason = "validation_receipt_invalid".to_string();
                tx.execute(
                    "INSERT INTO results(
                        result_id,mission_id,worker_id,fencing_token,target_state,
                        payload_hash,accepted,reason,created_at_ms
                     ) VALUES (?1,?2,?3,?4,?5,?6,0,?7,?8)",
                    params![
                        result_id,
                        mission_id,
                        worker_id,
                        token,
                        state_to_str(target),
                        payload_hash,
                        reason,
                        now_ms
                    ],
                )?;
                append_event_tx(
                    &tx,
                    mission_id,
                    "result_rejected",
                    &format!("result_id={result_id};reason={reason};token={token}"),
                    now_ms,
                )?;
                tx.commit()?;
                return Ok(ResultDisposition::Rejected { reason });
            }
            Some(receipt_id)
        } else {
            None
        };

        tx.execute(
            "UPDATE missions
             SET state=?2, terminal_result_hash=?3, updated_at_ms=?4
             WHERE mission_id=?1",
            params![mission_id, state_to_str(target), payload_hash, now_ms],
        )?;
        tx.execute(
            "INSERT INTO results(
                result_id,mission_id,worker_id,fencing_token,target_state,
                payload_hash,accepted,reason,created_at_ms
             ) VALUES (?1,?2,?3,?4,?5,?6,1,NULL,?7)",
            params![
                result_id,
                mission_id,
                worker_id,
                token,
                state_to_str(target),
                payload_hash,
                now_ms
            ],
        )?;
        let event_id = append_event_tx(
            &tx,
            mission_id,
            "result_accepted",
            &format!(
                "result_id={result_id};from={};to={};payload_hash={payload_hash};token={token};validation_receipt_id={}",
                state_to_str(current.state),
                state_to_str(target),
                validation_receipt_id.unwrap_or("")
            ),
            now_ms,
        )?;
        append_snapshot_tx(
            &tx,
            mission_id,
            target,
            &current.checkpoint,
            event_id,
            now_ms,
        )?;
        tx.execute("DELETE FROM queue WHERE mission_id=?1", params![mission_id])?;
        tx.execute(
            "DELETE FROM leases WHERE mission_id=?1",
            params![mission_id],
        )?;
        tx.commit()?;
        Ok(ResultDisposition::Accepted)
    }

    pub fn reconcile(&self, now_ms: i64) -> DurableResult<ReconcileReport> {
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

        let expired_ids = {
            let mut statement = tx.prepare(
                "SELECT mission_id FROM leases WHERE lease_until_ms<=?1 ORDER BY mission_id",
            )?;
            let rows = statement.query_map(params![now_ms], |row| row.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        for mission_id in &expired_ids {
            append_event_tx(
                &tx,
                mission_id,
                "lease_expired_reconciled",
                &format!("at={now_ms}"),
                now_ms,
            )?;
        }
        tx.execute(
            "DELETE FROM leases WHERE lease_until_ms<=?1",
            params![now_ms],
        )?;

        let requeued = tx.execute(
            "INSERT OR IGNORE INTO queue(mission_id,available_at_ms,enqueued_at_ms)
             SELECT mission_id,?1,?1 FROM missions
             WHERE state NOT IN ('succeeded','failed','deadline_exceeded')",
            params![now_ms],
        )?;

        let terminal_queue_removed = tx.execute(
            "DELETE FROM queue
             WHERE mission_id IN (
                 SELECT mission_id FROM missions
                 WHERE state IN ('succeeded','failed','deadline_exceeded')
             )",
            [],
        )?;

        tx.commit()?;
        Ok(ReconcileReport {
            expired_leases: expired_ids.len(),
            requeued,
            terminal_queue_removed,
        })
    }

    pub fn verify_ledger(&self) -> DurableResult<LedgerReport> {
        let conn = self.connect()?;
        let mut previous: HashMap<String, (i64, String)> = HashMap::new();
        let mut events = 0usize;

        let mut statement = conn.prepare(
            "SELECT mission_id,ordinal,kind,payload,prev_hash,event_hash,created_at_ms
             FROM mission_events ORDER BY mission_id,ordinal",
        )?;
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            let mission_id: String = row.get(0)?;
            let ordinal: i64 = row.get(1)?;
            let kind: String = row.get(2)?;
            let payload: String = row.get(3)?;
            let prev_hash: String = row.get(4)?;
            let event_hash: String = row.get(5)?;
            let created_at_ms: i64 = row.get(6)?;

            let (expected_ordinal, expected_prev) = match previous.get(&mission_id) {
                Some((last_ordinal, last_hash)) => (last_ordinal + 1, last_hash.clone()),
                None => (1, "GENESIS".to_string()),
            };

            if ordinal != expected_ordinal {
                return Err(DurableError::LedgerCorrupt(format!(
                    "{mission_id}: ordinal {ordinal} expected {expected_ordinal}"
                )));
            }
            if prev_hash != expected_prev {
                return Err(DurableError::LedgerCorrupt(format!(
                    "{mission_id}: prev hash mismatch at ordinal {ordinal}"
                )));
            }
            let expected_hash = event_hash_for(
                &mission_id,
                ordinal,
                &kind,
                &payload,
                &prev_hash,
                created_at_ms,
            );
            if event_hash != expected_hash {
                return Err(DurableError::LedgerCorrupt(format!(
                    "{mission_id}: event hash mismatch at ordinal {ordinal}"
                )));
            }
            previous.insert(mission_id, (ordinal, event_hash));
            events += 1;
        }

        let dangling_snapshots = conn.query_row(
            "SELECT COUNT(*)
             FROM mission_snapshots s
             LEFT JOIN mission_events e ON e.id=s.event_id
             WHERE e.id IS NULL",
            [],
            |row| row.get::<_, i64>(0),
        )?;
        if dangling_snapshots != 0 {
            return Err(DurableError::LedgerCorrupt(format!(
                "dangling snapshots: {dangling_snapshots}"
            )));
        }

        let missions = conn.query_row("SELECT COUNT(*) FROM missions", [], |row| {
            row.get::<_, i64>(0)
        })?;
        let snapshots = conn.query_row("SELECT COUNT(*) FROM mission_snapshots", [], |row| {
            row.get::<_, i64>(0)
        })?;
        Ok(LedgerReport {
            missions: missions as usize,
            events,
            snapshots: snapshots as usize,
        })
    }
}

fn validate_spec(spec: &MissionSpec) -> DurableResult<()> {
    if spec.mission_id.trim().is_empty()
        || spec.task_id.trim().is_empty()
        || spec.idempotency_key.trim().is_empty()
        || spec.payload_hash.trim().is_empty()
        || spec.deadline_unix_ms <= 0
    {
        return Err(DurableError::InvalidInput(
            "mission/task/idempotency/payload required and deadline positive".to_string(),
        ));
    }
    Ok(())
}

fn mission_from_conn(conn: &Connection, mission_id: &str) -> DurableResult<MissionRecord> {
    conn.query_row(
        "SELECT mission_id,task_id,state,deadline_unix_ms,priority,idempotency_key,
                payload_hash,checkpoint,fence_seq,terminal_result_hash
         FROM missions WHERE mission_id=?1",
        params![mission_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, i64>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        },
    )
    .optional()?
    .map(tuple_to_mission)
    .transpose()?
    .ok_or_else(|| DurableError::MissionNotFound(mission_id.to_string()))
}

fn mission_from_tx(tx: &Transaction<'_>, mission_id: &str) -> DurableResult<MissionRecord> {
    tx.query_row(
        "SELECT mission_id,task_id,state,deadline_unix_ms,priority,idempotency_key,
                payload_hash,checkpoint,fence_seq,terminal_result_hash
         FROM missions WHERE mission_id=?1",
        params![mission_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, i64>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        },
    )
    .optional()?
    .map(tuple_to_mission)
    .transpose()?
    .ok_or_else(|| DurableError::MissionNotFound(mission_id.to_string()))
}

type MissionTuple = (
    String,
    String,
    String,
    i64,
    i64,
    String,
    String,
    String,
    i64,
    Option<String>,
);

fn tuple_to_mission(value: MissionTuple) -> DurableResult<MissionRecord> {
    Ok(MissionRecord {
        mission_id: value.0,
        task_id: value.1,
        state: state_from_str(&value.2)?,
        deadline_unix_ms: value.3,
        priority: value.4,
        idempotency_key: value.5,
        payload_hash: value.6,
        checkpoint: value.7,
        fence_seq: value.8,
        terminal_result_hash: value.9,
    })
}

fn verify_active_lease_tx(
    tx: &Transaction<'_>,
    mission_id: &str,
    worker_id: &str,
    token: i64,
    now_ms: i64,
) -> DurableResult<()> {
    let lease = tx
        .query_row(
            "SELECT owner,fencing_token,lease_until_ms FROM leases WHERE mission_id=?1",
            params![mission_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| DurableError::LeaseMissing(mission_id.to_string()))?;

    if lease.0 != worker_id {
        return Err(DurableError::WrongOwner);
    }
    if lease.1 != token {
        return Err(DurableError::StaleFence {
            expected: lease.1,
            actual: token,
        });
    }
    if lease.2 <= now_ms {
        return Err(DurableError::LeaseExpired);
    }
    Ok(())
}

fn transition_projection_tx(
    tx: &Transaction<'_>,
    mission_id: &str,
    from: MissionState,
    to: MissionState,
    checkpoint: &str,
    now_ms: i64,
) -> DurableResult<i64> {
    if !allowed(from, to) {
        return Err(DurableError::IllegalTransition(TransitionError {
            from,
            to,
        }));
    }
    tx.execute(
        "UPDATE missions SET state=?2, updated_at_ms=?3 WHERE mission_id=?1",
        params![mission_id, state_to_str(to), now_ms],
    )?;
    let event_id = append_event_tx(
        tx,
        mission_id,
        "state_transition",
        &format!("from={};to={}", state_to_str(from), state_to_str(to)),
        now_ms,
    )?;
    append_snapshot_tx(tx, mission_id, to, checkpoint, event_id, now_ms)?;
    Ok(event_id)
}

fn append_event_tx(
    tx: &Transaction<'_>,
    mission_id: &str,
    kind: &str,
    payload: &str,
    now_ms: i64,
) -> DurableResult<i64> {
    let previous = tx
        .query_row(
            "SELECT ordinal,event_hash FROM mission_events
             WHERE mission_id=?1 ORDER BY ordinal DESC LIMIT 1",
            params![mission_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;

    let (ordinal, prev_hash) = match previous {
        Some((ordinal, hash)) => (ordinal + 1, hash),
        None => (1, "GENESIS".to_string()),
    };
    let event_hash = event_hash_for(mission_id, ordinal, kind, payload, &prev_hash, now_ms);

    tx.execute(
        "INSERT INTO mission_events(
            mission_id,ordinal,kind,payload,prev_hash,event_hash,created_at_ms
         ) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            mission_id, ordinal, kind, payload, prev_hash, event_hash, now_ms
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

fn append_snapshot_tx(
    tx: &Transaction<'_>,
    mission_id: &str,
    state: MissionState,
    checkpoint: &str,
    event_id: i64,
    now_ms: i64,
) -> DurableResult<()> {
    tx.execute(
        "INSERT INTO mission_snapshots(
            mission_id,state,checkpoint,event_id,created_at_ms
         ) VALUES (?1,?2,?3,?4,?5)",
        params![
            mission_id,
            state_to_str(state),
            checkpoint,
            event_id,
            now_ms
        ],
    )?;
    Ok(())
}

fn event_hash_for(
    mission_id: &str,
    ordinal: i64,
    kind: &str,
    payload: &str,
    prev_hash: &str,
    created_at_ms: i64,
) -> String {
    let input = format!("{mission_id}|{ordinal}|{kind}|{payload}|{prev_hash}|{created_at_ms}");
    let digest = Sha256::digest(input.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn lease_rejection_reason(error: &DurableError) -> Option<String> {
    match error {
        DurableError::LeaseMissing(_) => Some("lease_missing".to_string()),
        DurableError::WrongOwner => Some("wrong_owner".to_string()),
        DurableError::StaleFence { .. } => Some("stale_fence".to_string()),
        DurableError::LeaseExpired => Some("lease_expired".to_string()),
        _ => None,
    }
}

fn state_to_str(state: MissionState) -> &'static str {
    match state {
        MissionState::Received => "received",
        MissionState::Planned => "planned",
        MissionState::Running => "running",
        MissionState::Validating => "validating",
        MissionState::Succeeded => "succeeded",
        MissionState::Failed => "failed",
        MissionState::DeadlineExceeded => "deadline_exceeded",
    }
}

fn state_from_str(value: &str) -> DurableResult<MissionState> {
    match value {
        "received" => Ok(MissionState::Received),
        "planned" => Ok(MissionState::Planned),
        "running" => Ok(MissionState::Running),
        "validating" => Ok(MissionState::Validating),
        "succeeded" => Ok(MissionState::Succeeded),
        "failed" => Ok(MissionState::Failed),
        "deadline_exceeded" => Ok(MissionState::DeadlineExceeded),
        other => Err(DurableError::InvalidState(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn db_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "tma-f05-{name}-{}-{nonce}.sqlite3",
            std::process::id()
        ))
    }

    fn cleanup(path: &Path) {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}-wal", path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", path.display()));
    }

    fn spec(id: &str) -> MissionSpec {
        MissionSpec {
            mission_id: id.to_string(),
            task_id: format!("task-{id}"),
            deadline_unix_ms: 50_000,
            priority: 10,
            idempotency_key: format!("idem-{id}"),
            payload_hash: format!("payload-{id}"),
        }
    }

    #[test]
    fn sqlite_wal_full_and_foreign_keys_are_enabled() {
        let path = db_path("pragma");
        let store = DurableStore::open(&path).unwrap();
        let pragmas = store.pragmas().unwrap();
        assert_eq!(pragmas.journal_mode.to_lowercase(), "wal");
        assert_eq!(pragmas.synchronous, 2);
        assert_eq!(pragmas.foreign_keys, 1);
        assert_eq!(store.integrity_check().unwrap(), "ok");
        cleanup(&path);
    }

    #[test]
    fn mission_and_queue_survive_reopen() {
        let path = db_path("reopen");
        {
            let store = DurableStore::open(&path).unwrap();
            assert_eq!(
                store.create_mission(&spec("m1"), 100).unwrap(),
                CreateOutcome::Created("m1".to_string())
            );
            assert_eq!(store.queue_depth().unwrap(), 1);
        }
        let reopened = DurableStore::open(&path).unwrap();
        assert_eq!(reopened.mission("m1").unwrap().state, MissionState::Planned);
        assert_eq!(reopened.queue_depth().unwrap(), 1);
        assert!(reopened.event_count("m1").unwrap() >= 2);
        cleanup(&path);
    }

    #[test]
    fn idempotent_create_replays_and_conflict_is_rejected() {
        let path = db_path("idempotency");
        let store = DurableStore::open(&path).unwrap();
        let original = spec("m1");
        store.create_mission(&original, 100).unwrap();
        assert_eq!(
            store.create_mission(&original, 101).unwrap(),
            CreateOutcome::Existing("m1".to_string())
        );
        let mut conflicting = original.clone();
        conflicting.payload_hash = "different".to_string();
        assert!(matches!(
            store.create_mission(&conflicting, 102),
            Err(DurableError::IdempotencyConflict(_))
        ));
        cleanup(&path);
    }

    #[test]
    fn event_and_snapshot_tables_are_append_only() {
        let path = db_path("append-only");
        let store = DurableStore::open(&path).unwrap();
        store.create_mission(&spec("m1"), 100).unwrap();
        let conn = store.connect().unwrap();
        assert!(
            conn.execute(
                "UPDATE mission_events SET kind='x' WHERE mission_id='m1'",
                []
            )
            .is_err()
        );
        assert!(
            conn.execute("DELETE FROM mission_events WHERE mission_id='m1'", [])
                .is_err()
        );
        assert!(
            conn.execute(
                "UPDATE mission_snapshots SET state='x' WHERE mission_id='m1'",
                []
            )
            .is_err()
        );
        cleanup(&path);
    }

    #[test]
    fn checkpoint_and_snapshot_survive_reopen() {
        let path = db_path("checkpoint");
        let store = DurableStore::open(&path).unwrap();
        store.create_mission(&spec("m1"), 100).unwrap();
        let grant = match store.claim_next("worker-a", 200, 1_000).unwrap() {
            ClaimOutcome::Claimed(grant) => grant,
            ClaimOutcome::Empty => panic!("expected lease"),
        };
        store
            .save_checkpoint("m1", "worker-a", grant.fencing_token, r#"{"step":3}"#, 300)
            .unwrap();
        drop(store);
        let reopened = DurableStore::open(&path).unwrap();
        assert_eq!(reopened.mission("m1").unwrap().checkpoint, r#"{"step":3}"#);
        assert!(reopened.snapshot_count("m1").unwrap() >= 3);
        cleanup(&path);
    }

    #[test]
    fn lease_renewal_requires_owner_token_and_live_lease() {
        let path = db_path("renew");
        let store = DurableStore::open(&path).unwrap();
        store.create_mission(&spec("m1"), 100).unwrap();
        let grant = match store.claim_next("worker-a", 200, 100).unwrap() {
            ClaimOutcome::Claimed(grant) => grant,
            ClaimOutcome::Empty => panic!("expected lease"),
        };
        assert_eq!(
            store
                .renew_lease("m1", "worker-a", grant.fencing_token, 250, 200)
                .unwrap(),
            450
        );
        assert!(matches!(
            store.renew_lease("m1", "worker-b", grant.fencing_token, 260, 200),
            Err(DurableError::WrongOwner)
        ));
        assert!(matches!(
            store.renew_lease("m1", "worker-a", grant.fencing_token, 451, 200),
            Err(DurableError::LeaseExpired)
        ));
        cleanup(&path);
    }

    #[test]
    fn expired_lease_reclaim_increments_fence_and_rejects_stale_result() {
        let path = db_path("fence");
        let store = DurableStore::open(&path).unwrap();
        store.create_mission(&spec("m1"), 100).unwrap();
        let first = match store.claim_next("worker-a", 200, 100).unwrap() {
            ClaimOutcome::Claimed(grant) => grant,
            ClaimOutcome::Empty => panic!("expected first lease"),
        };
        store.reconcile(301).unwrap();
        let second = match store.claim_next("worker-b", 302, 1_000).unwrap() {
            ClaimOutcome::Claimed(grant) => grant,
            ClaimOutcome::Empty => panic!("expected second lease"),
        };
        assert!(second.fencing_token > first.fencing_token);
        let disposition = store
            .submit_result(&ResultSubmission {
                result_id: "result-stale".to_string(),
                mission_id: "m1".to_string(),
                worker_id: "worker-a".to_string(),
                fencing_token: first.fencing_token,
                target_state: MissionState::Failed,
                payload_hash: "hash-stale".to_string(),
                validation_receipt_id: None,
                created_at_ms: 303,
            })
            .unwrap();
        assert_eq!(
            disposition,
            ResultDisposition::Rejected {
                reason: "wrong_owner".to_string()
            }
        );
        cleanup(&path);
    }

    #[test]
    fn duplicate_result_does_not_duplicate_terminal_transition() {
        let path = db_path("result-dedupe");
        let store = DurableStore::open(&path).unwrap();
        store.create_mission(&spec("m1"), 100).unwrap();
        let grant = match store.claim_next("worker-a", 200, 1_000).unwrap() {
            ClaimOutcome::Claimed(grant) => grant,
            ClaimOutcome::Empty => panic!("expected lease"),
        };
        store
            .transition_with_lease(
                "m1",
                "worker-a",
                grant.fencing_token,
                MissionState::Validating,
                250,
            )
            .unwrap();
        let before = store.event_count("m1").unwrap();
        store
            .record_validation_receipt(&ValidationReceipt {
                receipt_id: "receipt-1".to_string(),
                mission_id: "m1".to_string(),
                validator_id: "validator-independent".to_string(),
                subject_payload_hash: "result-hash".to_string(),
                evidence_digest: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                accepted: true,
                created_at_ms: 290,
            })
            .unwrap();
        assert_eq!(
            store
                .submit_result(&ResultSubmission {
                    result_id: "result-1".to_string(),
                    mission_id: "m1".to_string(),
                    worker_id: "worker-a".to_string(),
                    fencing_token: grant.fencing_token,
                    target_state: MissionState::Succeeded,
                    payload_hash: "result-hash".to_string(),
                    validation_receipt_id: Some("receipt-1".to_string()),
                    created_at_ms: 300,
                })
                .unwrap(),
            ResultDisposition::Accepted
        );
        let after_first = store.event_count("m1").unwrap();
        assert!(after_first > before);
        assert_eq!(
            store
                .submit_result(&ResultSubmission {
                    result_id: "result-1".to_string(),
                    mission_id: "m1".to_string(),
                    worker_id: "worker-a".to_string(),
                    fencing_token: grant.fencing_token,
                    target_state: MissionState::Succeeded,
                    payload_hash: "result-hash".to_string(),
                    validation_receipt_id: Some("receipt-1".to_string()),
                    created_at_ms: 301,
                })
                .unwrap(),
            ResultDisposition::Duplicate {
                accepted: true,
                reason: None
            }
        );
        assert_eq!(store.event_count("m1").unwrap(), after_first);
        assert_eq!(store.queue_depth().unwrap(), 0);
        assert_eq!(store.mission("m1").unwrap().state, MissionState::Succeeded);
        cleanup(&path);
    }

    #[test]
    fn reconcile_restores_missing_queue_and_expired_running_lease() {
        let path = db_path("reconcile");
        let store = DurableStore::open(&path).unwrap();
        store.create_mission(&spec("m1"), 100).unwrap();
        let first = match store.claim_next("worker-a", 200, 100).unwrap() {
            ClaimOutcome::Claimed(grant) => grant,
            ClaimOutcome::Empty => panic!("expected lease"),
        };
        store
            .save_checkpoint("m1", "worker-a", first.fencing_token, "step=7", 220)
            .unwrap();
        {
            let conn = store.connect().unwrap();
            conn.execute("DELETE FROM queue WHERE mission_id='m1'", [])
                .unwrap();
        }
        drop(store);

        let reopened = DurableStore::open(&path).unwrap();
        let report = reopened.reconcile(500).unwrap();
        assert_eq!(report.expired_leases, 1);
        assert_eq!(report.requeued, 1);
        let second = match reopened.claim_next("worker-b", 501, 500).unwrap() {
            ClaimOutcome::Claimed(grant) => grant,
            ClaimOutcome::Empty => panic!("expected recovery lease"),
        };
        assert_eq!(second.state, MissionState::Running);
        assert_eq!(second.checkpoint, "step=7");
        assert!(second.fencing_token > first.fencing_token);
        cleanup(&path);
    }

    #[test]
    fn validating_state_is_reclaimed_without_state_regression() {
        let path = db_path("validating-recovery");
        let store = DurableStore::open(&path).unwrap();
        store.create_mission(&spec("m1"), 100).unwrap();
        let first = match store.claim_next("worker-a", 200, 100).unwrap() {
            ClaimOutcome::Claimed(grant) => grant,
            ClaimOutcome::Empty => panic!("expected lease"),
        };
        store
            .transition_with_lease(
                "m1",
                "worker-a",
                first.fencing_token,
                MissionState::Validating,
                220,
            )
            .unwrap();
        drop(store);

        let reopened = DurableStore::open(&path).unwrap();
        reopened.reconcile(500).unwrap();
        let second = match reopened.claim_next("worker-b", 501, 500).unwrap() {
            ClaimOutcome::Claimed(grant) => grant,
            ClaimOutcome::Empty => panic!("expected recovery lease"),
        };
        assert_eq!(second.state, MissionState::Validating);
        assert!(second.fencing_token > first.fencing_token);
        cleanup(&path);
    }

    #[test]
    fn succeeded_requires_independent_matching_validation_receipt() {
        let path = db_path("validation-receipt");
        let store = DurableStore::open(&path).unwrap();
        store.create_mission(&spec("m1"), 100).unwrap();
        let grant = match store.claim_next("executor-a", 200, 1_000).unwrap() {
            ClaimOutcome::Claimed(grant) => grant,
            ClaimOutcome::Empty => panic!("expected lease"),
        };
        store
            .transition_with_lease(
                "m1",
                "executor-a",
                grant.fencing_token,
                MissionState::Validating,
                220,
            )
            .unwrap();

        let rejected = store
            .submit_result(&ResultSubmission {
                result_id: "no-receipt".to_string(),
                mission_id: "m1".to_string(),
                worker_id: "executor-a".to_string(),
                fencing_token: grant.fencing_token,
                target_state: MissionState::Succeeded,
                payload_hash: "hash-ok".to_string(),
                validation_receipt_id: None,
                created_at_ms: 230,
            })
            .unwrap();
        assert_eq!(
            rejected,
            ResultDisposition::Rejected {
                reason: "validation_receipt_required".to_string()
            }
        );

        assert!(
            store
                .record_validation_receipt(&ValidationReceipt {
                    receipt_id: "same-owner".to_string(),
                    mission_id: "m1".to_string(),
                    validator_id: "executor-a".to_string(),
                    subject_payload_hash: "hash-ok".to_string(),
                    evidence_digest:
                        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                            .to_string(),
                    accepted: true,
                    created_at_ms: 240,
                })
                .is_err()
        );

        store
            .record_validation_receipt(&ValidationReceipt {
                receipt_id: "independent".to_string(),
                mission_id: "m1".to_string(),
                validator_id: "validator-b".to_string(),
                subject_payload_hash: "hash-ok".to_string(),
                evidence_digest: "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .to_string(),
                accepted: true,
                created_at_ms: 250,
            })
            .unwrap();

        let wrong_hash = store
            .submit_result(&ResultSubmission {
                result_id: "wrong-hash".to_string(),
                mission_id: "m1".to_string(),
                worker_id: "executor-a".to_string(),
                fencing_token: grant.fencing_token,
                target_state: MissionState::Succeeded,
                payload_hash: "hash-other".to_string(),
                validation_receipt_id: Some("independent".to_string()),
                created_at_ms: 260,
            })
            .unwrap();
        assert_eq!(
            wrong_hash,
            ResultDisposition::Rejected {
                reason: "validation_receipt_invalid".to_string()
            }
        );

        let accepted = store
            .submit_result(&ResultSubmission {
                result_id: "accepted".to_string(),
                mission_id: "m1".to_string(),
                worker_id: "executor-a".to_string(),
                fencing_token: grant.fencing_token,
                target_state: MissionState::Succeeded,
                payload_hash: "hash-ok".to_string(),
                validation_receipt_id: Some("independent".to_string()),
                created_at_ms: 270,
            })
            .unwrap();
        assert_eq!(accepted, ResultDisposition::Accepted);
        assert_eq!(store.mission("m1").unwrap().state, MissionState::Succeeded);
        cleanup(&path);
    }

    #[test]
    fn validation_receipts_are_append_only() {
        let path = db_path("validation-append-only");
        let store = DurableStore::open(&path).unwrap();
        store.create_mission(&spec("m1"), 100).unwrap();
        let grant = match store.claim_next("executor-a", 200, 1_000).unwrap() {
            ClaimOutcome::Claimed(grant) => grant,
            ClaimOutcome::Empty => panic!("expected lease"),
        };
        store
            .transition_with_lease(
                "m1",
                "executor-a",
                grant.fencing_token,
                MissionState::Validating,
                220,
            )
            .unwrap();
        store
            .record_validation_receipt(&ValidationReceipt {
                receipt_id: "receipt-append".to_string(),
                mission_id: "m1".to_string(),
                validator_id: "validator-b".to_string(),
                subject_payload_hash: "hash-ok".to_string(),
                evidence_digest: "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                    .to_string(),
                accepted: true,
                created_at_ms: 230,
            })
            .unwrap();
        let conn = store.connect().unwrap();
        assert!(
            conn.execute(
                "UPDATE validation_receipts SET accepted=0 WHERE receipt_id='receipt-append'",
                [],
            )
            .is_err()
        );
        assert!(
            conn.execute(
                "DELETE FROM validation_receipts WHERE receipt_id='receipt-append'",
                [],
            )
            .is_err()
        );
        cleanup(&path);
    }

    #[test]
    fn ledger_chain_and_sqlite_integrity_are_valid() {
        let path = db_path("ledger");
        let store = DurableStore::open(&path).unwrap();
        store.create_mission(&spec("m1"), 100).unwrap();
        let grant = match store.claim_next("worker-a", 200, 1_000).unwrap() {
            ClaimOutcome::Claimed(grant) => grant,
            ClaimOutcome::Empty => panic!("expected lease"),
        };
        store
            .save_checkpoint("m1", "worker-a", grant.fencing_token, "checkpoint", 210)
            .unwrap();
        let report = store.verify_ledger().unwrap();
        assert_eq!(report.missions, 1);
        assert!(report.events >= 4);
        assert!(report.snapshots >= 3);
        assert_eq!(store.integrity_check().unwrap(), "ok");
        cleanup(&path);
    }

    #[test]
    fn ledger_verifier_detects_tampering_when_append_only_guard_is_removed() {
        let path = db_path("tamper");
        let store = DurableStore::open(&path).unwrap();
        store.create_mission(&spec("m1"), 100).unwrap();
        {
            let conn = store.connect().unwrap();
            conn.execute_batch("DROP TRIGGER mission_events_no_update;")
                .unwrap();
            conn.execute(
                "UPDATE mission_events SET payload='tampered'
                 WHERE mission_id='m1' AND ordinal=1",
                [],
            )
            .unwrap();
        }
        assert!(matches!(
            store.verify_ledger(),
            Err(DurableError::LedgerCorrupt(_))
        ));
        cleanup(&path);
    }

    #[test]
    fn sixteen_way_claim_race_has_one_active_owner() {
        let path = db_path("race16");
        let store = DurableStore::open(&path).unwrap();
        store.create_mission(&spec("m1"), 100).unwrap();
        drop(store);

        let barrier = Arc::new(Barrier::new(16));
        let mut handles = Vec::new();
        for index in 0..16 {
            let path = path.clone();
            let barrier = barrier.clone();
            handles.push(thread::spawn(move || {
                let store = DurableStore::open(path).unwrap();
                barrier.wait();
                store
                    .claim_next(&format!("worker-{index}"), 200, 1_000)
                    .unwrap()
            }));
        }

        let claims = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|outcome| matches!(outcome, ClaimOutcome::Claimed(_)))
            .count();
        assert_eq!(claims, 1);
        cleanup(&path);
    }
}
