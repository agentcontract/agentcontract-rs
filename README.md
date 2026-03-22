# agentcontract-rs

**Rust implementation of the [AgentContract specification](https://github.com/agentcontract/spec).**

[![Crates.io](https://img.shields.io/crates/v/agentcontract)](https://crates.io/crates/agentcontract)
[![Spec](https://img.shields.io/badge/spec-v0.1.0-orange)](https://github.com/agentcontract/spec/blob/main/SPEC.md)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-7%20passing-brightgreen)](#)

---

## Install

```toml
# Cargo.toml
[dependencies]
agentcontract = "0.1"
```

---

## Quickstart

**1. Write a contract:**

```yaml
# my-agent.contract.yaml
agent: my-agent
spec-version: "0.1.0"
version: "1.0.0"

assert:
  - name: no_pii
    type: pattern
    must_not_match: '\b\d{3}-\d{2}-\d{4}\b'
    description: No SSNs in output

limits:
  max_latency_ms: 10000
  max_cost_usd: 0.10

on_violation:
  default: block
```

**2. Enforce it in your agent:**

```rust
use agentcontract::{load_contract, ContractRunner, RunContext};

fn main() {
    let contract = load_contract("my-agent.contract.yaml").unwrap();
    let runner = ContractRunner::new(contract);

    let output = run_my_agent("What are the GxP requirements?");

    let ctx = RunContext {
        input: "What are the GxP requirements?".into(),
        output,
        duration_ms: 1200.0,
        ..Default::default()
    };

    let result = runner.run(&ctx);

    if !result.passed {
        for v in result.blocking_violations() {
            eprintln!("[{}] {}: {}", v.action_taken, v.clause_name, v.details);
        }
        std::process::exit(1);
    }

    println!("✓ Contract passed ({} assertions checked)", result.violations.len());
}
```

**3. When a violation occurs:**

```
[block] no_pii: Forbidden pattern found at position 42: '123-45-6789'
```

---

## CLI

```bash
# Check a contract file is valid
agentcontract check my-agent.contract.yaml

# Show contract metadata and assertions
agentcontract info my-agent.contract.yaml

# Validate a JSONL file of run contexts against a contract
agentcontract validate my-agent.contract.yaml runs.jsonl
```

`runs.jsonl` format (one JSON object per line):

```jsonl
{"input": "question", "output": "answer", "duration_ms": 1200, "cost_usd": 0.002}
{"input": "question2", "output": "SSN 123-45-6789", "duration_ms": 800, "cost_usd": 0.001}
```

---

## Validator Types

| Type | How it works | Field |
|------|-------------|-------|
| `pattern` | Regex on output | `must_not_match`, `must_match` |
| `latency` | `duration_ms` vs `max_ms` | `max_ms` |
| `cost` | `cost_usd` vs `max_usd` | `max_usd` |
| `llm` | LLM judge (roadmap) | — |

Limits (`max_latency_ms`, `max_cost_usd`, `max_tokens`) are checked automatically from the `limits:` block.

---

## Audit Trail

Every run can be written to a tamper-evident JSONL file with a SHA-256 content hash:

```rust
use agentcontract::AuditWriter;

let writer = AuditWriter::new("audit.jsonl");
writer.write(&result, "my-agent.contract.yaml").unwrap();
```

Each entry:
```json
{
  "run_id": "uuid",
  "timestamp": "2026-03-22T10:00:00Z",
  "agent": "my-agent",
  "outcome": "violation",
  "passed": false,
  "violations": [...],
  "entry_sha256": "abc123..."
}
```

---

## Architecture

```
agentcontract-rs/src/
├── lib.rs          # Public API and re-exports
├── main.rs         # CLI (check / info / validate)
├── models.rs       # Contract, Assertion, Limits, OnViolation (serde)
├── loader.rs       # load_contract() — YAML or JSON
├── runner.rs       # ContractRunner, RunResult, ViolationRecord
├── audit.rs        # AuditWriter — JSONL + SHA-256
└── validators/
    ├── mod.rs      # RunContext, ValidationResult
    ├── pattern.rs  # Regex must_not_match / must_match
    ├── latency.rs  # duration_ms check
    └── cost.rs     # cost_usd check
```

Evaluation order follows [spec §6.1](https://github.com/agentcontract/spec/blob/main/SPEC.md):
`limits → assert → must → must_not → ensures`

---

## Full Documentation

See the [AgentContract specification](https://github.com/agentcontract/spec/blob/main/SPEC.md).

**Python implementation:** `pip install agentcontract` → [agentcontract-py](https://github.com/agentcontract/agentcontract-py)

**TypeScript implementation:** `npm install @agentcontract/core` → [agentcontract-ts](https://github.com/agentcontract/agentcontract-ts)

---

## Roadmap

- [ ] LLM judge validator (optional `anthropic` feature flag)
- [ ] `rollback` violation action (snapshot/restore hook)
- [ ] `requires` and `invariant` clause evaluation
- [ ] HMAC-signed audit trail (mirrors Python implementation)
- [ ] PyO3 bindings — expose Rust validator core to Python for performance

---

## License

Apache 2.0 — *Part of the [AgentContract](https://github.com/agentcontract) open standard.*
