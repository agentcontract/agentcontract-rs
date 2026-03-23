//! agentcontract CLI — check, info, validate commands.

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use agentcontract::{load_contract, ContractRunner, RunContext};

#[derive(Parser)]
#[command(name = "agentcontract", version = env!("CARGO_PKG_VERSION"))]
#[command(about = "AgentContract CLI — behavioral contracts for AI agents")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a contract file parses and is schema-compliant
    Check {
        /// Path to the contract YAML/JSON file
        contract: PathBuf,
    },
    /// Show contract metadata
    Info {
        /// Path to the contract YAML/JSON file
        contract: PathBuf,
    },
    /// Run a contract against a JSONL file of RunContexts
    Validate {
        /// Path to the contract YAML/JSON file
        contract: PathBuf,
        /// Path to a JSONL file of run contexts
        runs: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { contract } => cmd_check(&contract),
        Commands::Info { contract } => cmd_info(&contract),
        Commands::Validate { contract, runs } => cmd_validate(&contract, &runs),
    }
}

fn cmd_check(path: &PathBuf) {
    match load_contract(path) {
        Ok(c) => {
            let n_limits = c.limits.max_latency_ms.is_some() as usize
                + c.limits.max_cost_usd.is_some() as usize
                + c.limits.max_tokens.is_some() as usize;
            println!("✓ Contract valid: {} v{}", c.agent, c.version);
            println!("  {} assertions, {} limits", c.assert_.len(), n_limits);
        }
        Err(e) => {
            eprintln!("✗ {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_info(path: &PathBuf) {
    match load_contract(path) {
        Ok(c) => {
            println!("Agent        : {}", c.agent);
            println!("Version      : {}", c.version);
            println!("Spec version : {}", c.spec_version);
            if !c.description.is_empty() {
                println!("Description  : {}", c.description);
            }
            println!("On violation : {}", c.on_violation.default);
            if !c.assert_.is_empty() {
                println!("Assertions ({}):", c.assert_.len());
                for a in &c.assert_ {
                    println!("  [{:?}] {} — {}", a.assertion_type, a.name, a.description);
                }
            }
            if let Some(max_ms) = c.limits.max_latency_ms {
                println!("Limit: max_latency_ms = {max_ms}");
            }
            if let Some(max_usd) = c.limits.max_cost_usd {
                println!("Limit: max_cost_usd = ${max_usd:.4}");
            }
        }
        Err(e) => {
            eprintln!("✗ {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_validate(contract_path: &PathBuf, runs_path: &PathBuf) {
    let contract = match load_contract(contract_path) {
        Ok(c) => c,
        Err(e) => { eprintln!("✗ {e}"); std::process::exit(1); }
    };
    let runner = ContractRunner::new(contract);

    let content = match std::fs::read_to_string(runs_path) {
        Ok(c) => c,
        Err(e) => { eprintln!("✗ Cannot read runs file: {e}"); std::process::exit(1); }
    };

    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;

    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() { continue; }

        let ctx: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => { eprintln!("Line {}: parse error: {e}", i + 1); continue; }
        };

        let run_ctx = RunContext {
            input: ctx["input"].as_str().unwrap_or("").into(),
            output: ctx["output"].as_str().unwrap_or("").into(),
            duration_ms: ctx["duration_ms"].as_f64().unwrap_or(0.0),
            cost_usd: ctx["cost_usd"].as_f64().unwrap_or(0.0),
            ..Default::default()
        };

        let result = runner.run(&run_ctx);
        total += 1;
        if result.passed {
            passed += 1;
            println!("  ✓ run {} — pass", i + 1);
        } else {
            failed += 1;
            println!("  ✗ run {} — {} violation(s)", i + 1, result.violations.len());
            for v in &result.violations {
                println!("    [{}] {}: {}", v.action_taken, v.clause_name, v.details);
            }
        }
    }

    println!("\n{total} runs: {passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}
