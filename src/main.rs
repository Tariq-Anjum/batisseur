use serde_json::{json, Value};
use std::{env, fs, process::ExitCode};

const HELP:&str="batisseur experimental v0.1.0\n\nUSAGE:\n  batisseur route <intent.json> [--json]\n  batisseur validate <intent.json>\n  batisseur packs list\n  batisseur packs show <id>\n  batisseur bench [--json]\n\nRouting is advisory and performs no execution. See docs/SPEC.md and schemas/.";
fn read(path: &str) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|e| format!("cannot read intent: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("invalid intent JSON: {e}"))
}
fn print_json(v: &Value) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(v).map_err(|e| e.to_string())?
    );
    Ok(())
}
fn run(args: &[String]) -> Result<bool, String> {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    match a.as_slice() {
        [] | ["--help"] | ["-h"] => println!("{HELP}"),
        ["--version"] => println!("batisseur 0.1.0"),
        ["route", file] | ["route", file, "--json"] => {
            let r = batisseur::route(&read(file)?)?;
            if a.len() == 3 {
                print_json(&r)?;
            } else {
                println!(
                    "{}: {}",
                    r["route_status"].as_str().unwrap_or_default(),
                    r["selected_pack"].as_str().unwrap_or("no pack")
                );
                // Include the full trace in human mode as well.
                print_json(&r)?;
            }
        }
        ["validate", file] => {
            batisseur::validate(&read(file)?)?;
            print_json(&json!({"valid":true,"taxonomy_version":"tt-1.1.0"}))?;
        }
        ["packs", "list"] => {
            for pack in batisseur::packs()? {
                println!("{} {}", pack.id, pack.version);
            }
        }
        ["packs", "show", id] => {
            let pack = batisseur::packs()?
                .into_iter()
                .find(|p| p.id == *id)
                .ok_or_else(|| format!("unknown pack: {id}"))?;
            print_json(&serde_json::to_value(pack).map_err(|e| e.to_string())?)?;
        }
        ["bench"] | ["bench", "--json"] => {
            let metrics = batisseur::bench()?;
            if a.len() == 2 {
                print_json(&metrics)?;
            } else {
                println!("BuildBench: {}/{} correct ({}%); abstentions {}/{}; unexpected activations {}; deterministic three runs {}/{}; verification {}/{}; risk gates {}/{}",
                    metrics["correct"],metrics["total"],metrics["accuracy_percent"],metrics["correct_abstentions"],metrics["expected_abstentions"],metrics["unexpected_activations"],metrics["deterministic_three_runs"],metrics["total"],metrics["verification_matches"],metrics["total"],metrics["risk_gate_matches"],metrics["total"]);
            }
            return Ok(metrics["passed"] == true);
        }
        _ => return Err(format!("invalid arguments\n{HELP}")),
    }
    Ok(true)
}
fn main() -> ExitCode {
    match run(&env::args().skip(1).collect::<Vec<_>>()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("batisseur: {e}");
            ExitCode::from(2)
        }
    }
}
