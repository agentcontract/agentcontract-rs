//! Error types for agentcontract-rs.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("Failed to load contract: {0}")]
    LoadError(String),

    #[error("Contract validation error: {0}")]
    ValidationError(String),
}

/// Raised when a blocking violation occurs.
#[derive(Debug, Error)]
#[error("ContractViolation: {message}")]
pub struct ContractViolation {
    pub message: String,
    pub violations: Vec<String>,
}
