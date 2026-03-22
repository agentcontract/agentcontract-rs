//! Cost validator — checks cost_usd against max_usd.

use super::{RunContext, ValidationResult};

pub fn validate(ctx: &RunContext, max_usd: f64) -> ValidationResult {
    let clause_text = format!("cost must not exceed ${max_usd:.4} USD");
    if ctx.cost_usd > max_usd {
        ValidationResult::fail(
            "cost",
            &clause_text,
            "limits",
            &format!("Run cost ${:.6} exceeds limit of ${max_usd:.4}", ctx.cost_usd),
        )
    } else {
        ValidationResult::pass("cost", &clause_text, "limits")
    }
}
