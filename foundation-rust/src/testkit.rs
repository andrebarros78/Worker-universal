use std::sync::Arc;

use crate::contracts::AdapterManifest;
use crate::model::{CapabilityDescriptor, Version};
use crate::tool_bus::{
    CancellationToken, InMemoryTelemetry, Invocation, ResultStatus, ToolAdapter, ToolBus,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceReport {
    pub passed: bool,
    pub checks: Vec<String>,
    pub failures: Vec<String>,
}

pub fn adapter_conformance(
    descriptor: &CapabilityDescriptor,
    manifest: &AdapterManifest,
    adapter: Arc<dyn ToolAdapter>,
) -> ConformanceReport {
    let mut checks = Vec::new();
    let mut failures = Vec::new();

    if descriptor.validate().is_ok() {
        checks.push("descriptor_valid".to_string());
    } else {
        failures.push("descriptor_invalid".to_string());
    }

    if manifest.validate().is_ok() {
        checks.push("manifest_valid".to_string());
    } else {
        failures.push("manifest_invalid".to_string());
    }

    if manifest.capabilities.contains(&descriptor.id) {
        checks.push("manifest_exposes_capability".to_string());
    } else {
        failures.push("manifest_missing_capability".to_string());
    }

    if adapter.capability_id() == descriptor.id {
        checks.push("adapter_id_matches_descriptor".to_string());
    } else {
        failures.push("adapter_id_mismatch".to_string());
    }

    if adapter
        .contract_version()
        .compatible_with(descriptor.contract_version)
    {
        checks.push("contract_compatible".to_string());
    } else {
        failures.push("contract_incompatible".to_string());
    }

    if manifest.simulation_supported && adapter.supports_simulation() {
        let telemetry = Arc::new(InMemoryTelemetry::default());
        let mut bus = ToolBus::new(telemetry);
        if bus.register(adapter).is_err() {
            failures.push("adapter_registration_failed".to_string());
        } else {
            let result = bus.invoke(
                &Invocation {
                    invocation_id: "conformance-001".to_string(),
                    mission_id: "conformance".to_string(),
                    capability_id: descriptor.id.clone(),
                    contract_version: Version::new(descriptor.contract_version.major, 0, 0),
                    timeout_ms: 100,
                    idempotency_key: Some("conformance".to_string()),
                    session_ref: None,
                    secret_refs: Vec::new(),
                    artifact_refs: Vec::new(),
                    simulation: true,
                    payload: "ping".to_string(),
                    replayable: true,
                },
                &CancellationToken::default(),
            );
            if result.status == ResultStatus::Succeeded {
                checks.push("simulation_invoke".to_string());
            } else {
                failures.push("simulation_invoke_failed".to_string());
            }
        }
    }

    ConformanceReport {
        passed: failures.is_empty(),
        checks,
        failures,
    }
}
