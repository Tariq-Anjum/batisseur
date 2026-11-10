use serde_json::{json, Value};
use std::process::Command;
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_batisseur"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn frozen_benchmark_and_three_process_runs() {
    let first = run(&["bench", "--json"]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let metrics: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(metrics["total"], 16);
    assert_eq!(metrics["accuracy_percent"], 100.0);
    assert_eq!(metrics["correct_abstentions"], 4);
    assert_eq!(metrics["unexpected_activations"], 0);
    assert_eq!(metrics["deterministic_three_runs"], 16);
    assert_eq!(metrics["passed"], true);
    for _ in 0..2 {
        assert_eq!(first.stdout, run(&["bench", "--json"]).stdout);
    }
}
#[test]
fn lists_and_shows_embedded_packs() {
    let out = run(&["packs", "list"]);
    assert!(out.status.success());
    assert_eq!(String::from_utf8(out.stdout).unwrap().lines().count(), 8);
    let out = run(&["packs", "show", "android-tv"]);
    assert!(out.status.success());
    let pack: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(pack["id"], "android-tv");
    assert!(!run(&["packs", "show", "missing"]).status.success());
    assert!(!run(&["bench", "--garbage"]).status.success());
}
#[test]
fn file_routing_validation_and_error_streams() {
    let dir = std::path::Path::new("target").join(format!("cli-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("intent.json");
    let case: Value = serde_json::from_str(batisseur::BUILD_BENCH.lines().next().unwrap()).unwrap();
    let input = serde_json::to_vec(&case["intent"]).unwrap();
    std::fs::write(&file, &input).unwrap();
    let path = file.to_str().unwrap();
    let first = run(&["route", path, "--json"]);
    assert!(first.status.success());
    let r: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(r["selected_pack"], "android-tv");
    for _ in 0..2 {
        assert_eq!(first.stdout, run(&["route", path, "--json"]).stdout);
    }
    assert!(run(&["validate", path]).status.success());
    assert_eq!(std::fs::read(&file).unwrap(), input);
    std::fs::write(&file, b"{broken").unwrap();
    let bad = run(&["route", path, "--json"]);
    assert_eq!(bad.status.code(), Some(2));
    assert!(bad.stdout.is_empty());
    assert!(!bad.stderr.is_empty());
    std::fs::write(
        &file,
        serde_json::to_vec(&json!({"taxonomy_version":"tt-1.1.0","task_type":"implement"}))
            .unwrap(),
    )
    .unwrap();
    let partial = run(&["route", path, "--json"]);
    assert!(partial.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&partial.stdout).unwrap()["route_status"],
        "needs_information"
    );
    assert!(!run(&["validate", path]).status.success());
    std::fs::remove_dir_all(&dir).unwrap();
}
