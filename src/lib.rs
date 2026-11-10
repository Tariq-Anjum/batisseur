//! Typed, deterministic routing. No execution, installation, or network access.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub const TAXONOMY: &str = include_str!("../contracts/tt-1.toml");
pub const INTENT_SCHEMA: &str = include_str!("../schemas/build-intent.schema.json");
pub const RESULT_SCHEMA: &str = include_str!("../schemas/route-result.schema.json");
pub const BUILD_BENCH: &str = include_str!("../bench/buildbench-v0.jsonl");
pub const TIE_MARGIN: i64 = 3;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Framework {
    pub id: String,
    pub name: String,
    pub fit_notes: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Pack {
    pub id: String,
    pub version: String,
    pub taxonomy_version: String,
    pub platforms: Vec<String>,
    pub operating_systems: Vec<String>,
    pub form_factors: Vec<String>,
    pub deployment_envs: Vec<String>,
    pub application_kinds: Vec<String>,
    pub modes: Vec<String>,
    pub runtimes: Vec<String>,
    pub interaction_profiles: Vec<String>,
    pub primary_inputs: Vec<String>,
    pub offline_supported: bool,
    pub native_io_supported: bool,
    pub stack: String,
    pub required_verifiers: Vec<String>,
    pub conflicts: Vec<String>,
    pub official_references: Vec<String>,
    pub dependency_cost: i64,
    pub maintenance_cost: i64,
    pub context_cost: i64,
    pub runtime_cost: i64,
    pub licensing_penalty: i64,
    pub security_penalty: i64,
    pub unsupported_penalty: i64,
    pub framework_candidates: Vec<Framework>,
}

pub fn packs() -> Result<Vec<Pack>, String> {
    let mut packs = [
        include_str!("../packs/android-tv.toml"),
        include_str!("../packs/android-tablet.toml"),
        include_str!("../packs/macos-desktop.toml"),
        include_str!("../packs/linux-kiosk-touch.toml"),
        include_str!("../packs/web-pwa.toml"),
        include_str!("../packs/cli-linux.toml"),
        include_str!("../packs/daemon-linux.toml"),
        include_str!("../packs/browser-extension.toml"),
    ]
    .iter()
    .map(|s| toml::from_str::<Pack>(s).map_err(|e| e.to_string()))
    .collect::<Result<Vec<_>, _>>()?;
    let taxonomy = taxonomy()?;
    for pack in &packs {
        if pack.taxonomy_version != taxonomy["version"].as_str().unwrap_or_default() {
            return Err(format!("pack {} taxonomy mismatch", pack.id));
        }
    }
    packs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(packs)
}

fn taxonomy() -> Result<toml::Value, String> {
    let value: toml::Value = toml::from_str(TAXONOMY).map_err(|e| e.to_string())?;
    if value["version"].as_str() != Some("tt-1.1.0") {
        return Err("pinned taxonomy contract version mismatch".into());
    }
    Ok(value)
}

fn known(v: &Value) -> bool {
    !v.is_null() && v.as_str() != Some("unknown")
}
fn path_name(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        child.into()
    } else {
        format!("{parent}.{child}")
    }
}

