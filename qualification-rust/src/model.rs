use tma_foundation::model::PromotionState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Training,
    Benchmark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationState {
    PreRegistration,
    Registered,
    PostRegistration,
    ProductionReady,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCase {
    pub case_id: String,
    pub dataset_id: String,
    pub dataset_version: String,
    pub seed: u64,
    pub deadline_ms: u64,
    pub expected_accuracy_bps: u16,
    pub should_recover: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkSample {
    pub case_id: String,
    pub success: bool,
    pub accuracy_bps: u16,
    pub validation_proven: bool,
    pub latency_ms: u64,
    pub recovered: bool,
    pub timed_out: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationThresholds {
    pub minimum_runs: usize,
    pub minimum_success_bps: u16,
    pub minimum_accuracy_bps: u16,
    pub minimum_validation_bps: u16,
    pub maximum_p95_ms: u64,
    pub require_recovery_drill: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scorecard {
    pub capability_id: String,
    pub mode: RunMode,
    pub dataset_id: String,
    pub dataset_version: String,
    pub seed: u64,
    pub runs: usize,
    pub success_bps: u16,
    pub accuracy_bps: u16,
    pub validation_bps: u16,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub recovered_runs: usize,
    pub recovery_passed: bool,
    pub passed: bool,
    pub failures: Vec<String>,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationReceipt {
    pub receipt_id: String,
    pub capability_id: String,
    pub from: PromotionState,
    pub to: PromotionState,
    pub registration_state: RegistrationState,
    pub scorecard_fingerprint: String,
    pub accepted: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationSession {
    pub capability_id: String,
    pub registration_state: RegistrationState,
    pub training_scorecard: Option<String>,
    pub benchmark_scorecard: Option<String>,
    pub post_registration_scorecard: Option<String>,
    pub recovery_drill_passed: bool,
}
