//! Validator modules — pattern, latency, cost.

pub mod pattern;
pub mod latency;
pub mod cost;

/// Everything a validator knows about a single agent run.
#[derive(Debug, Clone, Default)]
pub struct RunContext {
    pub input: String,
    pub output: String,
    pub duration_ms: f64,
    pub cost_usd: f64,
    pub steps: u32,
}

/// Result of a single validation check.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub passed: bool,
    pub clause_name: String,
    pub clause_text: String,
    pub clause_type: String,
    pub judge: String,
    pub details: String,
}

impl ValidationResult {
    pub fn pass(clause_name: &str, clause_text: &str, clause_type: &str) -> Self {
        ValidationResult {
            passed: true,
            clause_name: clause_name.into(),
            clause_text: clause_text.into(),
            clause_type: clause_type.into(),
            judge: "deterministic".into(),
            details: String::new(),
        }
    }

    pub fn fail(clause_name: &str, clause_text: &str, clause_type: &str, details: &str) -> Self {
        ValidationResult {
            passed: false,
            clause_name: clause_name.into(),
            clause_text: clause_text.into(),
            clause_type: clause_type.into(),
            judge: "deterministic".into(),
            details: details.into(),
        }
    }
}
