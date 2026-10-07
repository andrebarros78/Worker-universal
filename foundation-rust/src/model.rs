use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapabilityKind {
    Native,
    Api,
    Mcp,
    Browser,
    Computer,
    Vision,
    Document,
    Research,
    Llm,
    Storage,
    Notification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Native,
    Api,
    Mcp,
    Browser,
    Computer,
    Ai,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Unavailable,
    Starting,
    Healthy,
    Degraded,
    Failed,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReadinessState {
    Installed,
    Configured,
    Authenticated,
    Operational,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PromotionState {
    Experimental,
    Tested,
    Qualified,
    Production,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffectClass {
    None,
    Read,
    Write,
    Submit,
    ExternalEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdempotencyClass {
    Safe,
    Conditional,
    Unsafe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    NetworkFailure,
    SessionExpired,
    ElementMoved,
    LayoutChanged,
    RateLimit,
    WorkerCrash,
    ModelTimeout,
    LowConfidence,
    ValidationFailed,
    DuplicateRisk,
    DeadlineRisk,
    ExternalDependency,
    PermissionDenied,
    ContractMismatch,
    Cancelled,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StableError {
    pub class: ErrorClass,
    pub retryable: bool,
    pub message: String,
    pub detail_code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn compatible_with(self, required: Self) -> bool {
        self.major == required.major && self >= required
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Version {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split('.');
        let major = parts
            .next()
            .ok_or("missing major")?
            .parse()
            .map_err(|_| "invalid major")?;
        let minor = parts
            .next()
            .ok_or("missing minor")?
            .parse()
            .map_err(|_| "invalid minor")?;
        let patch = parts
            .next()
            .ok_or("missing patch")?
            .parse()
            .map_err(|_| "invalid patch")?;
        if parts.next().is_some() {
            return Err("too many version components");
        }
        Ok(Self::new(major, minor, patch))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDescriptor {
    pub id: String,
    pub kind: CapabilityKind,
    pub provider: Option<String>,
    pub contract_version: Version,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
    pub fallbacks: Vec<String>,
    pub lifecycle: LifecycleState,
    pub readiness: ReadinessState,
    pub promotion: PromotionState,
    pub side_effect: SideEffectClass,
    pub idempotency: IdempotencyClass,
    pub max_concurrency: u16,
    pub resource_locks: Vec<String>,
    pub cost_microunits: u64,
    pub p95_ms: u64,
    pub reliability_bps: u16,
    pub evidence_types: Vec<String>,
    pub transport: TransportKind,
    pub config_ref: Option<String>,
    pub enabled: bool,
}

impl CapabilityDescriptor {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.id.trim().is_empty() {
            return Err("capability id cannot be empty");
        }
        if self.max_concurrency == 0 {
            return Err("max_concurrency must be positive");
        }
        if self.reliability_bps > 10_000 {
            return Err("reliability_bps must be <= 10000");
        }
        if self.contract_version.major == 0 {
            return Err("contract major version must be non-zero");
        }
        if self.dependencies.iter().any(|id| id == &self.id) {
            return Err("capability cannot depend on itself");
        }
        Ok(())
    }

    pub fn operational(&self, allow_degraded: bool) -> bool {
        self.enabled
            && self.readiness == ReadinessState::Operational
            && matches!(
                self.lifecycle,
                LifecycleState::Healthy | LifecycleState::Degraded if allow_degraded
            )
            || self.enabled
                && self.readiness == ReadinessState::Operational
                && self.lifecycle == LifecycleState::Healthy
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveRequest {
    pub kind: Option<CapabilityKind>,
    pub required_contract: Version,
    pub required_permissions: Vec<String>,
    pub required_evidence: Vec<String>,
    pub minimum_promotion: PromotionState,
    pub minimum_reliability_bps: u16,
    pub maximum_cost_microunits: Option<u64>,
    pub maximum_p95_ms: Option<u64>,
    pub provider_preference: Vec<String>,
    pub allow_degraded: bool,
}

impl ResolveRequest {
    pub fn provider_rank(&self, descriptor: &CapabilityDescriptor) -> usize {
        descriptor
            .provider
            .as_ref()
            .and_then(|provider| {
                self.provider_preference
                    .iter()
                    .position(|item| item == provider)
            })
            .unwrap_or(self.provider_preference.len())
    }
}

pub fn compare_candidates(
    request: &ResolveRequest,
    left: &CapabilityDescriptor,
    right: &CapabilityDescriptor,
) -> Ordering {
    request
        .provider_rank(left)
        .cmp(&request.provider_rank(right))
        .then_with(|| right.reliability_bps.cmp(&left.reliability_bps))
        .then_with(|| left.cost_microunits.cmp(&right.cost_microunits))
        .then_with(|| left.p95_ms.cmp(&right.p95_ms))
        .then_with(|| left.id.cmp(&right.id))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySnapshotEntry {
    pub id: String,
    pub lifecycle: LifecycleState,
    pub readiness: ReadinessState,
    pub promotion: PromotionState,
    pub enabled: bool,
    pub contract_version: Version,
}
