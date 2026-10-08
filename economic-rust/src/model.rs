#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opportunity {
    pub opportunity_id: String,
    pub mission_id: String,
    pub reward_cents: i64,
    pub estimated_cost_cents: i64,
    pub success_probability_bps: u16,
    pub expected_duration_ms: u64,
    pub deadline_ms: i64,
    pub required_slots: usize,
    pub resource_locks: Vec<String>,
    pub eligible: bool,
    pub qualified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EconomicPolicy {
    pub now_ms: i64,
    pub minimum_remaining_ms: u64,
    pub minimum_expected_profit_cents: i64,
    pub minimum_margin_bps: u16,
    pub maximum_cost_per_mission_cents: i64,
    pub maximum_daily_cost_cents: i64,
    pub stop_loss_cents: i64,
    pub max_concurrency: usize,
    pub available_slots: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    Ineligible,
    NotQualified,
    Invalid,
    Deadline,
    CostCap,
    DailyBudget,
    StopLoss,
    NegativeValue,
    Margin,
    Slots,
    ResourceLock,
    Duplicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateScore {
    pub opportunity_id: String,
    pub mission_id: String,
    pub expected_revenue_cents: i64,
    pub expected_profit_cents: i64,
    pub margin_bps: i64,
    pub profit_per_hour_cents: i64,
    pub expected_duration_ms: u64,
    pub required_slots: usize,
    pub resource_locks: Vec<String>,
    pub estimated_cost_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedCandidate {
    pub opportunity_id: String,
    pub reason: Rejection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulePlan {
    pub selected: Vec<CandidateScore>,
    pub rejected: Vec<RejectedCandidate>,
    pub allocated_slots: usize,
    pub reserved_cost_cents: i64,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    VerifiedEarning,
    PaymentReceived,
    Cost,
}

impl EntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VerifiedEarning => "verified_earning",
            Self::PaymentReceived => "payment_received",
            Self::Cost => "cost",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EconomicEntry {
    pub event_key: String,
    pub mission_id: String,
    pub kind: EntryKind,
    pub amount_cents: i64,
    pub source_ref: String,
    pub recorded_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordOutcome {
    Recorded,
    Duplicate,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IncomeReport {
    pub verified_earnings_cents: i64,
    pub received_payments_cents: i64,
    pub costs_cents: i64,
    pub accrued_profit_cents: i64,
    pub cash_profit_cents: i64,
    pub accounts_receivable_cents: i64,
    pub entries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuccessHistory {
    pub completed: usize,
    pub succeeded: usize,
    pub probability_bps: u16,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DispatchResources {
    pub reserved_prior_cents: i64,
    pub spent_today_cents: i64,
    pub busy_slots: usize,
    pub busy_resource_locks: Vec<String>,
}