/// Validates the schema keywords used by this crate's two schemas.
/// This is deliberately not a general JSON Schema implementation.
pub fn check_schema(value: &Value, schema: &Value, strict: bool) -> Result<(), String> {
    check_node(value, schema, "", strict)
}
fn check_node(value: &Value, schema: &Value, path: &str, strict: bool) -> Result<(), String> {
    let error = |why: &str| format!("{path}: {why}");
    if let Some(types) = schema.get("type") {
        let valid = |t: &str| match t {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "null" => value.is_null(),
            "boolean" => value.is_boolean(),
            "integer" => value.is_i64() || value.is_u64(),
            "number" => value.is_number(),
            _ => false,
        };
        let fits = types.as_str().map(valid).unwrap_or_else(|| {
            types
                .as_array()
                .is_some_and(|ts| ts.iter().any(|t| t.as_str().is_some_and(valid)))
        });
        if !fits {
            return Err(error("invalid JSON type"));
        }
    }
    if let Some(c) = schema.get("const") {
        if value != c {
            return Err(error("constant mismatch"));
        }
    }
    if let Some(es) = schema["enum"].as_array() {
        if !es.contains(value) {
            return Err(error("unknown enum value"));
        }
    }
    if let (Some(n), Some(min)) = (value.as_f64(), schema["minimum"].as_f64()) {
        if n < min {
            return Err(error("below minimum"));
        }
    }
    if let (Some(s), Some(min)) = (value.as_str(), schema["minLength"].as_u64()) {
        if (s.chars().count() as u64) < min {
            return Err(error("string too short"));
        }
    }
    if let Some(object) = value.as_object() {
        if strict {
            if let Some(required) = schema["required"].as_array() {
                for field in required {
                    let field = field.as_str().ok_or("invalid schema required field")?;
                    if !object.contains_key(field) {
                        return Err(error(&format!("missing {field}")));
                    }
                }
            }
        }
        for (field, child) in object {
            match schema["properties"].get(field) {
                Some(rule) => check_node(child, rule, &path_name(path, field), strict)?,
                None if schema["additionalProperties"] == false => {
                    return Err(error(&format!("unknown property {field}")))
                }
                _ => (),
            }
        }
    }
    if let Some(items) = value.as_array() {
        if let Some(rule) = schema.get("items") {
            for (n, item) in items.iter().enumerate() {
                check_node(item, rule, &format!("{path}[{n}]"), strict)?;
            }
        }
        if schema["uniqueItems"] == true {
            for (n, item) in items.iter().enumerate() {
                if items[..n].contains(item) {
                    return Err(error("duplicate array item"));
                }
            }
        }
    }
    Ok(())
}

fn missing_required(value: &Value, schema: &Value, path: &str, missing: &mut Vec<String>) {
    if let Some(required) = schema["required"].as_array() {
        for field in required.iter().filter_map(Value::as_str) {
            let name = path_name(path, field);
            let child = &value[field];
            if !known(child) {
                missing.push(name);
            } else {
                missing_required(child, &schema["properties"][field], &name, missing);
            }
        }
    }
}

pub fn validate(intent: &Value) -> Result<(), String> {
    let _ = route(intent)?;
    let schema: Value = serde_json::from_str(INTENT_SCHEMA).map_err(|e| e.to_string())?;
    check_schema(intent, &schema, true)?;
    let mut missing = Vec::new();
    missing_required(intent, &schema, "", &mut missing);
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("unknown required fields: {}", missing.join(", ")))
    }
}

fn family_map(task: &str, artifact: &str) -> Result<Vec<String>, String> {
    let table: Value = serde_json::from_str(include_str!("../contracts/families.json"))
        .map_err(|e| e.to_string())?;
    let taxonomy = taxonomy()?;
    let ids: BTreeSet<_> = taxonomy["type"]
        .as_array()
        .ok_or("missing taxonomy types")?
        .iter()
        .filter_map(|t| t["id"].as_str())
        .collect();
    let keys: BTreeSet<_> = table
        .as_object()
        .ok_or("invalid family map")?
        .keys()
        .map(String::as_str)
        .collect();
    if ids != keys {
        return Err("family map must exactly cover pinned taxonomy".into());
    }
    if task == "design" && artifact == "interface" {
        return Ok(vec!["Experience".into()]);
    }
    serde_json::from_value(table[task].clone()).map_err(|e| e.to_string())
}

fn rejection(pack: &Pack, i: &Value) -> Vec<String> {
    let axes = [
        ("/target/platform", &pack.platforms),
        ("/target/os", &pack.operating_systems),
        ("/target/form_factor", &pack.form_factors),
        ("/target/deployment_env", &pack.deployment_envs),
        ("/application/kind", &pack.application_kinds),
        ("/application/mode", &pack.modes),
        ("/application/runtime", &pack.runtimes),
        ("/interaction/profile", &pack.interaction_profiles),
        ("/interaction/primary_input", &pack.primary_inputs),
    ];
    let mut reasons = Vec::new();
    for (path, allowed) in axes {
        let value = i.pointer(path).and_then(Value::as_str).unwrap_or_default();
        if !allowed.iter().any(|a| a == value) {
            reasons.push(format!("{path}: {value} is ineligible"));
        }
    }
    if i["constraints"]["offline"] == true && !pack.offline_supported {
        reasons.push("offline unsupported".into());
    }
    reasons
}

