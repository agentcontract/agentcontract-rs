//! Latency validator — checks duration_ms against max_ms.

use super::{RunContext, ValidationResult};

pub fn validate(ctx: &RunContext, max_ms: u64) -> ValidationResult {
    let clause_text = format!("latency must not exceed {max_ms}ms");
    if ctx.duration_ms > max_ms as f64 {
        ValidationResult::fail(
            "latency",
            &clause_text,
            "limits",
            &format!("Response took {:.0}ms, limit is {max_ms}ms", ctx.duration_ms),
        )
    } else {
        ValidationResult::pass("latency", &clause_text, "limits")
    }
}
