pub mod contracts;
pub mod model;
pub mod registry;
pub mod services;
pub mod testkit;
pub mod tool_bus;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    use crate::contracts::{AdapterManifest, ContractDescriptor, ContractRegistry};
    use crate::model::{
        CapabilityDescriptor, CapabilityKind, ErrorClass, IdempotencyClass, LifecycleState,
        PromotionState, ReadinessState, ResolveRequest, SideEffectClass, TransportKind, Version,
    };
    use crate::registry::CapabilityRegistry;
    use crate::services::{
        ArtifactStore, InMemorySecretProvider, InMemorySessionManager, LocalArtifactStore,
        PolicyContext, PolicyEngine, ScopePolicy, SecretProvider, SecretRef,
    };
    use crate::testkit::adapter_conformance;
    use crate::tool_bus::{
        CancellationToken, InMemoryTelemetry, Invocation, LocalEchoAdapter, ResultStatus,
        ToolAdapter, ToolBus,
    };

    fn descriptor(
        id: &str,
        provider: &str,
        reliability: u16,
        cost: u64,
        p95: u64,
    ) -> CapabilityDescriptor {
        CapabilityDescriptor {
            id: id.to_string(),
            kind: CapabilityKind::Native,
            provider: Some(provider.to_string()),
            contract_version: Version::new(1, 1, 0),
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
            cost_microunits: cost,
            p95_ms: p95,
            reliability_bps: reliability,
            evidence_types: vec!["structured_result".to_string()],
            transport: TransportKind::Native,
            config_ref: None,
            enabled: true,
        }
    }

    fn request() -> ResolveRequest {
        ResolveRequest {
            kind: Some(CapabilityKind::Native),
            required_contract: Version::new(1, 0, 0),
            required_permissions: Vec::new(),
            required_evidence: vec!["structured_result".to_string()],
            minimum_promotion: PromotionState::Qualified,
            minimum_reliability_bps: 9000,
            maximum_cost_microunits: Some(100),
            maximum_p95_ms: Some(100),
            provider_preference: Vec::new(),
            allow_degraded: false,
        }
    }

    #[test]
    fn descriptor_preserves_execution_and_transport_metadata() {
        let mut item = descriptor("native.meta", "local", 9876, 42, 77);
        item.side_effect = SideEffectClass::Submit;
        item.idempotency = IdempotencyClass::Conditional;
        item.max_concurrency = 3;
        item.resource_locks = vec!["session:browser-1".to_string()];
        item.transport = TransportKind::Api;
        item.config_ref = Some("config://native.meta".to_string());

        assert_eq!(item.side_effect, SideEffectClass::Submit);
        assert_eq!(item.idempotency, IdempotencyClass::Conditional);
        assert_eq!(item.max_concurrency, 3);
        assert_eq!(item.resource_locks, vec!["session:browser-1"]);
        assert_eq!(item.transport, TransportKind::Api);
        assert_eq!(item.config_ref.as_deref(), Some("config://native.meta"));
        assert_eq!(item.cost_microunits, 42);
        assert_eq!(item.p95_ms, 77);
        assert_eq!(item.reliability_bps, 9876);
    }

    #[test]
    fn providerless_registry_boots_empty() {
        let registry = CapabilityRegistry::new();
        assert!(registry.is_empty());
        assert!(registry.resolve(&request()).is_empty());
    }

    #[test]
    fn registry_add_get_remove_and_snapshot_are_deterministic() {
        let mut registry = CapabilityRegistry::new();
        registry
            .register(descriptor("native.b", "local", 9990, 1, 2))
            .unwrap();
        registry
            .register(descriptor("native.a", "local", 9990, 1, 2))
            .unwrap();
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.snapshot()[0].id, "native.a");
        assert!(registry.get("native.b").is_some());
        assert!(registry.remove("native.b").is_some());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn duplicate_registration_is_rejected() {
        let mut registry = CapabilityRegistry::new();
        registry
            .register(descriptor("native.a", "local", 9990, 1, 2))
            .unwrap();
        assert!(
            registry
                .register(descriptor("native.a", "local", 9990, 1, 2))
                .is_err()
        );
    }

    #[test]
    fn resolver_is_deterministic_and_uses_provider_priority() {
        let mut registry = CapabilityRegistry::new();
        registry
            .register(descriptor("native.a", "cheap", 9999, 1, 1))
            .unwrap();
        registry
            .register(descriptor("native.b", "preferred", 9900, 5, 5))
            .unwrap();

        let mut req = request();
        req.provider_preference = vec!["preferred".to_string(), "cheap".to_string()];
        let resolved = registry.resolve(&req);
        assert_eq!(resolved[0].id, "native.b");
    }

    #[test]
    fn resolver_falls_back_to_quality_cost_latency_and_id() {
        let mut registry = CapabilityRegistry::new();
        registry
            .register(descriptor("native.c", "x", 9800, 1, 5))
            .unwrap();
        registry
            .register(descriptor("native.b", "x", 9990, 9, 9))
            .unwrap();
        registry
            .register(descriptor("native.a", "x", 9990, 2, 8))
            .unwrap();

        let resolved = registry.resolve(&request());
        assert_eq!(resolved[0].id, "native.a");
        assert_eq!(resolved[1].id, "native.b");
    }

    #[test]
    fn contract_major_incompatibility_is_rejected() {
        let mut registry = CapabilityRegistry::new();
        registry
            .register(descriptor("native.a", "local", 9990, 1, 2))
            .unwrap();
        let mut req = request();
        req.required_contract = Version::new(2, 0, 0);
        assert!(registry.resolve(&req).is_empty());
    }

    #[test]
    fn dependency_readiness_and_cycle_detection_work() {
        let mut registry = CapabilityRegistry::new();
        let dep = descriptor("native.dep", "local", 9990, 1, 2);
        let mut parent = descriptor("native.parent", "local", 9990, 1, 2);
        parent.dependencies = vec!["native.dep".to_string()];
        registry.register(parent).unwrap();
        assert!(registry.resolve(&request()).is_empty());

        registry.register(dep).unwrap();
        assert!(!registry.resolve(&request()).is_empty());

        let mut dep_cycle = descriptor("native.dep", "local", 9990, 1, 2);
        dep_cycle.dependencies = vec!["native.parent".to_string()];
        registry.upsert(dep_cycle).unwrap();
        assert!(registry.has_dependency_cycle());
    }

    #[test]
    fn health_and_promotion_gates_are_enforced() {
        let mut registry = CapabilityRegistry::new();
        registry
            .register(descriptor("native.a", "local", 9990, 1, 2))
            .unwrap();
        registry
            .set_health(
                "native.a",
                LifecycleState::Failed,
                ReadinessState::Configured,
            )
            .unwrap();
        assert!(registry.resolve(&request()).is_empty());

        registry
            .set_health(
                "native.a",
                LifecycleState::Healthy,
                ReadinessState::Operational,
            )
            .unwrap();
        registry
            .set_promotion("native.a", PromotionState::Tested)
            .unwrap();
        assert!(registry.resolve(&request()).is_empty());
    }

    #[test]
    fn fallback_chain_uses_only_ready_capabilities() {
        let mut registry = CapabilityRegistry::new();
        let mut primary = descriptor("native.primary", "local", 9990, 1, 2);
        primary.fallbacks = vec!["native.f1".to_string(), "native.f2".to_string()];
        registry.register(primary).unwrap();
        registry
            .register(descriptor("native.f1", "local", 9990, 1, 2))
            .unwrap();
        let mut failed = descriptor("native.f2", "local", 9990, 1, 2);
        failed.lifecycle = LifecycleState::Failed;
        registry.register(failed).unwrap();
        let fallback = registry.fallback_chain("native.primary", false);
        assert_eq!(fallback.len(), 1);
        assert_eq!(fallback[0].id, "native.f1");
    }

    #[test]
    fn contract_registry_negotiates_latest_compatible_version() {
        let mut contracts = ContractRegistry::default();
        for version in [
            Version::new(1, 0, 0),
            Version::new(1, 2, 0),
            Version::new(2, 0, 0),
        ] {
            contracts
                .register(ContractDescriptor {
                    id: "tool.echo".to_string(),
                    version,
                    input_schema_ref: "in".to_string(),
                    output_schema_ref: "out".to_string(),
                })
                .unwrap();
        }
        assert_eq!(
            contracts
                .negotiate("tool.echo", Version::new(1, 1, 0))
                .unwrap()
                .version,
            Version::new(1, 2, 0)
        );
        assert!(
            contracts
                .negotiate("tool.echo", Version::new(3, 0, 0))
                .is_none()
        );
    }

    #[test]
    fn scope_policy_denies_missing_permission_and_explicit_denial() {
        let mut capability = descriptor("native.secured", "local", 9990, 1, 2);
        capability.permissions = vec!["documents.read".to_string()];
        let policy = ScopePolicy;
        let context = PolicyContext {
            granted_scopes: BTreeSet::new(),
            denied_capabilities: BTreeSet::new(),
        };
        assert!(!policy.evaluate(&context, &capability).allowed);

        let context = PolicyContext {
            granted_scopes: BTreeSet::from(["documents.read".to_string()]),
            denied_capabilities: BTreeSet::from(["native.secured".to_string()]),
        };
        assert!(!policy.evaluate(&context, &capability).allowed);
    }

    #[test]
    fn secret_debug_is_redacted() {
        let mut provider = InMemorySecretProvider::default();
        let reference = SecretRef("secret://test".to_string());
        provider.insert(reference.clone(), "super-secret-material");
        let secret = provider.resolve(&reference).unwrap();
        assert_eq!(secret.expose(), "super-secret-material");
        let debug = format!("{secret:?}");
        assert!(!debug.contains("super-secret-material"));
        assert!(debug.contains("REDACTED"));
    }

    #[test]
    fn local_artifact_store_round_trip() {
        let root =
            std::env::temp_dir().join(format!("tma-foundation-artifact-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let store = LocalArtifactStore::new(&root).unwrap();
        let reference = store.put("invoice/test.txt", b"payload").unwrap();
        assert_eq!(store.get(&reference).unwrap(), b"payload");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn session_manager_uses_opaque_handles() {
        let mut sessions = InMemorySessionManager::default();
        let reference = sessions.create("browser.playwright");
        assert!(reference.0.starts_with("session-"));
        assert_eq!(
            sessions.get(&reference).unwrap().capability_id,
            "browser.playwright"
        );
        sessions.close(&reference).unwrap();
        assert!(!sessions.get(&reference).unwrap().active);
    }

    fn invocation(version: Version) -> Invocation {
        Invocation {
            invocation_id: "inv-001".to_string(),
            mission_id: "mission-001".to_string(),
            capability_id: "native.local.echo".to_string(),
            contract_version: version,
            timeout_ms: 100,
            idempotency_key: Some("idem-001".to_string()),
            session_ref: None,
            secret_refs: vec![SecretRef("secret://opaque".to_string())],
            artifact_refs: Vec::new(),
            simulation: true,
            payload: "ping".to_string(),
            replayable: true,
        }
    }

    #[test]
    fn tool_bus_runs_offline_without_external_provider() {
        let telemetry = Arc::new(InMemoryTelemetry::default());
        let mut bus = ToolBus::new(telemetry.clone());
        bus.register(Arc::new(LocalEchoAdapter::new(
            "native.local.echo",
            Version::new(1, 0, 0),
        )))
        .unwrap();

        let result = bus.invoke(
            &invocation(Version::new(1, 0, 0)),
            &CancellationToken::default(),
        );
        assert_eq!(result.status, ResultStatus::Succeeded);
        assert_eq!(result.payload.as_deref(), Some("ping"));
        assert_eq!(telemetry.events().len(), 2);
    }

    #[test]
    fn tool_bus_rejects_incompatible_contract() {
        let telemetry = Arc::new(InMemoryTelemetry::default());
        let mut bus = ToolBus::new(telemetry);
        bus.register(Arc::new(LocalEchoAdapter::new(
            "native.local.echo",
            Version::new(1, 0, 0),
        )))
        .unwrap();
        let result = bus.invoke(
            &invocation(Version::new(2, 0, 0)),
            &CancellationToken::default(),
        );
        assert_eq!(result.status, ResultStatus::Failed);
        assert_eq!(result.errors[0].class, ErrorClass::ContractMismatch);
    }

    #[test]
    fn tool_bus_honors_cancellation() {
        let telemetry = Arc::new(InMemoryTelemetry::default());
        let mut bus = ToolBus::new(telemetry);
        bus.register(Arc::new(LocalEchoAdapter::new(
            "native.local.echo",
            Version::new(1, 0, 0),
        )))
        .unwrap();
        let token = CancellationToken::default();
        token.cancel();
        let result = bus.invoke(&invocation(Version::new(1, 0, 0)), &token);
        assert_eq!(result.status, ResultStatus::Cancelled);
    }

    struct SlowAdapter;

    impl ToolAdapter for SlowAdapter {
        fn capability_id(&self) -> &str {
            "native.local.echo"
        }

        fn contract_version(&self) -> Version {
            Version::new(1, 0, 0)
        }

        fn supports_simulation(&self) -> bool {
            true
        }

        fn invoke(
            &self,
            invocation: &Invocation,
            _cancellation: &CancellationToken,
        ) -> crate::tool_bus::ToolResult {
            thread::sleep(Duration::from_millis(15));
            crate::tool_bus::ToolResult {
                invocation_id: invocation.invocation_id.clone(),
                capability_id: invocation.capability_id.clone(),
                status: ResultStatus::Succeeded,
                elapsed_ms: 0,
                payload: Some("late".to_string()),
                errors: Vec::new(),
                evidence: Vec::new(),
                artifact_refs: Vec::new(),
                session_ref: None,
                cost_microunits: 0,
            }
        }
    }

    #[test]
    fn late_success_is_converted_to_deadline_exceeded() {
        let telemetry = Arc::new(InMemoryTelemetry::default());
        let mut bus = ToolBus::new(telemetry);
        bus.register(Arc::new(SlowAdapter)).unwrap();
        let mut call = invocation(Version::new(1, 0, 0));
        call.timeout_ms = 5;
        let result = bus.invoke(&call, &CancellationToken::default());
        assert_eq!(result.status, ResultStatus::DeadlineExceeded);
        assert_eq!(result.errors[0].class, ErrorClass::DeadlineRisk);
    }

    #[test]
    fn adapter_testkit_proves_local_echo() {
        let descriptor = descriptor("native.local.echo", "local", 10000, 0, 5);
        let manifest = AdapterManifest {
            adapter_id: "local.echo".to_string(),
            adapter_version: Version::new(1, 0, 0),
            sdk_contract_version: Version::new(1, 0, 0),
            capabilities: vec!["native.local.echo".to_string()],
            config_schema_ref: None,
            secret_refs: Vec::new(),
            simulation_supported: true,
        };
        let report = adapter_conformance(
            &descriptor,
            &manifest,
            Arc::new(LocalEchoAdapter::new(
                "native.local.echo",
                Version::new(1, 1, 0),
            )),
        );
        assert!(report.passed, "{:?}", report.failures);
        assert!(report.checks.contains(&"simulation_invoke".to_string()));
    }
}
