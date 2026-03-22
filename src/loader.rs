//! Contract loader — reads and parses YAML or JSON contract files.

use std::path::Path;
use crate::errors::ContractError;
use crate::models::Contract;

/// Load and parse a contract from a YAML or JSON file.
pub fn load_contract<P: AsRef<Path>>(path: P) -> Result<Contract, ContractError> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .map_err(|e| ContractError::LoadError(format!("Cannot read {}: {}", path.display(), e)))?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");
    let contract: Contract = match ext {
        "json" => serde_json::from_str(&content)
            .map_err(|e| ContractError::LoadError(format!("JSON parse error: {e}")))?,
        _ => serde_yaml::from_str(&content)
            .map_err(|e| ContractError::LoadError(format!("YAML parse error: {e}")))?,
    };

    Ok(contract)
}
