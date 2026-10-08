use crate::model::IncomeReport;

/// Machine-readable financial dashboard. Amounts are BRL cents, never
/// floating-point. Unpaid accrual is distinct from cash actually recorded.
pub fn dashboard_json(report: &IncomeReport) -> String {
    format!(
        concat!(
            "{{",
            "\"currency\":\"BRL\",",
            "\"unit\":\"cents\",",
            "\"verified_earnings\":{},",
            "\"payments_recorded\":{},",
            "\"costs\":{},",
            "\"accrued_profit\":{},",
            "\"cash_profit\":{},",
            "\"accounts_receivable\":{},",
            "\"entry_count\":{}",
            "}}"
        ),
        report.verified_earnings_cents,
        report.received_payments_cents,
        report.costs_cents,
        report.accrued_profit_cents,
        report.cash_profit_cents,
        report.accounts_receivable_cents,
        report.entries
    )
}
