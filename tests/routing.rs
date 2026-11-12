use batisseur::route;
use serde_json::json;
#[test]
fn unclear_abstains() {
    let result = route(&json!({"taxonomy_version":"tt-1.1.0", "task_type":"unclear"})).unwrap();
    assert_eq!(result["route_status"], "needs_information");
    assert_eq!(result["capabilities"], json!([]));
}
#[test]
fn rejects_taxonomy_mismatch() {
    assert!(
        route(&json!({"taxonomy_version":"tt-2", "task_type":"implement"}))
            .unwrap_err()
            .contains("taxonomy")
    );
}

fn fixture(id: &str) -> serde_json::Value {
    include_str!("../bench/buildbench-v0.jsonl")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|case| case["id"] == id)
        .unwrap()["intent"]
        .clone()
}
#[test]
fn routes_with_explained_scores() {
    let result = route(&fixture("android-tv")).unwrap();
    assert_eq!(result["selected_pack"], "android-tv");
    let terms = result["candidates"][0]["terms"].as_array().unwrap();
    assert_eq!(terms.len(), 14);
    let sum: i64 = terms.iter().map(|t| t["value"].as_i64().unwrap()).sum();
    assert_eq!(result["candidates"][0]["score"], sum);
    assert!(terms
        .iter()
        .all(|t| !t["reason"].as_str().unwrap().is_empty()));
}
#[test]
fn interaction_collision_abstains() {
    let result = route(&fixture("touch-panel-collision")).unwrap();
    assert_eq!(result["route_status"], "needs_information");
    assert_eq!(result["capabilities"], json!([]));
    assert_eq!(
        result["missing_fields"],
        json!(["constraints.existing_stack"])
    );
}
#[test]
fn preserves_type_over_work_kind() {
    let mut i = fixture("android-tv");
    i["task_type"] = json!("design");
    i["work_kind"] = json!("bug");
    i["artifact"] = json!("interface");
    let r = route(&i).unwrap();
    assert_eq!(r["task_type"], "design");
    assert_eq!(r["families"], json!(["Experience"]));
    assert_eq!(r["plan"][0]["workflow"], "design");
    assert_eq!(r["plan"].as_array().unwrap().len(), 1);
}
#[test]
fn no_authority_is_inferred() {
    let mut i = fixture("cli-linux");
    i["authority"]["permitted_ops"] = json!(["read"]);
    let r = route(&i).unwrap();
    assert_eq!(r["risk_gate"]["status"], "requires_authorization");
    assert_eq!(r["risk_gate"]["missing_permissions"], json!(["write"]));
}
#[test]
fn unknown_required_fields_abstain() {
    let mut i = fixture("android-tv");
    i["interaction"]["profile"] = json!("unknown");
    let r = route(&i).unwrap();
    assert_eq!(r["route_status"], "needs_information");
    assert_eq!(r["missing_fields"], json!(["interaction.profile"]));
}
#[test]
fn refuses_bad_types_and_unknown_properties() {
    let mut i = fixture("cli-linux");
    i["constraints"]["offline"] = json!("yes");
    assert!(route(&i).is_err());
    i["constraints"] = json!({"surprise":true});
    assert!(route(&i).is_err());
    i["constraints"] = json!({});
    i["task_type"] = json!("build");
    assert!(route(&i).is_err());
}
#[test]
fn review_requires_finished_work() {
    let mut i = fixture("security-audit");
    i["evidence"] = json!({});
    let r = route(&i).unwrap();
    assert_eq!(r["route_status"], "needs_information");
    assert_eq!(r["missing_fields"], json!(["evidence.finished_work"]));
}
#[test]
fn serialization_is_identical_across_three_runs() {
    let i = fixture("rpi-native-hmi");
    let first = serde_json::to_vec_pretty(&route(&i).unwrap()).unwrap();
    for _ in 0..2 {
        assert_eq!(
            first,
            serde_json::to_vec_pretty(&route(&i).unwrap()).unwrap()
        );
    }
}

#[test]
fn contradictory_interaction_is_not_accepted() {
    let mut i = fixture("web-pwa");
    i["interaction"]["profile"] = json!("direct_touch");
    i["interaction"]["primary_input"] = json!("pointer");
    let r = route(&i).unwrap();
    assert_eq!(r["route_status"], "needs_information");
    assert_eq!(r["missing_fields"], json!(["interaction.primary_input"]));
}
#[test]
fn unmodeled_hard_limits_require_information() {
    let mut i = fixture("cli-linux");
    i["constraints"]["memory_mb"] = json!(1);
    let r = route(&i).unwrap();
    assert_eq!(r["route_status"], "needs_information");
    assert_eq!(
        r["missing_fields"],
        json!(["constraints.memory_mb_evidence"])
    );
}
#[test]
fn linux_desktop_gui_routes_without_touching_macos_or_kiosk() {
    let intent: serde_json::Value =
        serde_json::from_str(include_str!("../examples/linux-desktop-gui.json")).unwrap();
    let result = route(&intent).unwrap();
    assert_eq!(result["route_status"], "ok");
    assert_eq!(result["selected_pack"], "linux-desktop-gui");
    assert_eq!(result["risk_gate"]["status"], "allowed");
    // A macOS desktop intent must still select the macOS pack.
    let mut mac = intent.clone();
    mac["target"]["platform"] = json!("macos");
    mac["target"]["os"] = json!("macos");
    assert_eq!(route(&mac).unwrap()["selected_pack"], "macos-desktop");
    // Kiosk touch fixtures are unchanged by the new pack.
    assert_eq!(
        route(&fixture("linux-kiosk-touch")).unwrap()["selected_pack"],
        "linux-kiosk-touch"
    );
}
