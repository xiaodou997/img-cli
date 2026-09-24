use serde_json::Value;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yu"))
        .args(args)
        .output()
        .expect("yu should execute")
}

fn parse_stdout(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

#[test]
fn doctor_json_is_machine_readable() {
    let output = run(&["doctor", "--json"]);
    let json = parse_stdout(&output);

    assert_eq!(json["schema_version"], "1");
    assert_eq!(json["operation"], "runtime.doctor");
    assert_eq!(json["result"]["healthy"], true);
    assert_eq!(json["result"]["engines"]["ready"], 1);
}

#[test]
fn capabilities_json_contains_runtime_capabilities() {
    let output = run(&["capabilities", "--json"]);
    let json = parse_stdout(&output);
    let capabilities = json["result"].as_array().expect("result should be an array");

    assert!(capabilities.iter().any(|item| item["id"] == "runtime.doctor"));
    assert!(
        capabilities
            .iter()
            .any(|item| item["id"] == "runtime.capabilities")
    );
    assert!(capabilities.iter().any(|item| item["id"] == "engine.list"));
}

#[test]
fn engine_list_json_contains_built_in_runtime() {
    let output = run(&["engine", "list", "--json"]);
    let json = parse_stdout(&output);
    let engines = json["result"].as_array().expect("result should be an array");

    assert_eq!(engines.len(), 1);
    assert_eq!(engines[0]["id"], "yu-runtime");
    assert_eq!(engines[0]["provider"], "built_in");
    assert_eq!(engines[0]["state"], "ready");
}

#[test]
fn human_engine_list_is_readable() {
    let output = run(&["engine", "list"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("yu-runtime"));
    assert!(stdout.contains("built_in"));
    assert!(stdout.contains("ready"));
}
