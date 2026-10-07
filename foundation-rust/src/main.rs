use std::sync::Arc;

use tma_foundation::contracts::AdapterManifest;
use tma_foundation::model::{
    CapabilityDescriptor, CapabilityKind, IdempotencyClass, LifecycleState, PromotionState,
    ReadinessState, SideEffectClass, TransportKind, Version,
};
use tma_foundation::registry::CapabilityRegistry;
use tma_foundation::testkit::adapter_conformance;
use tma_foundation::tool_bus::LocalEchoAdapter;

fn local_descriptor() -> CapabilityDescriptor {
    CapabilityDescriptor {
        id: "native.local.echo".to_string(),
        kind: CapabilityKind::Native,
        provider: Some("local".to_string()),
        contract_version: Version::new(1, 0, 0),
        dependencies: Vec::new(),
        permissions: Vec::new(),
        fallbacks: Vec::new(),
        lifecycle: LifecycleState::Healthy,
        readiness: ReadinessState::Operational,
        promotion: PromotionState::Production,
        side_effect: SideEffectClass::None,
        idempotency: IdempotencyClass::Safe,
        max_concurrency: 16,
        resource_locks: Vec::new(),
        cost_microunits: 0,
        p95_ms: 5,
        reliability_bps: 10_000,
        evidence_types: vec!["structured_result".to_string()],
        transport: TransportKind::Native,
        config_ref: None,
        enabled: true,
    }
}

fn self_test() -> Result<(), String> {
    let mut registry = CapabilityRegistry::new();
    if !registry.is_empty() {
        return Err("providerless registry must boot empty".to_string());
    }

    let descriptor = local_descriptor();
    registry.register(descriptor.clone())?;

    let manifest = AdapterManifest {
        adapter_id: "local.echo".to_string(),
        adapter_version: Version::new(1, 0, 0),
        sdk_contract_version: Version::new(1, 0, 0),
        capabilities: vec![descriptor.id.clone()],
        config_schema_ref: None,
        secret_refs: Vec::new(),
        simulation_supported: true,
    };
    let report = adapter_conformance(
        &descriptor,
        &manifest,
        Arc::new(LocalEchoAdapter::new(
            "native.local.echo",
            Version::new(1, 0, 0),
        )),
    );
    if !report.passed {
        return Err(format!("conformance failed: {:?}", report.failures));
    }

    println!(
        "tma-foundation status=ok registered={} checks={} external_providers=0",
        registry.len(),
        report.checks.len()
    );
    Ok(())
}

fn main() {
    let command = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "self-test".to_string());
    match command.as_str() {
        "self-test" => {
            if let Err(error) = self_test() {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("unknown command: {command}");
            std::process::exit(2);
        }
    }
}
