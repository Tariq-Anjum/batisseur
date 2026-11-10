use serde_json::{json, Value};
#[test]
fn frozen_oracles_match_routes_and_schemas() {
    let input_schema: Value = serde_json::from_str(batisseur::INTENT_SCHEMA).unwrap();
    let output_schema: Value = serde_json::from_str(batisseur::RESULT_SCHEMA).unwrap();
    let mut count = 0;
    for line in batisseur::BUILD_BENCH.lines() {
        let case: Value = serde_json::from_str(line).unwrap();
        let i = &case["intent"];
        batisseur::check_schema(i, &input_schema, false).unwrap();
        let r = batisseur::route(i).unwrap();
        let e = &case["expected"];
        assert_eq!(r["route_status"], e["route_status"], "{}", case["id"]);
        assert_eq!(r["selected_pack"], e["selected_pack"], "{}", case["id"]);
        assert_eq!(r["families"], e["families"], "{}", case["id"]);
        assert_eq!(r["capabilities"], e["capabilities"], "{}", case["id"]);
        assert_eq!(
            r["required_verifiers"], e["required_verifiers"],
            "{}",
            case["id"]
        );
        assert_eq!(r["risk_gate"]["status"], e["risk_gate"], "{}", case["id"]);
        batisseur::check_schema(&r, &output_schema, true).unwrap();
        if r["route_status"] == "needs_information" {
            assert_eq!(r["plan"], json!([]));
            assert!(!r["missing_fields"].as_array().unwrap().is_empty());
        }
        count += 1;
    }
    assert_eq!(count, 16);
}
