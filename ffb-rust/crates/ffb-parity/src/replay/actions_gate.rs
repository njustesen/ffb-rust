//! `ffb-replay actions <out_dir> <java_dir> <dir|file>...` -- the BACKLOG §O O3 gate: replay every
//! file with the [`Recorder`], write `<out>/<id>.actions.jsonl` + `<id>.status.json` (the Java
//! `--per-replay` formats) and compare with the Java run (`<java_dir>/<id>.actions.jsonl`,
//! `.status.json`).
//!
//! Every record is compared byte for byte (after the `replay_id/seq/final_score_*` prefix; the
//! final score itself is compared separately), plus `record_errors`, `end_check` and
//! `unknown_model_changes` of the status. The MoveDecisionEngine records (`player_select`,
//! `move_target`) need a Java run of the o3 harness (canonical move_target candidate order);
//! `FFB_REPLAY_NO_MDE=1` leaves them out on both sides (to compare with an older Java run).

use super::actions::Recorder;
use super::canonical::write_json_string;
use super::ffbr;
use super::states::replay_with;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The MDE record types, left out of the comparison under `FFB_REPLAY_NO_MDE=1`.
const MDE_TYPES: [&str; 2] = ["\"type\":\"player_select\"", "\"type\":\"move_target\""];

/// A Java action line split into (final score, record text without its leading `{`).
fn split_java_line(line: &str) -> Option<((i32, i32), &str)> {
    let rest = line.strip_prefix("{\"replay_id\":\"")?;
    let (_, rest) = rest.split_once("\",\"seq\":")?;
    let (_, rest) = rest.split_once(",\"final_score_home\":")?;
    let (home, rest) = rest.split_once(",\"final_score_away\":")?;
    let cut = rest.find(|c: char| c == ',' || c == '}')?;
    let away = &rest[..cut];
    let record = rest[cut..].strip_prefix(',').unwrap_or(&rest[cut..]);
    Some(((home.parse().ok()?, away.parse().ok()?), record))
}

fn is_mde(record: &str) -> bool {
    MDE_TYPES.iter().any(|t| record.starts_with(t))
}

fn json_string(s: Option<&str>) -> String {
    match s {
        None => "null".into(),
        Some(s) => {
            let mut out = String::new();
            write_json_string(s, &mut out);
            out
        }
    }
}

fn json_map(m: &BTreeMap<String, u64>) -> String {
    let body: Vec<String> = m.iter().map(|(k, v)| format!("{}:{v}", json_string(Some(k)))).collect();
    format!("{{{}}}", body.join(","))
}

/// A record's kind for the tallies: `dialog_id` of a dialog record, else its `type`.
fn record_kind(record: &str) -> String {
    let field = |key: &str| {
        record.find(key).map(|i| {
            let rest = &record[i + key.len()..];
            rest.split('"').next().unwrap_or("").to_string()
        })
    };
    field("\"dialog_id\":\"").or_else(|| field("\"type\":\"")).unwrap_or_else(|| "?".into())
}

fn status_map(v: &Value, key: &str) -> BTreeMap<String, u64> {
    v.get(key)
        .and_then(Value::as_object)
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0))).collect())
        .unwrap_or_default()
}

