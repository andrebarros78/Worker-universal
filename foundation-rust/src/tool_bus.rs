use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::model::{ErrorClass, StableError, Version};
use crate::services::{ArtifactRef, SecretRef, SessionRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub invocation_id: String,
    pub mission_id: String,
    pub capability_id: String,
    pub contract_version: Version,
    pub timeout_ms: u64,
    pub idempotency_key: Option<String>,
    pub session_ref: Option<SessionRef>,
    pub secret_refs: Vec<SecretRef>,
    pub artifact_refs: Vec<ArtifactRef>,
    pub simulation: bool,
    pub payload: String,
    pub replayable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultStatus {
    Succeeded,
    Failed,
    Cancelled,
    DeadlineExceeded,
    NeedsReview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    pub kind: String,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolResult {
    pub invocation_id: String,
    pub capability_id: String,
    pub status: ResultStatus,
    pub elapsed_ms: u64,
    pub payload: Option<String>,
    pub errors: Vec<StableError>,
    pub evidence: Vec<Evidence>,
    pub artifact_refs: Vec<ArtifactRef>,
    pub session_ref: Option<SessionRef>,
    pub cost_microunits: u64,
}

#[derive(Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelemetryEventKind {
    InvocationStarted,
    InvocationFinished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryEvent {
    pub kind: TelemetryEventKind,
    pub invocation_id: String,
    pub capability_id: String,
    pub elapsed_ms: Option<u64>,
    pub status: Option<ResultStatus>,
}

pub trait TelemetrySink: Send + Sync {
    fn emit(&self, event: TelemetryEvent);
}

#[derive(Default)]
pub struct InMemoryTelemetry {
    events: Mutex<Vec<TelemetryEvent>>,
}

impl InMemoryTelemetry {
    pub fn events(&self) -> Vec<TelemetryEvent> {
        self.events.lock().expect("telemetry lock poisoned").clone()
    }
}

impl TelemetrySink for InMemoryTelemetry {
    fn emit(&self, event: TelemetryEvent) {
        self.events
            .lock()
            .expect("telemetry lock poisoned")
            .push(event);
    }
}

pub trait ToolAdapter: Send + Sync {
    fn capability_id(&self) -> &str;
    fn contract_version(&self) -> Version;
    fn supports_simulation(&self) -> bool;
    fn invoke(&self, invocation: &Invocation, cancellation: &CancellationToken) -> ToolResult;
}

pub struct ToolBus {
    adapters: BTreeMap<String, Arc<dyn ToolAdapter>>,
    telemetry: Arc<dyn TelemetrySink>,
}

impl ToolBus {
    pub fn new(telemetry: Arc<dyn TelemetrySink>) -> Self {
        Self {
            adapters: BTreeMap::new(),
            telemetry,
        }
    }

    pub fn register(&mut self, adapter: Arc<dyn ToolAdapter>) -> Result<(), String> {
        let id = adapter.capability_id().to_string();
        if self.adapters.contains_key(&id) {
            return Err(format!("duplicate adapter: {id}"));
        }
        self.adapters.insert(id, adapter);
        Ok(())
    }

    pub fn invoke(&self, invocation: &Invocation, cancellation: &CancellationToken) -> ToolResult {
        let started = Instant::now();
        self.telemetry.emit(TelemetryEvent {
            kind: TelemetryEventKind::InvocationStarted,
            invocation_id: invocation.invocation_id.clone(),
            capability_id: invocation.capability_id.clone(),
            elapsed_ms: None,
            status: None,
        });

        let mut result = if cancellation.is_cancelled() {
            failed_result(
                invocation,
                ResultStatus::Cancelled,
                ErrorClass::Cancelled,
                false,
                "cancelled_before_start",
            )
        } else if invocation.timeout_ms == 0 {
            failed_result(
                invocation,
                ResultStatus::DeadlineExceeded,
                ErrorClass::DeadlineRisk,
                false,
                "timeout_must_be_positive",
            )
        } else if let Some(adapter) = self.adapters.get(&invocation.capability_id) {
            if !adapter
                .contract_version()
                .compatible_with(invocation.contract_version)
            {
                failed_result(
                    invocation,
                    ResultStatus::Failed,
                    ErrorClass::ContractMismatch,
                    false,
                    "contract_mismatch",
                )
            } else if invocation.simulation && !adapter.supports_simulation() {
                failed_result(
                    invocation,
                    ResultStatus::Failed,
                    ErrorClass::ExternalDependency,
                    false,
                    "simulation_not_supported",
                )
            } else {
                adapter.invoke(invocation, cancellation)
            }
        } else {
            failed_result(
                invocation,
                ResultStatus::Failed,
                ErrorClass::ExternalDependency,
                true,
                "adapter_unavailable",
            )
        };

        result.elapsed_ms = started.elapsed().as_millis() as u64;
        if result.elapsed_ms > invocation.timeout_ms && result.status == ResultStatus::Succeeded {
            result.status = ResultStatus::DeadlineExceeded;
            result.errors.push(StableError {
                class: ErrorClass::DeadlineRisk,
                retryable: false,
                message: "adapter_result_arrived_after_timeout".to_string(),
                detail_code: None,
            });
        }

        self.telemetry.emit(TelemetryEvent {
            kind: TelemetryEventKind::InvocationFinished,
            invocation_id: invocation.invocation_id.clone(),
            capability_id: invocation.capability_id.clone(),
            elapsed_ms: Some(result.elapsed_ms),
            status: Some(result.status),
        });
        result
    }
}

fn failed_result(
    invocation: &Invocation,
    status: ResultStatus,
    class: ErrorClass,
    retryable: bool,
    message: &str,
) -> ToolResult {
    ToolResult {
        invocation_id: invocation.invocation_id.clone(),
        capability_id: invocation.capability_id.clone(),
        status,
        elapsed_ms: 0,
        payload: None,
        errors: vec![StableError {
            class,
            retryable,
            message: message.to_string(),
            detail_code: None,
        }],
        evidence: Vec::new(),
        artifact_refs: Vec::new(),
        session_ref: invocation.session_ref.clone(),
        cost_microunits: 0,
    }
}

pub struct LocalEchoAdapter {
    id: String,
    version: Version,
}

impl LocalEchoAdapter {
    pub fn new(id: impl Into<String>, version: Version) -> Self {
        Self {
            id: id.into(),
            version,
        }
    }
}

impl ToolAdapter for LocalEchoAdapter {
    fn capability_id(&self) -> &str {
        &self.id
    }

    fn contract_version(&self) -> Version {
        self.version
    }

    fn supports_simulation(&self) -> bool {
        true
    }

    fn invoke(&self, invocation: &Invocation, cancellation: &CancellationToken) -> ToolResult {
        if cancellation.is_cancelled() {
            return failed_result(
                invocation,
                ResultStatus::Cancelled,
                ErrorClass::Cancelled,
                false,
                "cancelled",
            );
        }
        ToolResult {
            invocation_id: invocation.invocation_id.clone(),
            capability_id: invocation.capability_id.clone(),
            status: ResultStatus::Succeeded,
            elapsed_ms: 0,
            payload: Some(invocation.payload.clone()),
            errors: Vec::new(),
            evidence: vec![Evidence {
                kind: "structured_result".to_string(),
                reference: "inline".to_string(),
            }],
            artifact_refs: Vec::new(),
            session_ref: invocation.session_ref.clone(),
            cost_microunits: 0,
        }
    }
}
