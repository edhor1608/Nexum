use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub capsule_id: String,
    pub step_count: u32,
    pub duration_ms: u64,
    pub attention_priority: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParityReport {
    pub capsule_id: String,
    pub matches: bool,
    pub parity_score: f64,
    pub mismatches: Vec<String>,
}

pub fn compare_execution(primary: &ExecutionResult, candidate: &ExecutionResult) -> ParityReport {
    let mut mismatches = Vec::new();
    let mut passed_checks: f64 = 0.0;
    let total_checks: f64 = 4.0;

    if primary.capsule_id == candidate.capsule_id {
        passed_checks += 1.0;
    } else {
        mismatches.push(format!(
            "capsule_id mismatch: primary={}, candidate={}",
            primary.capsule_id, candidate.capsule_id
        ));
    }

    if primary.step_count == candidate.step_count {
        passed_checks += 1.0;
    } else {
        mismatches.push(format!(
            "step_count mismatch: primary={}, candidate={}",
            primary.step_count, candidate.step_count
        ));
    }

    let duration_delta = primary.duration_ms.abs_diff(candidate.duration_ms);
    if duration_delta <= 500 {
        passed_checks += 1.0;
    } else {
        mismatches.push(format!(
            "duration_ms mismatch: primary={}, candidate={}, delta={}",
            primary.duration_ms, candidate.duration_ms, duration_delta
        ));
    }

    if primary.attention_priority == candidate.attention_priority {
        passed_checks += 1.0;
    } else {
        mismatches.push(format!(
            "attention_priority mismatch: primary={}, candidate={}",
            primary.attention_priority, candidate.attention_priority
        ));
    }

    let parity_score = (passed_checks / total_checks * 1000.0).round() / 1000.0;

    ParityReport {
        capsule_id: primary.capsule_id.clone(),
        matches: mismatches.is_empty(),
        parity_score,
        mismatches,
    }
}
