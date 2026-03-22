//! # agentcontract
//!
//! Rust implementation of the [AgentContract specification](https://github.com/agentcontract/spec).
//!
//! Behavioral contracts for AI agents — declare what an agent must, must not,
//! and can do, enforced on every run.
//!
//! ## Quick Start
//!
//! ```no_run
//! use agentcontract::{load_contract, ContractRunner, RunContext};
//!
//! let contract = load_contract("my-agent.contract.yaml").unwrap();
//! let runner = ContractRunner::new(contract);
//!
//! let ctx = RunContext {
//!     input: "What are the GxP requirements?".into(),
//!     output: "Per 21 CFR Part 211...".into(),
//!     duration_ms: 1200.0,
//!     ..Default::default()
//! };
//!
//! let result = runner.run(&ctx);
//! if !result.passed {
//!     for v in result.blocking_violations() {
//!         eprintln!("[{}] {}: {}", v.action_taken, v.clause_name, v.details);
//!     }
//! }
//! ```

pub mod audit;
pub mod errors;
pub mod loader;
pub mod models;
pub mod runner;
pub mod validators;

// Convenience re-exports
pub use audit::AuditWriter;
pub use errors::{ContractError, ContractViolation};
pub use loader::load_contract;
pub use models::Contract;
pub use runner::{ContractRunner, RunResult, ViolationRecord};
pub use validators::RunContext;
