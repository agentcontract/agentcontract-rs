//! Integration tests — mirrors the Python test suite structure.

use agentcontract::{ContractRunner, RunContext};

fn make_runner() -> ContractRunner {
    // Inline minimal contract for tests that don't need a file
    let yaml = r#"
agent: test-agent
spec-version: "0.1.0"
version: "1.0.0"
assert:
  - name: no_pii_leak
    type: pattern
    must_not_match: '\b\d{3}-\d{2}-\d{4}\b'
    description: No US Social Security Numbers in output
  - name: no_credentials
    type: pattern
    must_not_match: '(?i)(api_key|password|secret)\s*[:=]\s*\S+'
    description: No credentials in output
limits:
  max_latency_ms: 30000
on_violation:
  default: block
"#;
    let contract: agentcontract::Contract = serde_yaml::from_str(yaml).unwrap();
    ContractRunner::new(contract)
}

#[test]
fn test_clean_output_passes() {
    let runner = make_runner();
    let ctx = RunContext {
        input: "What are the cleaning validation requirements?".into(),
        output: "Per 21 CFR Part 211.67, equipment must be cleaned and maintained.".into(),
        duration_ms: 1200.0,
        ..Default::default()
    };
    let result = runner.run(&ctx);
    assert!(result.passed, "Expected pass, got violations: {:?}", result.violations);
    assert!(result.blocking_violations().is_empty());
}

#[test]
fn test_pii_pattern_triggers_violation() {
    let runner = make_runner();
    let ctx = RunContext {
        input: "Who processed the deviation?".into(),
        output: "Processed by employee SSN 123-45-6789 on 2024-01-15.".into(),
        duration_ms: 800.0,
        ..Default::default()
    };
    let result = runner.run(&ctx);
    assert!(!result.passed, "Expected violation but run passed");
    let names: Vec<&str> = result.violations.iter().map(|v| v.clause_name.as_str()).collect();
    assert!(names.contains(&"no_pii_leak"), "Expected no_pii_leak, got: {names:?}");
}

#[test]
fn test_pii_violation_is_blocking() {
    let runner = make_runner();
    let ctx = RunContext {
        input: "Any question".into(),
        output: "Patient SSN: 987-65-4321".into(),
        duration_ms: 500.0,
        ..Default::default()
    };
    let result = runner.run(&ctx);
    let blocking = result.blocking_violations();
    assert!(!blocking.is_empty(), "Expected a blocking violation for PII");
    assert_eq!(blocking[0].clause_name, "no_pii_leak");
    assert_eq!(blocking[0].action_taken, "block");
}

#[test]
fn test_credentials_pattern_triggers_violation() {
    let runner = make_runner();
    let ctx = RunContext {
        input: "What is the API configuration?".into(),
        output: "The service uses api_key=sk-abc123secret to authenticate.".into(),
        duration_ms: 600.0,
        ..Default::default()
    };
    let result = runner.run(&ctx);
    let names: Vec<&str> = result.violations.iter().map(|v| v.clause_name.as_str()).collect();
    assert!(names.contains(&"no_credentials"), "Expected no_credentials, got: {names:?}");
}

#[test]
fn test_latency_limit_violation() {
    let runner = make_runner();
    let ctx = RunContext {
        input: "question".into(),
        output: "answer".into(),
        duration_ms: 45_000.0,  // exceeds 30 000ms limit
        ..Default::default()
    };
    let result = runner.run(&ctx);
    let names: Vec<&str> = result.violations.iter().map(|v| v.clause_name.as_str()).collect();
    assert!(names.contains(&"max_latency_ms"), "Expected latency violation, got: {names:?}");
}

#[test]
fn test_inline_contract_parse() {
    let yaml = r#"
agent: my-agent
spec-version: "0.1.0"
version: "1.0.0"
assert:
  - name: no_pii
    type: pattern
    must_not_match: '\b\d{3}-\d{2}-\d{4}\b'
on_violation:
  default: block
"#;
    let contract: agentcontract::Contract = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(contract.agent, "my-agent");
    assert_eq!(contract.assert_.len(), 1);
}
