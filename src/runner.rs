//! ContractRunner — evaluates a Contract against a RunContext.

use uuid::Uuid;
use crate::models::{AssertionType, Contract, JudgeType, ViolationAction};
use crate::validators::{RunContext, ValidationResult, pattern, latency, cost};

/// A single contract violation record.
#[derive(Debug, Clone)]
pub struct ViolationRecord {
    pub clause_type: String,
    pub clause_name: String,
    pub clause_text: String,
    pub severity: String,
    pub action_taken: String,
    pub judge: String,
    pub details: String,
}

/// Result of running a contract against a RunContext.
#[derive(Debug)]
pub struct RunResult {
    pub passed: bool,
    pub run_id: String,
    pub agent: String,
    pub contract_version: String,
    pub violations: Vec<ViolationRecord>,
}

impl RunResult {
    pub fn outcome(&self) -> &str {
        if self.passed { "pass" } else { "violation" }
    }

    pub fn blocking_violations(&self) -> Vec<&ViolationRecord> {
        self.violations.iter().filter(|v| {
            matches!(v.action_taken.as_str(), "block" | "rollback" | "halt_and_alert")
        }).collect()
    }
}

/// Evaluates a Contract against a RunContext in spec order.
pub struct ContractRunner {
    pub contract: Contract,
}

impl ContractRunner {
    pub fn new(contract: Contract) -> Self {
        ContractRunner { contract }
    }

    pub fn run(&self, ctx: &RunContext) -> RunResult {
        let run_id = Uuid::new_v4().to_string();
        let mut violations: Vec<ViolationRecord> = Vec::new();
        let c = &self.contract;
        let ov = &c.on_violation;

        // 1. Limits
        violations.extend(self.check_limits(ctx));

        // 2. assert (typed assertions)
        for assertion in &c.assert_ {
            let result = self.run_assertion(assertion, ctx);
            if !result.passed {
                let action = ov.action_for(&assertion.name);
                violations.push(ViolationRecord {
                    clause_type: "assert".into(),
                    clause_name: assertion.name.clone(),
                    clause_text: result.clause_text.clone(),
                    severity: action.to_string(),
                    action_taken: action.to_string(),
                    judge: result.judge.clone(),
                    details: result.details.clone(),
                });
            }
        }

        // 3. must
        for clause in &c.must {
            let text = clause.text();
            if clause.judge() == &JudgeType::Deterministic {
                // Deterministic natural language: pass by default (no handler)
                continue;
            }
            let key = format!("must:{}", &text[..text.len().min(30)]);
            let action = ov.action_for(&key);
            violations.push(ViolationRecord {
                clause_type: "must".into(),
                clause_name: key,
                clause_text: text.into(),
                severity: action.to_string(),
                action_taken: action.to_string(),
                judge: "llm".into(),
                details: "LLM judge not supported in this build.".into(),
            });
        }

        // 4. must_not
        for clause in &c.must_not {
            let text = clause.text();
            if clause.judge() == &JudgeType::Deterministic {
                continue;
            }
            let key = format!("must_not:{}", &text[..text.len().min(30)]);
            let action = ov.action_for(&key);
            violations.push(ViolationRecord {
                clause_type: "must_not".into(),
                clause_name: key,
                clause_text: text.into(),
                severity: action.to_string(),
                action_taken: action.to_string(),
                judge: "llm".into(),
                details: "LLM judge not supported in this build.".into(),
            });
        }

        let passed = !violations.iter().any(|v| {
            matches!(v.action_taken.as_str(), "block" | "rollback" | "halt_and_alert")
        });

        RunResult {
            passed,
            run_id,
            agent: c.agent.clone(),
            contract_version: c.version.clone(),
            violations,
        }
    }

    fn check_limits(&self, ctx: &RunContext) -> Vec<ViolationRecord> {
        let mut records = Vec::new();
        let limits = &self.contract.limits;
        let ov = &self.contract.on_violation;

        if let Some(max_ms) = limits.max_latency_ms {
            let result = latency::validate(ctx, max_ms);
            if !result.passed {
                let action = ov.action_for("max_latency_ms");
                records.push(violation_from_result("limits", "max_latency_ms", action, &result));
            }
        }

        if let Some(max_usd) = limits.max_cost_usd {
            let result = cost::validate(ctx, max_usd);
            if !result.passed {
                let action = ov.action_for("max_cost_usd");
                records.push(violation_from_result("limits", "max_cost_usd", action, &result));
            }
        }

        if let Some(max_tokens) = limits.max_tokens {
            let estimated = ctx.output.len() as u64 / 4;
            if estimated > max_tokens {
                let action = ov.action_for("max_tokens");
                records.push(ViolationRecord {
                    clause_type: "limits".into(),
                    clause_name: "max_tokens".into(),
                    clause_text: format!("output must not exceed {max_tokens} tokens"),
                    severity: action.to_string(),
                    action_taken: action.to_string(),
                    judge: "deterministic".into(),
                    details: format!("Estimated {estimated} tokens exceeds limit of {max_tokens}"),
                });
            }
        }

        records
    }

    fn run_assertion(&self, assertion: &crate::models::Assertion, ctx: &RunContext) -> ValidationResult {
        match assertion.assertion_type {
            AssertionType::Pattern => pattern::validate(
                ctx,
                &assertion.name,
                assertion.must_not_match.as_deref(),
                assertion.must_match.as_deref(),
                &assertion.description,
            ),
            AssertionType::Latency => {
                let max_ms = assertion.max_ms.unwrap_or(0);
                latency::validate(ctx, max_ms)
            }
            AssertionType::Cost => {
                let max_usd = assertion.max_usd.unwrap_or(0.0);
                cost::validate(ctx, max_usd)
            }
            _ => ValidationResult {
                passed: false,
                clause_name: assertion.name.clone(),
                clause_text: if assertion.description.is_empty() { assertion.name.clone() } else { assertion.description.clone() },
                clause_type: "assert".into(),
                judge: "deterministic".into(),
                details: format!("Unsupported assertion type in this build: {:?}", assertion.assertion_type),
            },
        }
    }
}

fn violation_from_result(
    clause_type: &str,
    clause_name: &str,
    action: &ViolationAction,
    result: &ValidationResult,
) -> ViolationRecord {
    ViolationRecord {
        clause_type: clause_type.into(),
        clause_name: clause_name.into(),
        clause_text: result.clause_text.clone(),
        severity: action.to_string(),
        action_taken: action.to_string(),
        judge: result.judge.clone(),
        details: result.details.clone(),
    }
}
