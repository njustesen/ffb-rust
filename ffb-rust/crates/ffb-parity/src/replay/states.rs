//! `ffb-replay states` -- BACKLOG §O O2(b) gate: the Rust replay of every model sync against the
//! Java per-sync state hashes (`FumbblReplayParser --per-replay`, run5: `<id>.states`).
//!
//! The pipeline is `FumbblReplayParser.parseFile`'s state part, 1:1: the first serverGameState
//! (after `dropUnknownInducements`) -> `startOfGame`; then every later command in order --
//! `serverModelSync`: unknown model-change ids dropped (`KNOWN_MODEL_CHANGES` = the exact
//! `ModelChangeId` names), `dropUnknownValues` per change, the list decoded, applied with
//! `IGNORE_PLAYER_MARKER`, both inducement sets reset when a change opens the coin choice
//! (`ClientReplayer.replayTo`), then one state line `<command index> <sha256 of the canonical Game
//! JSON>`; `serverAddPlayer` / `serverZapPlayer` / `serverUnzapPlayer` are handled by their client
//! command handlers (`add_player.rs`; Zap/Unzap not ported yet: the replay stops with an error).

use super::{canonical, drop_rules, ffbr};
use ffb_model::enums::ModelChangeId;
use ffb_model::inducement::inducement::Inducement;
use ffb_model::model::change::model_change::ModelChange;
use ffb_model::model::change::model_change_data_type::ModelChangeValue;
use ffb_model::model::change::model_change_processor::{DecodedModelChange, ModelChangeProcessor};
use ffb_model::model::game::Game;
use ffb_model::model::inducement_set::InducementSet;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The result of replaying one file.
pub struct ReplayStates {
    /// `<command index> <sha256>` lines, one per applied model sync.
    pub lines: Vec<String>,
    /// The canonical Game text after each applied sync (only when requested).
    pub texts: Vec<String>,
    /// countUnknown keys (dropped ids / values), as the Java status reports them.
    pub unknown: BTreeMap<String, u64>,
    /// The first error (the replay stopped there), if any.
    pub error: Option<String>,
}

/// `ClientReplayer.reset` via `FumbblReplayParser.resetInducements`: every inducement re-added as
/// `new Inducement(type, value)` (uses back to 0).
fn reset_inducements(set: &mut InducementSet) {
    for (_, inducement) in set.get_inducement_mapping() {
        set.add_inducement(Inducement::new(inducement.type_id.clone(), inducement.value, inducement.usages.clone()));
    }
}

fn known_model_change(name: &str) -> bool {
    ModelChangeId::VALUES.iter().any(|id| id.get_name() == name)
}

/// Replays one `.ffbr`; `keep_texts` also keeps every canonical text (for dumps).
pub fn replay_states(path: &Path, keep_texts: bool) -> ReplayStates {
    let mut out = ReplayStates { lines: Vec::new(), texts: Vec::new(), unknown: BTreeMap::new(), error: None };
    if let Err(e) = replay_into(path, keep_texts, &mut out) {
        out.error = Some(e);
    }
    out
}

fn replay_into(path: &Path, keep_texts: bool, out: &mut ReplayStates) -> Result<(), String> {
    let mut root = ffbr::read_ffbr(path)?;
    let commands = root.get_mut("commandArray").and_then(Value::as_array_mut).ok_or("no commandArray")?;
    let start_idx = commands
        .iter()
        .position(|c| ffbr::net_command_id(c) == "serverGameState")
        .ok_or("no serverGameState")?;
    if start_idx > 0 {
        return Err(format!("{start_idx} command(s) before the serverGameState (Java: unhandled)"));
    }
    let mut game_json = commands[start_idx].get("game").cloned().ok_or("serverGameState without game")?;
    let mut options = ffb_model::model::game_options::GameOptions::new();
    options.init_from(game_json.get("gameOptions").ok_or("gameOptions missing in serverGameState")?)?;
    if let Ok(rules) = options.rules_version() {
        drop_rules::drop_unknown_inducements(rules, &options, &mut game_json)?;
    }
    let mut game: Game = super::start_of_game::start_of_game(Game::from_java_json(&game_json)?)?;
    let processor = ModelChangeProcessor::new();

    for (i, cmd) in commands.iter_mut().enumerate().skip(start_idx + 1) {
        let net_command_id = ffbr::net_command_id(cmd).to_string();
        if net_command_id == "serverAddPlayer" {
            let command = super::add_player::ServerCommandAddPlayer::init_from(&game, cmd).map_err(|e| format!("command {i}: {e}"))?;
            super::add_player::handle_add_player(&mut game, command).map_err(|e| format!("command {i}: {e}"))?;
            continue;
        }
        if net_command_id != "serverModelSync" {
            return Err(format!("command {i}: {net_command_id} handler not ported yet"));
        }
        // Unknown ids dropped, then dropUnknownValues per kept change.
        let mut kept: Vec<Value> = Vec::new();
        if let Some(Value::Array(changes)) = cmd.pointer_mut("/modelChangeList/modelChangeArray") {
            for mut mc in changes.drain(..) {
                let id = mc.get("modelChangeId").and_then(Value::as_str).unwrap_or("").to_string();
                if !known_model_change(&id) {
                    *out.unknown.entry(id).or_default() += 1;
                    continue;
                }
                let (drop, keys) = drop_rules::drop_unknown_values(game.rules, &game.options, &mut mc)?;
                for k in keys {
                    *out.unknown.entry(k).or_default() += 1;
                }
                if !drop {
                    kept.push(mc);
                }
            }
        }
        // ServerCommandModelSync decode: ModelChangeList.initFrom decodes every value first.
        let mut decoded = Vec::with_capacity(kept.len());
        for mc in &kept {
            let change = ModelChange::init_from(mc).map_err(|e| format!("command {i}: {e}"))?;
            decoded.push(DecodedModelChange::decode(&game, &change).map_err(|e| format!("command {i}: {e}"))?);
        }
        // ModelChangeList.applyTo(game, IGNORE_PLAYER_MARKER).
        for change in &decoded {
            if matches!(change.change_id, ModelChangeId::FieldModelAddPlayerMarker | ModelChangeId::FieldModelRemovePlayerMarker) {
                continue;
            }
            processor.apply(&mut game, change).map_err(|e| format!("command {i}: apply {e}"))?;
        }
        let coin_choice = decoded.iter().any(|c| {
            c.change_id == ModelChangeId::GameSetDialogParameter
                && matches!(&c.value, ModelChangeValue::DialogParameter(p)
                    if matches!(**p, ffb_model::dialog::dialog_parameter_factory::DialogParameter::CoinChoice(_)))
        });
        if coin_choice {
            reset_inducements(&mut game.turn_data_away.inducement_set);
            reset_inducements(&mut game.turn_data_home.inducement_set);
        }
        let text = canonical::canonical_game_json(&game.to_json_value().map_err(|e| format!("command {i}: toJsonValue {e}"))?)?;
        out.lines.push(format!("{i} {}", canonical::sha256_hex(text.as_bytes())));
        if keep_texts {
            out.texts.push(text);
        }
    }
    Ok(())
}

