//! `ffb-replay start-check` -- BACKLOG §O O2(a) gate: the Rust Java-JSON loader against the Java
//! decode of every replay's `serverGameState`.
//!
//! Java side: `ReplayStartDump <out> --full <corpus>` (harness) writes `<id>.gs.json` = the
//! canonical JSON of the decoded end-of-game `Game` (after the parser's drop rules). Rust side:
//! the same `serverGameState.game` JSON through the 1:1 ported `init_from` -> `to_json_value` of one
//! model class (a SUBTREE of the Game JSON, so the loader can be gated class by class before the
//! whole `Game` exists), canonicalised with the same path, compared byte for byte.

use super::{canonical, ffbr};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The `serverGameState.game` JSON of one replay (the first serverGameState, as the parser).
pub fn game_state_json(path: &Path) -> Result<Value, String> {
    let mut root = ffbr::read_ffbr(path)?;
    let cmds = root
        .get_mut("commandArray")
        .and_then(Value::as_array_mut)
        .ok_or("no commandArray")?;
    for c in cmds.iter_mut() {
        if ffbr::net_command_id(c) == "serverGameState" {
            return c.get_mut("game").map(Value::take).ok_or_else(|| "serverGameState without game".to_string());
        }
    }
    Err("no serverGameState".to_string())
}

/// Rust `init_from` -> `to_json_value` of the model class at `subtree` of the Game JSON.
fn load_subtree(subtree: &str, game: &Value) -> Result<Value, String> {
    let input = game.get(subtree).ok_or_else(|| format!("{subtree} missing in serverGameState"))?;
    // Game.initFrom parses the options first and initialises the rules from them; every other
    // class is decoded with that ruleset's factories.
    let mut options = ffb_model::model::game_options::GameOptions::new();
    options.init_from(game.get("gameOptions").ok_or("gameOptions missing in serverGameState")?)?;
    match subtree {
        "gameOptions" => Ok(options.to_json_value()),
        "actingPlayer" => {
            let rules = options.rules_version()?;
            let mut acting_player = ffb_model::model::acting_player::ActingPlayer::new();
            acting_player.init_from(rules, input)?;
            acting_player.to_json_value(rules)
        }
        other => Err(format!("no Rust loader for subtree {other} yet")),
    }
}

/// First differing byte offset with a little context, for the report.
fn first_difference(a: &str, b: &str) -> String {
    let pos = a.bytes().zip(b.bytes()).position(|(x, y)| x != y).unwrap_or(a.len().min(b.len()));
    let from = pos.saturating_sub(60);
    let clip = |s: &str| s.get(from..(pos + 60).min(s.len())).unwrap_or("").to_string();
    format!("at byte {pos}: rust ...{}... java ...{}...", clip(a), clip(b))
}

pub fn start_check(java_dir: &Path, subtree: &str, inputs: &[PathBuf]) -> i32 {
    let replays = ffbr::collect_inputs(inputs);
    let (mut ok, mut bad) = (0u64, 0u64);
    let mut shown = 0;
    for (id, path) in &replays {
        let result = (|| -> Result<(String, String), String> {
            let game = game_state_json(path)?;
            let rust = load_subtree(subtree, &game)?;
            let rust_text = canonical::canonical_copy(&rust, Some(subtree))?.to_json_string();
            let java_file = java_dir.join(format!("{id}.gs.json"));
            let java_text = std::fs::read_to_string(&java_file).map_err(|e| format!("{}: {e}", java_file.display()))?;
            let java: Value = serde_json::from_str(&java_text).map_err(|e| format!("java json: {e}"))?;
            let java_sub = java.get(subtree).ok_or_else(|| format!("{subtree} missing in the Java dump"))?;
            let java_sub_text = canonical::canonical_copy(java_sub, Some(subtree))?.to_json_string();
            Ok((rust_text, java_sub_text))
        })();
        match result {
            Ok((r, j)) if r == j => ok += 1,
            Ok((r, j)) => {
                bad += 1;
                if shown < 10 {
                    println!("DIFF {id} {subtree}: {}", first_difference(&r, &j));
                    shown += 1;
                }
            }
            Err(e) => {
                bad += 1;
                if shown < 10 {
                    println!("ERROR {id}: {e}");
                    shown += 1;
                }
            }
        }
    }
    println!("start-check {subtree}: {ok} identical, {bad} different, of {} replays", replays.len());
    if bad == 0 && ok > 0 { 0 } else { 1 }
}
