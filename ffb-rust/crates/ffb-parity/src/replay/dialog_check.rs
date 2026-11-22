//! `ffb-replay dialog-check <dialog.tsv> [dialogId...]` -- BACKLOG §O O2(a)(6) gate: the Rust
//! dialog-parameter JSON port against the Java decode of every DISTINCT dialog parameter in the
//! corpus.
//!
//! Java side: `ReplayDialogDump <out.tsv> <corpus>` (harness) writes one line per distinct input,
//! `<ruleset>\t<start|change>\t<canonical RAW input>\t<canonical output | null | ERROR ..>`, where
//! raw = before `dropUnknownValues` (model changes only) and output = `new DialogParameterFactory()
//! .forJsonValue(rules, value).toJsonValue()` canonicalised under the path `dialogParameter`.
//! Rust side: the same drop rule (`drop_rules::drop_unknown_values`), `DialogParameterFactory::
//! for_json_value`, `to_json_value`, the same canonicalisation; a Java ERROR must be a Rust `Err`.

use super::{canonical, drop_rules};
use ffb_model::dialog::dialog_parameter_factory::DialogParameterFactory;
use ffb_model::enums::Rules;
use ffb_model::model::game_options::GameOptions;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

fn rules_for(name: &str) -> Result<Rules, String> {
    match name {
        "BB2016" => Ok(Rules::Bb2016),
        "BB2020" => Ok(Rules::Bb2020),
        "BB2025" => Ok(Rules::Bb2025),
        other => Err(format!("unknown ruleset {other}")),
    }
}

/// The Rust result for one dump line: `Ok(text)` = canonical output or "null", `Err` = a throw.
pub fn rust_output(rules: Rules, source: &str, raw: &Value) -> Result<String, String> {
    let mut value = raw.clone();
    if source == "change" {
        let mut mc = serde_json::json!({"modelChangeId": "gameSetDialogParameter", "modelChangeValue": value});
        // GameOptions only matter to the prayer rules, which never fire on this change id.
        let (drop, _) = drop_rules::drop_unknown_values(rules, &GameOptions::new(), &mut mc)?;
        if drop {
            return Err("change dropped".to_string());
        }
        value = mc["modelChangeValue"].take();
    }
    match DialogParameterFactory::new().for_json_value(rules, &value)? {
        None => Ok("null".to_string()),
        Some(p) => Ok(canonical::canonical_copy(&p.to_json_value(rules)?, Some("dialogParameter"))?.to_json_string()),
    }
}

pub fn dialog_check(tsv: &Path, only: &[String]) -> i32 {
    let text = match std::fs::read_to_string(tsv) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}: {e}", tsv.display());
            return 2;
        }
    };
    // dialogId -> (identical, different, examples)
    let mut tally: BTreeMap<String, (u64, u64, Vec<String>)> = BTreeMap::new();
    for (n, line) in text.lines().enumerate() {
        let cols: Vec<&str> = line.splitn(4, '\t').collect();
        if cols.len() != 4 {
            eprintln!("line {}: {} columns", n + 1, cols.len());
            return 2;
        }
        let (ruleset, source, raw_text, java) = (cols[0], cols[1], cols[2], cols[3]);
        let raw: Value = match serde_json::from_str(raw_text) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("line {}: raw json {e}", n + 1);
                return 2;
            }
        };
        let dialog_id = match raw.get("dialogId") {
            Some(Value::String(s)) => s.clone(),
            Some(other) => other.to_string(),
            None => "<none>".to_string(),
        };
        if !only.is_empty() && !only.contains(&dialog_id) {
            continue;
        }
        let rust = rules_for(ruleset).and_then(|rules| rust_output(rules, source, &raw));
        let same = match (&rust, java.starts_with("ERROR")) {
            (Err(_), true) => true,
            (Ok(r), false) => r == java,
            _ => false,
        };
        let entry = tally.entry(dialog_id).or_default();
        if same {
            entry.0 += 1;
        } else {
            entry.1 += 1;
            if entry.2.len() < 3 {
                let r = match &rust {
                    Ok(t) => t.clone(),
                    Err(e) => format!("Err({e})"),
                };
                entry.2.push(format!("  {ruleset} {source}\n    raw  {raw_text}\n    rust {r}\n    java {java}"));
            }
        }
    }
    let (mut ok, mut bad) = (0u64, 0u64);
    for (id, (o, b, examples)) in &tally {
        ok += o;
        bad += b;
        println!("{:<36} {o:>7} identical {b:>7} different", id);
        for e in examples {
            println!("{e}");
        }
    }
    println!("dialog-check: {ok} identical, {bad} different, {} dialog ids", tally.len());
    if bad == 0 && ok > 0 { 0 } else { 1 }
}
