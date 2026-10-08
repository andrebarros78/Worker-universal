use crate::model::{
    CandidateScore, EconomicPolicy, IncomeReport, Opportunity, RejectedCandidate, Rejection,
    SchedulePlan, SuccessHistory,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub fn observed_success(completed: usize, succeeded: usize) -> Result<SuccessHistory, String> {
    if succeeded > completed {
        return Err("succeeded cannot exceed completed".to_string());
    }
    // No observations => unknown, not assumed 100% success.
    let probability_bps = if completed == 0 {
        0
    } else {
        (succeeded as u128 * 10_000 / completed as u128) as u16
    };
    Ok(SuccessHistory {
        completed,
        succeeded,
        probability_bps,
    })
}

pub fn score(opportunity: &Opportunity) -> Result<CandidateScore, Rejection> {
    if opportunity.opportunity_id.trim().is_empty()
        || opportunity.mission_id.trim().is_empty()
        || opportunity.reward_cents < 0
        || opportunity.estimated_cost_cents < 0
        || opportunity.success_probability_bps > 10_000
        || opportunity.expected_duration_ms == 0
        || opportunity.required_slots == 0
    {
        return Err(Rejection::Invalid);
    }
    if !opportunity.eligible {
        return Err(Rejection::Ineligible);
    }
    if !opportunity.qualified {
        return Err(Rejection::NotQualified);
    }

    let expected_revenue = i128::from(opportunity.reward_cents)
        .checked_mul(i128::from(opportunity.success_probability_bps))
        .ok_or(Rejection::Invalid)?
        / 10_000;
    let expected_profit = expected_revenue - i128::from(opportunity.estimated_cost_cents);
    if expected_revenue > i128::from(i64::MAX)
        || expected_profit < i128::from(i64::MIN)
        || expected_profit > i128::from(i64::MAX)
    {
        return Err(Rejection::Invalid);
    }
    let margin = if expected_revenue > 0 {
        expected_profit * 10_000 / expected_revenue
    } else {
        -10_000
    };
    // cents/hour truncated conservatively; no floating-point money arithmetic.
    let per_hour = expected_profit
        .checked_mul(3_600_000)
        .ok_or(Rejection::Invalid)?
        / i128::from(opportunity.expected_duration_ms);
    if margin < i128::from(i64::MIN)
        || margin > i128::from(i64::MAX)
        || per_hour < i128::from(i64::MIN)
        || per_hour > i128::from(i64::MAX)
    {
        return Err(Rejection::Invalid);
    }
    Ok(CandidateScore {
        opportunity_id: opportunity.opportunity_id.clone(),
        mission_id: opportunity.mission_id.clone(),
        expected_revenue_cents: expected_revenue as i64,
        expected_profit_cents: expected_profit as i64,
        margin_bps: margin as i64,
        profit_per_hour_cents: per_hour as i64,
        expected_duration_ms: opportunity.expected_duration_ms,
        required_slots: opportunity.required_slots,
        resource_locks: opportunity.resource_locks.clone(),
        estimated_cost_cents: opportunity.estimated_cost_cents,
    })
}

pub fn schedule(
    opportunities: &[Opportunity],
    policy: &EconomicPolicy,
    report: &IncomeReport,
    reserved_prior_cents: i64,
    spent_today_cents: i64,
    busy_slots: usize,
    busy_resource_locks: &[String],
) -> SchedulePlan {
    let mut rejections = Vec::<RejectedCandidate>::new();
    let mut viable = Vec::<CandidateScore>::new();
    let mut seen_ids = BTreeSet::<String>::new();
    let mut seen_missions = BTreeSet::<String>::new();

    for op in opportunities {
        if !seen_ids.insert(op.opportunity_id.clone())
            || !seen_missions.insert(op.mission_id.clone())
        {
            rejections.push(RejectedCandidate {
                opportunity_id: op.opportunity_id.clone(),
                reason: Rejection::Duplicate,
            });
            continue;
        }
        let computed = match score(op) {
            Ok(c) => c,
            Err(reason) => {
                rejections.push(RejectedCandidate {
                    opportunity_id: op.opportunity_id.clone(),
                    reason,
                });
                continue;
            }
        };
        let remaining = op.deadline_ms.saturating_sub(policy.now_ms);
        let duration = i64::try_from(op.expected_duration_ms).unwrap_or(i64::MAX);
        let window = i64::try_from(policy.minimum_remaining_ms).unwrap_or(i64::MAX);
        let reason = if policy.max_concurrency == 0
            || policy.available_slots == 0
            || policy.minimum_expected_profit_cents < 0
            || policy.maximum_cost_per_mission_cents < 0
            || policy.maximum_daily_cost_cents < 0
            || reserved_prior_cents < 0
            || spent_today_cents < 0
            || busy_slots > policy.max_concurrency
        {
            Some(Rejection::Invalid)
        } else if remaining < duration.saturating_add(window) {
            Some(Rejection::Deadline)
        } else if op.estimated_cost_cents > policy.maximum_cost_per_mission_cents {
            Some(Rejection::CostCap)
        } else if report.entries != 0
            && report.accrued_profit_cents < 0
            && -i128::from(report.accrued_profit_cents) >= i128::from(policy.stop_loss_cents.max(0))
        {
            Some(Rejection::StopLoss)
        } else if computed.expected_profit_cents <= 0
            || computed.expected_profit_cents < policy.minimum_expected_profit_cents
        {
            Some(Rejection::NegativeValue)
        } else if computed.margin_bps < i64::from(policy.minimum_margin_bps) {
            Some(Rejection::Margin)
        } else {
            None
        };
        if let Some(reason) = reason {
            rejections.push(RejectedCandidate {
                opportunity_id: op.opportunity_id.clone(),
                reason,
            });
        } else {
            viable.push(computed);
        }
    }

    viable.sort_by(|a, b| {
        b.profit_per_hour_cents
            .cmp(&a.profit_per_hour_cents)
            .then_with(|| b.expected_profit_cents.cmp(&a.expected_profit_cents))
            .then_with(|| a.expected_duration_ms.cmp(&b.expected_duration_ms))
            .then_with(|| a.opportunity_id.cmp(&b.opportunity_id))
    });

    let mut selected = Vec::new();
    let mut used = busy_slots;
    let mut budget = reserved_prior_cents;
    let mut locked = busy_resource_locks.iter().cloned().collect::<BTreeSet<_>>();
    let total_slots = policy.max_concurrency.min(policy.available_slots);
    for candidate in viable {
        let reject = if candidate.required_slots > total_slots.saturating_sub(used) {
            Some(Rejection::Slots)
        } else if candidate
            .resource_locks
            .iter()
            .any(|lock| locked.contains(lock))
        {
            Some(Rejection::ResourceLock)
        } else {
            let cap = i128::from(policy.maximum_daily_cost_cents);
            let spent = i128::from(spent_today_cents)
                + i128::from(budget)
                + i128::from(candidate.estimated_cost_cents);
            if spent > cap {
                Some(Rejection::DailyBudget)
            } else {
                None
            }
        };
        if let Some(reason) = reject {
            rejections.push(RejectedCandidate {
                opportunity_id: candidate.opportunity_id,
                reason,
            });
            continue;
        }
        used += candidate.required_slots;
        budget += candidate.estimated_cost_cents;
        locked.extend(candidate.resource_locks.iter().cloned());
        selected.push(candidate);
    }

    rejections.sort_by(|a, b| {
        a.opportunity_id
            .cmp(&b.opportunity_id)
            .then_with(|| format!("{:?}", a.reason).cmp(&format!("{:?}", b.reason)))
    });
    let canonical = format!(
        "selected={}|rejected={}|slots={used}|budget={budget}|day_spent={spent_today_cents}",
        selected
            .iter()
            .map(|x| format!(
                "{}:{}:{}:{}",
                x.opportunity_id, x.mission_id, x.expected_profit_cents, x.profit_per_hour_cents
            ))
            .collect::<Vec<_>>()
            .join(";"),
        rejections
            .iter()
            .map(|x| format!("{}:{:?}", x.opportunity_id, x.reason))
            .collect::<Vec<_>>()
            .join(";")
    );
    let digest = Sha256::digest(canonical.as_bytes());
    let fingerprint = digest.iter().map(|b| format!("{b:02x}")).collect();
    SchedulePlan {
        selected,
        rejected: rejections,
        allocated_slots: used - busy_slots,
        reserved_cost_cents: budget - reserved_prior_cents,
        fingerprint,
    }
}
