//! Serde models for the AgentContract specification.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationAction {
    Warn,
    Block,
    Rollback,
    HaltAndAlert,
}

impl Default for ViolationAction {
    fn default() -> Self {
        ViolationAction::Block
    }
}

impl std::fmt::Display for ViolationAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViolationAction::Warn => write!(f, "warn"),
            ViolationAction::Block => write!(f, "block"),
            ViolationAction::Rollback => write!(f, "rollback"),
            ViolationAction::HaltAndAlert => write!(f, "halt_and_alert"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssertionType {
    Pattern,
    Schema,
    Llm,
    Cost,
    Latency,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JudgeType {
    Deterministic,
    Llm,
}

impl Default for JudgeType {
    fn default() -> Self {
        JudgeType::Deterministic
    }
}

/// A clause is either a plain string or an object with judge annotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Clause {
    Simple(String),
    Object(ClauseObject),
}

impl Clause {
    pub fn text(&self) -> &str {
        match self {
            Clause::Simple(s) => s,
            Clause::Object(o) => &o.text,
        }
    }

    pub fn judge(&self) -> &JudgeType {
        match self {
            Clause::Simple(_) => &JudgeType::Deterministic,
            Clause::Object(o) => &o.judge,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClauseObject {
    pub text: String,
    #[serde(default)]
    pub judge: JudgeType,
    #[serde(default)]
    pub description: String,
}

/// Named, typed assertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assertion {
    pub name: String,
    #[serde(rename = "type")]
    pub assertion_type: AssertionType,
    #[serde(default)]
    pub description: String,

    // pattern
    pub must_not_match: Option<String>,
    pub must_match: Option<String>,

    // cost
    pub max_usd: Option<f64>,

    // latency
    pub max_ms: Option<u64>,

    // llm
    pub prompt: Option<String>,
    pub pass_when: Option<String>,
    pub model: Option<String>,
}

/// Quantitative hard limits.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Limits {
    pub max_tokens: Option<u64>,
    pub max_input_tokens: Option<u64>,
    pub max_latency_ms: Option<u64>,
    pub max_cost_usd: Option<f64>,
    pub max_tool_calls: Option<u64>,
    pub max_steps: Option<u64>,
}

/// Violation handlers — default + per-assertion overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnViolation {
    #[serde(default)]
    pub default: ViolationAction,
    #[serde(flatten)]
    pub overrides: HashMap<String, ViolationAction>,
}

impl Default for OnViolation {
    fn default() -> Self {
        OnViolation {
            default: ViolationAction::Block,
            overrides: HashMap::new(),
        }
    }
}

impl OnViolation {
    pub fn action_for(&self, name: &str) -> &ViolationAction {
        self.overrides.get(name).unwrap_or(&self.default)
    }
}

/// Root AgentContract model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub agent: String,
    #[serde(rename = "spec-version")]
    pub spec_version: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub must: Vec<Clause>,
    #[serde(default)]
    pub must_not: Vec<Clause>,
    #[serde(default)]
    pub can: Vec<String>,
    #[serde(default)]
    pub ensures: Vec<Clause>,
    #[serde(default)]
    pub invariant: Vec<Clause>,
    #[serde(rename = "assert", default)]
    pub assert_: Vec<Assertion>,
    #[serde(default)]
    pub limits: Limits,
    #[serde(default)]
    pub on_violation: OnViolation,
}
