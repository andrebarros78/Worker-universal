use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::model::CapabilityDescriptor;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SecretRef(pub String);

pub struct SecretValue(String);

impl SecretValue {
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretValue([REDACTED])")
    }
}

pub trait SecretProvider {
    fn resolve(&self, reference: &SecretRef) -> Option<SecretValue>;
}

#[derive(Default)]
pub struct InMemorySecretProvider {
    values: BTreeMap<String, String>,
}

impl InMemorySecretProvider {
    pub fn insert(&mut self, reference: SecretRef, value: impl Into<String>) {
        self.values.insert(reference.0, value.into());
    }
}

impl SecretProvider for InMemorySecretProvider {
    fn resolve(&self, reference: &SecretRef) -> Option<SecretValue> {
        self.values.get(&reference.0).cloned().map(SecretValue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRef(pub String);

pub trait ArtifactStore {
    fn put(&self, name: &str, bytes: &[u8]) -> Result<ArtifactRef, String>;
    fn get(&self, reference: &ArtifactRef) -> Result<Vec<u8>, String>;
}

pub struct LocalArtifactStore {
    root: PathBuf,
    sequence: AtomicU64,
}

impl LocalArtifactStore {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, String> {
        fs::create_dir_all(root.as_ref()).map_err(|error| error.to_string())?;
        Ok(Self {
            root: root.as_ref().to_path_buf(),
            sequence: AtomicU64::new(1),
        })
    }

    fn safe_name(name: &str) -> String {
        name.chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                    ch
                } else {
                    '_'
                }
            })
            .collect()
    }
}

impl ArtifactStore for LocalArtifactStore {
    fn put(&self, name: &str, bytes: &[u8]) -> Result<ArtifactRef, String> {
        let id = self.sequence.fetch_add(1, Ordering::SeqCst);
        let filename = format!("{id:016x}-{}", Self::safe_name(name));
        fs::write(self.root.join(&filename), bytes).map_err(|error| error.to_string())?;
        Ok(ArtifactRef(filename))
    }

    fn get(&self, reference: &ArtifactRef) -> Result<Vec<u8>, String> {
        fs::read(self.root.join(&reference.0)).map_err(|error| error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRef(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    pub capability_id: String,
    pub state_ref: Option<ArtifactRef>,
    pub active: bool,
}

#[derive(Default)]
pub struct InMemorySessionManager {
    sessions: BTreeMap<String, SessionRecord>,
    next_id: u64,
}

impl InMemorySessionManager {
    pub fn create(&mut self, capability_id: impl Into<String>) -> SessionRef {
        self.next_id += 1;
        let id = format!("session-{:016x}", self.next_id);
        self.sessions.insert(
            id.clone(),
            SessionRecord {
                capability_id: capability_id.into(),
                state_ref: None,
                active: true,
            },
        );
        SessionRef(id)
    }

    pub fn get(&self, reference: &SessionRef) -> Option<&SessionRecord> {
        self.sessions.get(&reference.0)
    }

    pub fn attach_state(
        &mut self,
        reference: &SessionRef,
        state_ref: ArtifactRef,
    ) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(&reference.0)
            .ok_or_else(|| "unknown session".to_string())?;
        session.state_ref = Some(state_ref);
        Ok(())
    }

    pub fn close(&mut self, reference: &SessionRef) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(&reference.0)
            .ok_or_else(|| "unknown session".to_string())?;
        session.active = false;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyContext {
    pub granted_scopes: BTreeSet<String>,
    pub denied_capabilities: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub reason: String,
}

pub trait PolicyEngine {
    fn evaluate(
        &self,
        context: &PolicyContext,
        capability: &CapabilityDescriptor,
    ) -> PolicyDecision;
}

#[derive(Default)]
pub struct ScopePolicy;

impl PolicyEngine for ScopePolicy {
    fn evaluate(
        &self,
        context: &PolicyContext,
        capability: &CapabilityDescriptor,
    ) -> PolicyDecision {
        if context.denied_capabilities.contains(&capability.id) {
            return PolicyDecision {
                allowed: false,
                reason: "capability_denied".to_string(),
            };
        }
        if let Some(missing) = capability
            .permissions
            .iter()
            .find(|scope| !context.granted_scopes.contains(*scope))
        {
            return PolicyDecision {
                allowed: false,
                reason: format!("missing_scope:{missing}"),
            };
        }
        PolicyDecision {
            allowed: true,
            reason: "allowed".to_string(),
        }
    }
}
