//! `ffb-replay` -- the Rust side of BACKLOG §O (replay processing parity).
//!
//! Subcommands (all read-only on their inputs):
//!   scan <dir|file>...                 read every replay (gunzip + JSON), census the net
//!                                      commands and model-change ids, list unreadable files
//!   canon-check <dump_dir> <states>    re-canonicalise the harness's FFB_REPLAY_STATE_DUMP files
//!                                      (after scrambling every unordered array) and compare the
//!                                      bytes and the sha256 with the Java `.states` lines
//!   start-check <java_dir> <subtree> <dir|file>...
//!                                      Rust init_from -> to_json_value of one Game subtree of every
//!                                      replay's serverGameState vs the Java `ReplayStartDump`
//!                                      `<id>.gs.json` (see start_check.rs); subtree `game` = the
//!                                      whole Game, `startOfGame` = the harness's startOfGame of
//!                                      it vs `<id>.sog.json` (both also vs start_hashes.txt)
//!   dialog-check <dialog.tsv> [dialogId...]
//!                                      Rust dialog-parameter JSON vs the Java `ReplayDialogDump`
//!                                      (every distinct dialog parameter; see dialog_check.rs)
//!   states <out_dir> <java_dir> <dir|file>...
//!                                      replay every model sync in Rust, write <id>.states and diff
//!                                      them with the Java per-sync state hashes (see states.rs)

mod canonical;
mod dialog_check;
mod drop_rules;
mod ffbr;
mod start_check;
mod start_of_game;
mod states;

use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn usage() -> ! {
    eprintln!("usage: ffb-replay scan <dir|file>...");
    eprintln!("       ffb-replay canon-check <dump_dir> <states_file>");
    eprintln!("       ffb-replay start-check <java_dir> <subtree> <dir|file>...");
    eprintln!("       ffb-replay dialog-check <dialog.tsv> [dialogId...]");
    eprintln!("       ffb-replay states <out_dir> <java_dir> <dir|file>...");
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else { usage() };
    let code = match cmd.as_str() {
        "scan" if args.len() >= 2 => scan(&args[1..].iter().map(PathBuf::from).collect::<Vec<_>>()),
        "canon-check" if args.len() == 3 => canon_check(Path::new(&args[1]), Path::new(&args[2])),
        "start-check" if args.len() >= 4 => start_check::start_check(
            Path::new(&args[1]),
            &args[2],
            &args[3..].iter().map(PathBuf::from).collect::<Vec<_>>(),
        ),
        "states" if args.len() >= 4 => states::states(
            Path::new(&args[1]),
            Path::new(&args[2]),
            &args[3..].iter().map(PathBuf::from).collect::<Vec<_>>(),
        ),
        "dialog-check" if args.len() >= 2 => dialog_check::dialog_check(Path::new(&args[1]), &args[2..]),
        _ => usage(),
    };
    std::process::exit(code);
}

/// Census of the corpus as the Rust reader sees it (BACKLOG §O "corpus censuses").
fn scan(inputs: &[PathBuf]) -> i32 {
    let replays = ffbr::collect_inputs(inputs);
    let mut net_commands: BTreeMap<String, (u64, u64)> = BTreeMap::new(); // id -> (commands, replays)
    let mut model_changes: BTreeMap<String, u64> = BTreeMap::new();
    let mut failures: Vec<String> = Vec::new();
    let mut no_game_state = 0u64;
    for (n, (id, path)) in replays.iter().enumerate() {
        match ffbr::read_ffbr(path) {
            Err(e) => failures.push(format!("{id}: {e}")),
            Ok(root) => {
                let Some(cmds) = ffbr::command_array(&root) else {
                    failures.push(format!("{id}: no commandArray"));
                    continue;
                };
                let mut seen_here: BTreeMap<&str, ()> = BTreeMap::new();
                for c in cmds {
                    let nid = ffbr::net_command_id(c);
                    let e = net_commands.entry(nid.to_string()).or_default();
                    e.0 += 1;
                    if seen_here.insert(nid, ()).is_none() {
                        e.1 += 1;
                    }
                    if nid == "serverModelSync" {
                        if let Some(arr) = c.pointer("/modelChangeList/modelChangeArray").and_then(Value::as_array) {
                            for mc in arr {
                                let mid = mc.get("modelChangeId").and_then(Value::as_str).unwrap_or("");
                                *model_changes.entry(mid.to_string()).or_default() += 1;
                            }
                        }
                    }
                }
                if !seen_here.contains_key("serverGameState") {
                    no_game_state += 1;
                }
            }
        }
        if (n + 1) % 200 == 0 {
            eprintln!("  [{}] {} unreadable", n + 1, failures.len());
        }
    }
    println!("replays {} (deduplicated by id), unreadable {}, without serverGameState {}",
        replays.len(), failures.len(), no_game_state);
    println!("net commands (id: commands / replays):");
    for (k, (c, r)) in &net_commands {
        println!("  {k}: {c} / {r}");
    }
    println!("model-change ids: {}", model_changes.len());
    for (k, c) in &model_changes {
        println!("  {k}: {c}");
    }
    for f in &failures {
        println!("UNREADABLE {f}");
    }
    if failures.is_empty() { 0 } else { 1 }
}

/// Reverses every array the canonical form must sort, so a check proves the sort, not the input.
fn scramble(value: &mut Value, path: &str) {
    match value {
        Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                let sub = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
                scramble(v, &sub);
            }
        }
        Value::Array(arr) => {
            let sub = format!("{path}[]");
            for v in arr.iter_mut() {
                scramble(v, &sub);
            }
            if canonical::is_unordered_array(path) {
                arr.reverse();
            }
        }
        _ => {}
    }
}

fn canon_check(dump_dir: &Path, states: &Path) -> i32 {
    let Ok(states_text) = std::fs::read_to_string(states) else {
        eprintln!("cannot read {}", states.display());
        return 2;
    };
    let by_index: BTreeMap<String, String> = states_text
        .lines()
        .filter_map(|l| l.split_once(' ').map(|(i, h)| (i.to_string(), h.to_string())))
        .collect();
    let mut files: Vec<PathBuf> = std::fs::read_dir(dump_dir)
        .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect())
        .unwrap_or_default();
    files.sort();
    let (mut ok, mut bad) = (0u64, 0u64);
    for f in &files {
        // <replay id>.<command index>.json
        let stem = f.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let index = stem.rsplit_once('.').map(|(_, i)| i.to_string()).unwrap_or_default();
        let java_text = match std::fs::read_to_string(f) {
            Ok(t) => t,
            Err(e) => {
                println!("BAD {}: {e}", f.display());
                bad += 1;
                continue;
            }
        };
        let mut value: Value = match serde_json::from_str(&java_text) {
            Ok(v) => v,
            Err(e) => {
                println!("BAD {}: json {e}", f.display());
                bad += 1;
                continue;
            }
        };
        scramble(&mut value, "");
        let rust_text = match canonical::canonical_game_json(&value) {
            Ok(t) => t,
            Err(e) => {
                println!("BAD {}: {e}", f.display());
                bad += 1;
                continue;
            }
        };
        let hash = canonical::sha256_hex(rust_text.as_bytes());
        let java_hash = by_index.get(&index);
        if rust_text == java_text && java_hash == Some(&hash) {
            ok += 1;
        } else {
            bad += 1;
            println!("BAD {}: bytes equal {}, sha {} vs java {:?}", f.display(), rust_text == java_text, hash, java_hash);
        }
    }
    println!("canon-check: {ok} identical, {bad} different, of {} dumps", files.len());
    if bad == 0 && ok > 0 { 0 } else { 1 }
}