fn score(pack: &Pack, i: &Value) -> Value {
    let distribution = i["constraints"]["distribution"].as_str();
    let stack = i["constraints"]["existing_stack"].as_str();
    let verified = i["evidence"]["verifiers"].as_array().is_some_and(|vs| {
        pack.required_verifiers
            .iter()
            .all(|v| vs.contains(&json!(v)))
    });
    let accessible = i["interaction"]["accessibility"]
        .as_array()
        .is_some_and(|a| !a.is_empty());
    let mut terms = vec![
        json!({"term":"platform_fit","value":30,"reason":"platform, OS, form factor and deployment passed typed eligibility"}),
        json!({"term":"interaction_fit","value":25,"reason":"profile and primary input passed typed eligibility"}),
        json!({"term":"runtime_fit","value":10,"reason":"application kind, mode and runtime passed typed eligibility"}),
        json!({"term":"distribution_fit","value":if distribution.is_some_and(|d|pack.deployment_envs.iter().any(|p|p==d)){10}else{0},"reason":"+10 for explicit supported distribution; otherwise 0"}),
        json!({"term":"existing_stack_fit","value":if stack==Some(pack.stack.as_str()){20}else{0},"reason":"+20 for exact existing_stack match; otherwise 0"}),
        json!({"term":"verification_fit","value":if verified{5}else{0},"reason":"+5 if supplied verifier evidence covers the full pack contract; otherwise 0"}),
        json!({"term":"accessibility_fit","value":if accessible{5}else{0},"reason":"+5 for declared accessibility needs with pack verifiers; a requirement signal, not proof of compliance"}),
    ];
    for (name, cost) in [
        ("dependency_cost", pack.dependency_cost),
        ("maintenance_cost", pack.maintenance_cost),
        ("context_cost", pack.context_cost),
        ("runtime_cost", pack.runtime_cost),
        ("licensing_penalty", pack.licensing_penalty),
        ("security_penalty", pack.security_penalty),
        ("unsupported_penalty", pack.unsupported_penalty),
    ] {
        terms.push(json!({"term":name,"value":-cost,"reason":format!("subtract pack-declared {name} = {cost}; heuristic routing units, not measured resources")}));
    }
    let total: i64 = terms.iter().filter_map(|t| t["value"].as_i64()).sum();
    json!({"pack":pack.id,"score":total,"terms":terms})
}

fn abstain(mut r: Value, mut missing: Vec<String>, reason: &str) -> Value {
    missing.sort();
    missing.dedup();
    r["route_status"] = json!("needs_information");
    r["missing_fields"] = json!(missing);
    r["reasons"] = json!([reason]);
    r["families"] = json!([]);
    r
}