/// `ffb-replay states <out_dir> <java_dir> <dir|file>...`: writes `<out_dir>/<id>.states` and
/// compares it with `<java_dir>/<id>.states`. For the first diverging sync of a replay the Rust
/// canonical text is written to `<out_dir>/<id>.<index>.rust.json` (compare it with the Java
/// `FFB_REPLAY_STATE_DUMP` of the same index).
pub fn states(out_dir: &Path, java_dir: &Path, inputs: &[PathBuf]) -> i32 {
    let replays = ffbr::collect_inputs(inputs);
    if let Err(e) = std::fs::create_dir_all(out_dir) {
        eprintln!("cannot create {}: {e}", out_dir.display());
        return 2;
    }
    let (mut identical, mut different, mut errors) = (0u64, 0u64, 0u64);
    let mut error_kinds: BTreeMap<String, u64> = BTreeMap::new();
    let mut syncs = 0u64;
    for (n, (id, path)) in replays.iter().enumerate() {
        let result = replay_states(path, false);
        syncs += result.lines.len() as u64;
        let text = result.lines.iter().map(|l| format!("{l}\n")).collect::<String>();
        let _ = std::fs::write(out_dir.join(format!("{id}.states")), &text);
        let java = std::fs::read_to_string(java_dir.join(format!("{id}.states"))).unwrap_or_default();
        let java_lines: Vec<&str> = java.lines().collect();
        let first_diff = result.lines.iter().zip(java_lines.iter()).position(|(r, j)| r != j);
        match (&result.error, first_diff) {
            (_, Some(pos)) => {
                different += 1;
                println!("DIFF {id}: line {pos} rust {:?} java {:?}", result.lines[pos], java_lines[pos]);
                // Re-run once keeping the texts to dump the diverging state.
                let again = replay_states(path, true);
                if let Some(t) = again.texts.get(pos) {
                    let index = result.lines[pos].split(' ').next().unwrap_or("x");
                    let _ = std::fs::write(out_dir.join(format!("{id}.{index}.rust.json")), t);
                }
            }
            (Some(e), None) => {
                errors += 1;
                println!("ERROR {id} (after {} identical lines of {}): {e}", result.lines.len(), java_lines.len());
                // the error kind without the command index
                let kind = e.split_once(": ").map_or(e.as_str(), |(_, k)| k);
                let kind: String = kind.chars().take(90).collect();
                *error_kinds.entry(kind).or_default() += 1;
            }
            (None, None) if result.lines.len() == java_lines.len() => identical += 1,
            (None, None) => {
                different += 1;
                println!("DIFF {id}: {} rust lines vs {} java lines", result.lines.len(), java_lines.len());
            }
        }
        if (n + 1) % 100 == 0 {
            eprintln!("  [{}] identical {identical} different {different} errors {errors}", n + 1);
        }
    }
    println!("error kinds:");
    for (k, c) in &error_kinds {
        println!("  {c:6} {k}");
    }
    println!(
        "states: {identical} identical, {different} different, {errors} errors, of {} replays ({syncs} rust state lines)",
        replays.len()
    );
    if different == 0 && errors == 0 && identical > 0 { 0 } else { 1 }
}
