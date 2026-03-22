//! Regex pattern validator — must_match / must_not_match.

use regex::Regex;
use super::{RunContext, ValidationResult};

pub fn validate(
    ctx: &RunContext,
    name: &str,
    must_not_match: Option<&str>,
    must_match: Option<&str>,
    description: &str,
) -> ValidationResult {
    let output = &ctx.output;

    if let Some(pattern) = must_not_match {
        match Regex::new(pattern) {
            Ok(re) => {
                if let Some(m) = re.find(output) {
                    let snippet: String = m.as_str().chars().take(50).collect();
                    let text = if description.is_empty() {
                        format!("must_not_match: {pattern}")
                    } else {
                        description.into()
                    };
                    return ValidationResult::fail(
                        name,
                        &text,
                        "assert",
                        &format!("Forbidden pattern found at position {}: '{snippet}'", m.start()),
                    );
                }
            }
            Err(e) => {
                return ValidationResult::fail(name, description, "assert", &format!("Invalid regex: {e}"));
            }
        }
    }

    if let Some(pattern) = must_match {
        match Regex::new(pattern) {
            Ok(re) => {
                if !re.is_match(output) {
                    let text = if description.is_empty() {
                        format!("must_match: {pattern}")
                    } else {
                        description.into()
                    };
                    return ValidationResult::fail(
                        name,
                        &text,
                        "assert",
                        "Required pattern not found in output.",
                    );
                }
            }
            Err(e) => {
                return ValidationResult::fail(name, description, "assert", &format!("Invalid regex: {e}"));
            }
        }
    }

    let text = if description.is_empty() { name } else { description };
    ValidationResult::pass(name, text, "assert")
}
