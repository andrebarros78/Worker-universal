use std::time::Instant;
use tma_validation::ValidationReport;

use crate::model::{BenchmarkCase, BenchmarkSample};

pub trait BenchmarkExecutor {
    fn execute(&mut self, case: &BenchmarkCase) -> BenchmarkSample;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SyntheticHarness;

impl SyntheticHarness {
    pub fn run(
        &self,
        cases: &[BenchmarkCase],
        executor: &mut dyn BenchmarkExecutor,
    ) -> Vec<BenchmarkSample> {
        cases.iter().map(|case| executor.execute(case)).collect()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WallClockHarness;

impl WallClockHarness {
    pub fn run_case<F>(&self, case: &BenchmarkCase, operation: F) -> BenchmarkSample
    where
        F: FnOnce() -> (bool, u16, bool, bool),
    {
        let started = Instant::now();
        let (success, accuracy_bps, validation_proven, recovered) = operation();
        let elapsed_ms = started.elapsed().as_millis() as u64;
        let timed_out = elapsed_ms > case.deadline_ms;
        BenchmarkSample {
            case_id: case.case_id.clone(),
            success: success && !timed_out,
            accuracy_bps,
            validation_proven: validation_proven && !timed_out,
            latency_ms: elapsed_ms,
            recovered,
            timed_out,
        }
    }
}

pub fn sample_from_validation(
    case: &BenchmarkCase,
    report: &ValidationReport,
    latency_ms: u64,
    accuracy_bps: u16,
    recovered: bool,
) -> BenchmarkSample {
    let timed_out = latency_ms > case.deadline_ms;
    BenchmarkSample {
        case_id: case.case_id.clone(),
        success: report.is_proven() && !timed_out,
        accuracy_bps,
        validation_proven: report.is_proven() && !timed_out,
        latency_ms,
        recovered,
        timed_out,
    }
}
