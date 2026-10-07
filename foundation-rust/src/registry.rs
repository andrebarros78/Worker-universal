use std::collections::{BTreeMap, BTreeSet};

use crate::model::{
    CapabilityDescriptor, CapabilitySnapshotEntry, LifecycleState, PromotionState, ReadinessState,
    ResolveRequest, compare_candidates,
};

#[derive(Debug, Default)]
pub struct CapabilityRegistry {
    entries: BTreeMap<String, CapabilityDescriptor>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, descriptor: CapabilityDescriptor) -> Result<(), String> {
        descriptor.validate().map_err(str::to_owned)?;
        if self.entries.contains_key(&descriptor.id) {
            return Err(format!("duplicate capability: {}", descriptor.id));
        }
        self.entries.insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    pub fn upsert(&mut self, descriptor: CapabilityDescriptor) -> Result<(), String> {
        descriptor.validate().map_err(str::to_owned)?;
        self.entries.insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    pub fn remove(&mut self, id: &str) -> Option<CapabilityDescriptor> {
        self.entries.remove(id)
    }

    pub fn get(&self, id: &str) -> Option<&CapabilityDescriptor> {
        self.entries.get(id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn snapshot(&self) -> Vec<CapabilitySnapshotEntry> {
        self.entries
            .values()
            .map(|item| CapabilitySnapshotEntry {
                id: item.id.clone(),
                lifecycle: item.lifecycle,
                readiness: item.readiness,
                promotion: item.promotion,
                enabled: item.enabled,
                contract_version: item.contract_version,
            })
            .collect()
    }

    pub fn dependencies_ready(
        &self,
        descriptor: &CapabilityDescriptor,
        allow_degraded: bool,
    ) -> bool {
        descriptor.dependencies.iter().all(|dependency_id| {
            self.entries
                .get(dependency_id)
                .is_some_and(|dependency| dependency.operational(allow_degraded))
        })
    }

    pub fn has_dependency_cycle(&self) -> bool {
        fn visit(
            id: &str,
            registry: &CapabilityRegistry,
            visiting: &mut BTreeSet<String>,
            visited: &mut BTreeSet<String>,
        ) -> bool {
            if visited.contains(id) {
                return false;
            }
            if !visiting.insert(id.to_string()) {
                return true;
            }

            if let Some(item) = registry.get(id) {
                for dependency in &item.dependencies {
                    if registry.get(dependency).is_some()
                        && visit(dependency, registry, visiting, visited)
                    {
                        return true;
                    }
                }
            }

            visiting.remove(id);
            visited.insert(id.to_string());
            false
        }

        let mut visited = BTreeSet::new();
        let mut visiting = BTreeSet::new();
        self.entries
            .keys()
            .any(|id| visit(id, self, &mut visiting, &mut visited))
    }

    pub fn resolve(&self, request: &ResolveRequest) -> Vec<&CapabilityDescriptor> {
        let mut matches = self
            .entries
            .values()
            .filter(|item| {
                request.kind.is_none_or(|kind| item.kind == kind)
                    && item
                        .contract_version
                        .compatible_with(request.required_contract)
                    && item.promotion >= request.minimum_promotion
                    && item.reliability_bps >= request.minimum_reliability_bps
                    && request
                        .maximum_cost_microunits
                        .is_none_or(|maximum| item.cost_microunits <= maximum)
                    && request
                        .maximum_p95_ms
                        .is_none_or(|maximum| item.p95_ms <= maximum)
                    && request
                        .required_permissions
                        .iter()
                        .all(|required| item.permissions.contains(required))
                    && request
                        .required_evidence
                        .iter()
                        .all(|required| item.evidence_types.contains(required))
                    && item.operational(request.allow_degraded)
                    && self.dependencies_ready(item, request.allow_degraded)
            })
            .collect::<Vec<_>>();

        matches.sort_by(|left, right| compare_candidates(request, left, right));
        matches
    }

    pub fn fallback_chain(&self, id: &str, allow_degraded: bool) -> Vec<&CapabilityDescriptor> {
        let Some(primary) = self.get(id) else {
            return Vec::new();
        };
        primary
            .fallbacks
            .iter()
            .filter_map(|fallback| self.get(fallback))
            .filter(|fallback| {
                fallback.operational(allow_degraded)
                    && self.dependencies_ready(fallback, allow_degraded)
            })
            .collect()
    }

    pub fn health_counts(&self) -> (usize, usize, usize, usize) {
        let mut healthy = 0;
        let mut degraded = 0;
        let mut failed = 0;
        let mut disabled = 0;
        for item in self.entries.values() {
            match item.lifecycle {
                LifecycleState::Healthy => healthy += 1,
                LifecycleState::Degraded => degraded += 1,
                LifecycleState::Failed | LifecycleState::Unavailable => failed += 1,
                LifecycleState::Disabled => disabled += 1,
                LifecycleState::Starting => {}
            }
        }
        (healthy, degraded, failed, disabled)
    }

    pub fn set_health(
        &mut self,
        id: &str,
        lifecycle: LifecycleState,
        readiness: ReadinessState,
    ) -> Result<(), String> {
        let item = self
            .entries
            .get_mut(id)
            .ok_or_else(|| format!("unknown capability: {id}"))?;
        item.lifecycle = lifecycle;
        item.readiness = readiness;
        Ok(())
    }

    pub fn set_promotion(&mut self, id: &str, promotion: PromotionState) -> Result<(), String> {
        let item = self
            .entries
            .get_mut(id)
            .ok_or_else(|| format!("unknown capability: {id}"))?;
        item.promotion = promotion;
        Ok(())
    }
}