pub fn actions(out_dir: &Path, java_dir: &Path, inputs: &[PathBuf]) -> i32 {
    let replays = ffbr::collect_inputs(inputs);
    if let Err(e) = std::fs::create_dir_all(out_dir) {
        eprintln!("cannot create {}: {e}", out_dir.display());
        return 2;
    }
    // run_info.json like the Java --per-replay run: the done-check (scripts/replay_parity_check.py)
    // only counts outputs whose tool_sha256 is the sha256 of the tool the manifest names.
    let exe = std::env::current_exe().ok();
    let tool_sha = exe.as_ref().and_then(|p| std::fs::read(p).ok()).map(|b| super::canonical::sha256_hex(&b));
    let tool_path = exe.as_ref().map(|p| p.to_string_lossy().replace('\\', "/"));
    let _ = std::fs::write(
        out_dir.join("run_info.json"),
        format!(
            "{{\"tool\":{},\"tool_sha256\":{},\"replays\":{}}}\n",
            json_string(tool_path.as_deref()),
            json_string(tool_sha.as_deref()),
            replays.len()
        ),
    );
    let (mut identical, mut different, mut errors) = (0u64, 0u64, 0u64);
    let (mut compared_records, mut skipped_mde) = (0u64, 0u64);
    let mut diff_kinds: BTreeMap<String, u64> = BTreeMap::new();
    let mut error_kinds: BTreeMap<String, u64> = BTreeMap::new();
    let mde = std::env::var("FFB_REPLAY_NO_MDE").map_or(true, |v| v != "1");
    for (n, (id, path)) in replays.iter().enumerate() {
        let mut recorder = Recorder::new();
        recorder.mde = mde;
        let result = replay_with(path, false, Some(&mut recorder));
        // Rust outputs, Java formats.
        let mut actions = String::new();
        for (seq, r) in recorder.records.iter().enumerate() {
            actions.push_str(&format!(
                "{{\"replay_id\":\"{id}\",\"seq\":{seq},\"final_score_home\":{},\"final_score_away\":{}{}{}\n",
                result.end_score.0,
                result.end_score.1,
                if r.len() > 2 { "," } else { "" },
                &r[1..]
            ));
        }
        let _ = std::fs::write(out_dir.join(format!("{id}.actions.jsonl")), &actions);
        let status = format!(
            "{{\"exception\":{},\"skipped_syncs\":0,\"applied_syncs\":{},\"apply_errors\":0,\"records\":{},\"record_errors\":{},\"end_check\":{},\"unknown_model_changes\":{},\"unhandled_commands\":{{}}}}\n",
            json_string(result.error.as_deref()),
            result.lines.len(),
            recorder.records.len(),
            json_map(&recorder.record_errors),
            json_string(Some(&result.end_check)),
            json_map(&result.unknown),
        );
        let _ = std::fs::write(out_dir.join(format!("{id}.status.json")), &status);

        if let Some(e) = &result.error {
            errors += 1;
            println!("ERROR {id}: {e}");
            let kind = e.split_once(": ").map_or(e.as_str(), |(_, k)| k);
            *error_kinds.entry(kind.chars().take(90).collect()).or_default() += 1;
            continue;
        }
        let java_text = std::fs::read_to_string(java_dir.join(format!("{id}.actions.jsonl"))).unwrap_or_default();
        let java_status: Value = std::fs::read_to_string(java_dir.join(format!("{id}.status.json")))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or(Value::Null);
        let mut problems: Vec<String> = Vec::new();
        // The state lines too (the same replay produced both).
        let states_text: String = result.lines.iter().map(|l| format!("{l}\n")).collect();
        let _ = std::fs::write(out_dir.join(format!("{id}.states")), &states_text);
        let java_states = std::fs::read_to_string(java_dir.join(format!("{id}.states"))).unwrap_or_default();
        if java_states != states_text {
            let pos = java_states.lines().zip(result.lines.iter()).position(|(j, r)| j != r);
            problems.push(format!(
                "states differ (first at line {pos:?}; {} java lines, {} rust)",
                java_states.lines().count(),
                result.lines.len()
            ));
        }
        let mut java_records: Vec<&str> = Vec::new();
        for line in java_text.lines() {
            match split_java_line(line) {
                Some((score, record)) => {
                    if score != result.end_score {
                        problems.push(format!("final score java {score:?} rust {:?}", result.end_score));
                        break;
                    }
                    if !mde && is_mde(record) {
                        skipped_mde += 1;
                    } else {
                        java_records.push(record);
                    }
                }
                None => {
                    problems.push(format!("unparsable java line {:?}", &line[..line.len().min(80)]));
                    break;
                }
            }
        }
        let rust_records: Vec<&str> = recorder.records.iter().map(|r| &r[1..]).collect();
        compared_records += java_records.len().min(rust_records.len()) as u64;
        if let Some(pos) = java_records.iter().zip(rust_records.iter()).position(|(j, r)| j != r) {
            let kind = record_kind(java_records[pos]);
            *diff_kinds.entry(kind.clone()).or_default() += 1;
            problems.push(format!("record {pos} ({kind}) differs"));
            let _ = std::fs::write(
                out_dir.join(format!("{id}.actions.diff.txt")),
                format!("record {pos}\njava {{{}\nrust {{{}\n", java_records[pos], rust_records[pos]),
            );
        } else if java_records.len() != rust_records.len() {
            let pos = java_records.len().min(rust_records.len());
            let kind = java_records.get(pos).or(rust_records.get(pos)).map_or("?".into(), |r| record_kind(r));
            *diff_kinds.entry(format!("count:{kind}")).or_default() += 1;
            problems.push(format!("{} java records vs {} rust; first extra ({kind}) at {pos}", java_records.len(), rust_records.len()));
            let _ = std::fs::write(
                out_dir.join(format!("{id}.actions.diff.txt")),
                format!("record {pos}\njava {:?}\nrust {:?}\n", java_records.get(pos), rust_records.get(pos)),
            );
        }
        let java_errors = status_map(&java_status, "record_errors");
        if java_errors != recorder.record_errors {
            problems.push(format!("record_errors java {java_errors:?} rust {:?} {:?}", recorder.record_errors, recorder.first_errors));
        }
        let java_end = java_status.get("end_check").and_then(Value::as_str).unwrap_or("?");
        if java_end != result.end_check {
            problems.push(format!("end_check java {java_end:?} rust {:?}", result.end_check));
        }
        let java_unknown = status_map(&java_status, "unknown_model_changes");
        if java_unknown != result.unknown {
            problems.push(format!("unknown_model_changes java {java_unknown:?} rust {:?}", result.unknown));
        }
        if problems.is_empty() {
            identical += 1;
        } else {
            different += 1;
            println!("DIFF {id}: {}", problems.join("; "));
        }
        if (n + 1) % 100 == 0 {
            eprintln!("  [{}] identical {identical} different {different} errors {errors}", n + 1);
        }
    }
    println!("first-diff kinds:");
    for (k, c) in &diff_kinds {
        println!("  {c:6} {k}");
    }
    println!("error kinds:");
    for (k, c) in &error_kinds {
        println!("  {c:6} {k}");
    }
    println!(
        "actions ({}): {identical} identical, {different} different, {errors} errors, of {} replays ({compared_records} records compared, {skipped_mde} java MDE records left out)",
        if mde { "all records" } else { "without MDE records" },
        replays.len()
    );
    if different == 0 && errors == 0 && identical > 0 { 0 } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_lines_split_into_score_and_record() {
        let line = "{\"replay_id\":\"1966013\",\"seq\":12,\"final_score_home\":2,\"final_score_away\":-1,\"type\":\"dialog\",\"action\":0}";
        let ((h, a), rec) = split_java_line(line).unwrap();
        assert_eq!((h, a), (2, -1));
        assert_eq!(rec, "\"type\":\"dialog\",\"action\":0}");
        assert!(is_mde("\"type\":\"move_target\",\"x\":1}"));
        assert!(!is_mde(rec));
        assert_eq!(record_kind("\"type\":\"dialog\",\"dialog_id\":\"RE_ROLL\",\"a\":1}"), "RE_ROLL");
        assert_eq!(record_kind("\"type\":\"foul_target\",\"a\":1}"), "foul_target");
    }
}