pub fn route(intent: &Value) -> Result<Value, String> {
    let taxonomy = taxonomy()?;
    if let Some(version) = intent.get("taxonomy_version") {
        if version != taxonomy["version"].as_str().unwrap_or_default() {
            return Err("taxonomy version must equal pinned tt-1.1.0".into());
        }
    }
    let schema: Value = serde_json::from_str(INTENT_SCHEMA).map_err(|e| e.to_string())?;
    check_schema(intent, &schema, false)?;
    let task = intent["task_type"].as_str().unwrap_or("unknown");
    let mut result = json!({"taxonomy_version":"tt-1.1.0","route_status":"ok","task_type":intent.get("task_type").cloned().unwrap_or(Value::Null),
        "families":[],"selected_pack":null,"capabilities":[],"required_verifiers":[],"missing_fields":[],
        "candidates":[],"rejected":[],"reasons":[],"tie_margin":TIE_MARGIN,"confidence":"insufficient",
        "risk_gate":{"status":"not_evaluated","missing_permissions":[],"reasons":[]},"plan":[]});
    if task == "unclear" {
        return Ok(abstain(
            result,
            vec!["task_type".into()],
            "shared taxonomy unclear maps to needs_information",
        ));
    }
    let mut missing = Vec::new();
    missing_required(intent, &schema, "", &mut missing);
    if !missing.is_empty() {
        return Ok(abstain(
            result,
            missing,
            "required typed values are missing or unknown",
        ));
    }
    // Hard limits without comparable pack evidence cannot be silently satisfied.
    let unsupported: Vec<String> = [
        "hardware",
        "latency_ms",
        "memory_mb",
        "storage_mb",
        "power",
        "security",
        "privacy",
    ]
    .iter()
    .filter(|field| intent["constraints"].get(**field).is_some())
    .map(|field| format!("constraints.{field}_evidence"))
    .collect();
    if !unsupported.is_empty() {
        return Ok(abstain(result, unsupported, "pack evidence does not model these explicit constraints; supply a vetted pack contract"));
    }
    let profiles: Value =
        serde_json::from_str(include_str!("../contracts/interaction-profiles.json"))
            .map_err(|e| e.to_string())?;
    let profile = intent["interaction"]["profile"]
        .as_str()
        .unwrap_or_default();
    if !profiles[profile]
        .as_array()
        .is_some_and(|inputs| inputs.contains(&intent["interaction"]["primary_input"]))
    {
        return Ok(abstain(
            result,
            vec!["interaction.primary_input".into()],
            "primary input contradicts the declared interaction profile",
        ));
    }
    if task == "review" && intent["evidence"]["finished_work"] != true {
        return Ok(abstain(
            result,
            vec!["evidence.finished_work".into()],
            "review is eligible only for finished work",
        ));
    }
    if intent["interaction"]["profile"] == "indirect_focus"
        && !known(&intent["interaction"]["focus_model"])
    {
        return Ok(abstain(
            result,
            vec!["interaction.focus_model".into()],
            "indirect focus needs an explicit focus model",
        ));
    }
    let families = family_map(task, intent["artifact"].as_str().unwrap_or_default())?;
    if families.is_empty() {
        result["confidence"] = json!("typed");
        return Ok(result);
    }
    let mut candidates = Vec::new();
    let mut rejected = Vec::new();
    for pack in packs()? {
        let mut reasons = rejection(&pack, intent);
        if reasons.is_empty() {
            let scored = score(&pack, intent);
            // Composition/conflict validation follows scoring, before selection.
            if intent["constraints"]["native_io"] == true
                && (!pack.native_io_supported || pack.conflicts.iter().any(|c| c == "native_io"))
            {
                reasons.push("native_io conflicts with browser capability".into());
            }
            if let Some(d) = intent["constraints"]["distribution"]
                .as_str()
                .filter(|s| *s != "unknown")
            {
                if !pack.deployment_envs.iter().any(|p| p == d) {
                    reasons.push("explicit distribution conflicts with pack".into());
                }
            }
            if reasons.is_empty() {
                candidates.push((pack, scored));
                continue;
            }
        }
        rejected.push(json!({"pack":pack.id,"reasons":reasons}));
    }
    candidates.sort_by(|a, b| {
        b.1["score"]
            .as_i64()
            .cmp(&a.1["score"].as_i64())
            .then(a.0.id.cmp(&b.0.id))
    });
    result["candidates"] = json!(candidates.iter().map(|(_, s)| s).collect::<Vec<_>>());
    result["rejected"] = json!(rejected);
    let Some((selected, _)) = candidates.first() else {
        return Ok(abstain(result,vec!["supported_pack".into()],"no eligible conflict-free pack; supply supported target/interaction values or extend the pack registry"));
    };
    if candidates.len() > 1 {
        let delta = candidates[0].1["score"].as_i64().unwrap_or_default()
            - candidates[1].1["score"].as_i64().unwrap_or_default();
        if delta <= TIE_MARGIN {
            return Ok(abstain(result,vec!["constraints.existing_stack".into()],"top candidates are within the inclusive tie margin; supply a distinguishing stack or constraint"));
        }
    }
    let workflows: Value = serde_json::from_str(include_str!("../contracts/workflows.json"))
        .map_err(|e| e.to_string())?;
    let workflow = workflows.get(task).ok_or("task workflow missing")?;
    let permitted = intent["authority"]["permitted_ops"]
        .as_array()
        .ok_or("authority.permitted_ops must be an array")?;
    let mut permissions: Vec<String> = workflow["required_ops"]
        .as_array()
        .ok_or("invalid workflow ops")?
        .iter()
        .filter(|op| !permitted.contains(op))
        .filter_map(Value::as_str)
        .map(String::from)
        .collect();
    if intent["work_kind"] == "reconstruction" && intent["authority"]["reverse_engineering"] != true
    {
        permissions.push("reverse_engineering".into());
    }
    if intent["constraints"]["privileged"] == true {
        permissions.push("privileged_execution_outside_mvp".into());
    }
    permissions.sort();
    permissions.dedup();
    result["risk_gate"] = json!({"status":if permissions.is_empty(){"allowed"}else{"requires_authorization"},"missing_permissions":permissions,
        "reasons":["routing is advisory; declared permissions authorize no execution by this CLI"]});
    result["families"] = json!(families);
    result["selected_pack"] = json!(selected.id);
    result["capabilities"] = json!([format!("{task}:{}", selected.id)]);
    result["required_verifiers"] = json!(selected.required_verifiers);
    result["confidence"] = json!("typed_unique");
    result["reasons"]=json!(["unique highest eligible score outside tie margin; one pack suffices; task type is unchanged"]);
    result["plan"] = json!([{"id":"primary","task_type":task,"workflow":task,"stages":workflow["stages"],"pack":selected.id,"depends_on":[],"executable":false}]);
    Ok(result)
}

