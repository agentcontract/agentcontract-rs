//! Append-only JSONL audit writer with SHA-256 entry hashing.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde_json::json;
use sha2::{Sha256, Digest};

use crate::runner::RunResult;

pub struct AuditWriter {
    path: PathBuf,
}

impl AuditWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        AuditWriter { path: path.as_ref().to_path_buf() }
    }

    /// Append a run result as a JSONL entry with a SHA-256 content hash.
    pub fn write(&self, result: &RunResult, contract_path: &str) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let violations: Vec<serde_json::Value> = result.violations.iter().map(|v| {
            json!({
                "clause_type": v.clause_type,
                "clause_name": v.clause_name,
                "clause_text": v.clause_text,
                "action_taken": v.action_taken,
                "details": v.details,
            })
        }).collect();

        let entry = json!({
            "run_id": result.run_id,
            "timestamp": Utc::now().to_rfc3339(),
            "agent": result.agent,
            "contract_version": result.contract_version,
            "contract_path": contract_path,
            "outcome": result.outcome(),
            "passed": result.passed,
            "violations": violations,
        });

        let entry_str = serde_json::to_string(&entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // SHA-256 of the serialised entry for tamper-evidence
        let hash = hex::encode(Sha256::digest(entry_str.as_bytes()));

        let mut signed = serde_json::from_str::<serde_json::Value>(&entry_str)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        signed["entry_sha256"] = json!(hash);

        let line = serde_json::to_string(&signed)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        writeln!(file, "{line}")
    }
}