pub fn bench() -> Result<Value, String> {
    let mut total = 0;
    let mut correct = 0;
    let mut abstentions = 0;
    let mut correct_abstentions = 0;
    let mut deterministic = 0;
    let mut unexpected = 0;
    let mut verifier_matches = 0;
    let mut gate_matches = 0;
    let mut cases = Vec::new();
    for line in BUILD_BENCH.lines().filter(|s| !s.trim().is_empty()) {
        let case: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
        let r = route(&case["intent"])?;
        let e = &case["expected"];
        let passed = r["route_status"] == e["route_status"]
            && r["selected_pack"] == e["selected_pack"]
            && r["families"] == e["families"]
            && r["capabilities"] == e["capabilities"];
        let bytes = serde_json::to_vec(&r).map_err(|e| e.to_string())?;
        let same = (0..2).all(|_| {
            route(&case["intent"])
                .ok()
                .and_then(|v| serde_json::to_vec(&v).ok())
                .is_some_and(|b| b == bytes)
        });
        if passed {
            correct += 1;
        }
        if same {
            deterministic += 1;
        }
        let expected_count = usize::from(e["selected_pack"].is_string());
        unexpected += r["capabilities"]
            .as_array()
            .map_or(0, Vec::len)
            .saturating_sub(expected_count);
        let expected_verifiers = e["required_verifiers"].clone();
        if r["required_verifiers"] == expected_verifiers {
            verifier_matches += 1;
        }
        if r["risk_gate"]["status"] == e["risk_gate"] {
            gate_matches += 1;
        }
        if e["route_status"] == "needs_information" {
            abstentions += 1;
            if r["route_status"] == "needs_information"
                && r["missing_fields"]
                    .as_array()
                    .is_some_and(|a| !a.is_empty())
            {
                correct_abstentions += 1;
            }
        }
        total += 1;
        cases.push(json!({"id":case["id"],"passed":passed,"deterministic_three_runs":same}));
    }
    Ok(
        json!({"suite":"buildbench-v0","total":total,"correct":correct,"accuracy_percent":100.0*correct as f64/total as f64,
        "expected_abstentions":abstentions,"correct_abstentions":correct_abstentions,"unexpected_activations":unexpected,
        "deterministic_three_runs":deterministic,"verification_matches":verifier_matches,"risk_gate_matches":gate_matches,
        "context_bytes":BUILD_BENCH.len()+TAXONOMY.len()+INTENT_SCHEMA.len(),"tokens":null,"latency_ms":null,
        "measurement_notes":"context_bytes counts benchmark, taxonomy and intent schema only; token/latency instrumentation is out of scope",
        "passed":correct==total && deterministic==total && unexpected==0 && verifier_matches==total && gate_matches==total && correct_abstentions==abstentions,
        "cases":cases}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(id: &str) -> Value {
        BUILD_BENCH
            .lines()
            .map(|s| serde_json::from_str::<Value>(s).unwrap())
            .find(|c| c["id"] == id)
            .unwrap()["intent"]
            .clone()
    }
    #[test]
    fn scoring_terms_are_signed_and_additive() {
        let pack = packs()
            .unwrap()
            .into_iter()
            .find(|p| p.id == "cli-linux")
            .unwrap();
        let mut intent = fixture("cli-linux");
        let baseline = score(&pack, &intent)["score"].as_i64().unwrap();
        intent["constraints"]["existing_stack"] = json!("rust");
        assert_eq!(
            score(&pack, &intent)["score"].as_i64().unwrap(),
            baseline + 20
        );
    }
    #[test]
    fn abstention_is_empty_and_deterministic() {
        let i = json!({"taxonomy_version":"tt-1.1.0","task_type":"implement"});
        let a = route(&i).unwrap();
        assert_eq!(a["route_status"], "needs_information");
        assert_eq!(a["capabilities"], json!([]));
        for _ in 0..2 {
            assert_eq!(
                serde_json::to_vec(&a).unwrap(),
                serde_json::to_vec(&route(&i).unwrap()).unwrap()
            );
        }
    }
}
