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
fn load_subtree(subtree: &str, game: &mut Value) -> Result<Value, String> {
    // Game.initFrom parses the options first and initialises the rules from them; every other
    // class is decoded with that ruleset's factories.
    let mut options = ffb_model::model::game_options::GameOptions::new();
    options.init_from(game.get("gameOptions").ok_or("gameOptions missing in serverGameState")?)?;
    // The parser's drop rules on the serverGameState JSON, as before the Java decode.
    if let Ok(rules) = options.rules_version() {
        super::drop_rules::drop_unknown_inducements(rules, &options, game)?;
    }
    let input = game.get(subtree).ok_or_else(|| format!("{subtree} missing in serverGameState"))?;
    match subtree {
        "turnDataHome" | "turnDataAway" => {
            let rules = options.rules_version()?;
            let mut turn_data = ffb_model::model::turn_data::TurnData::new();
            turn_data.init_from(rules, &options, input)?;
            turn_data.to_json_value(rules)
        }
        "teamHome" | "teamAway" => {
            let rules = options.rules_version()?;
            let mut team = ffb_model::model::team::Team::new();
            team.init_from(rules, input)?;
            team.to_json_value(rules)
        }
        "gameResult" => {
            // Game.initFrom decodes both teams (in place) before the game result, whose
            // TeamResults resolve their players through them.
            let rules = options.rules_version()?;
            let team = |key: &str| -> Result<ffb_model::model::team::Team, String> {
                let mut team = ffb_model::model::team::Team::new();
                team.init_from(rules, game.get(key).ok_or_else(|| format!("{key} missing in serverGameState"))?)?;
                Ok(team)
            };
            let (team_home, team_away) = (team("teamHome")?, team("teamAway")?);
            let mut game_result = ffb_model::model::game_result::GameResult::default();
            game_result.init_from(rules, &team_home, &team_away, input)?;
            game_result.to_json_value(rules, Some(&team_home), Some(&team_away))
        }
        "fieldModel" => {
            // Game.initFrom decodes both teams before the field model, whose player data attach
            // to the game's players (getPlayerById) and are written per getPlayers().
            let rules = options.rules_version()?;
            let team = |key: &str| -> Result<ffb_model::model::team::Team, String> {
                let mut team = ffb_model::model::team::Team::new();
                team.init_from(rules, game.get(key).ok_or_else(|| format!("{key} missing in serverGameState"))?)?;
                Ok(team)
            };
            let (team_home, team_away) = (team("teamHome")?, team("teamAway")?);
            let mut field_model = ffb_model::model::field_model::FieldModel::new();
            field_model.init_from(rules, &team_home, &team_away, input)?;
            field_model.to_json_value(rules, &team_home, &team_away)
        }
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

/// `Game.initFrom`'s dialog parameter: `JsonObjectOption.getFrom` (absent or null -> null, a
/// non-object throws), then `new DialogParameterFactory().forJsonValue(source, object)`; the Java
/// `toJsonValue` of the result, `None` when the game has no parameter.
fn load_dialog_parameter(game: &mut Value) -> Result<Option<Value>, String> {
    let mut options = ffb_model::model::game_options::GameOptions::new();
    options.init_from(game.get("gameOptions").ok_or("gameOptions missing in serverGameState")?)?;
    let rules = options.rules_version()?;
    let obj = ffb_model::util::java_json::to_json_object(game)?;
    let Some(dialog_object) = ffb_model::util::java_json::get_object(obj, "dialogParameter")? else {
        return Ok(None);
    };
    let factory = ffb_model::dialog::dialog_parameter_factory::DialogParameterFactory::new();
    match factory.for_json_value(rules, &Value::Object(dialog_object.clone()))? {
        None => Ok(None),
        Some(p) => p.to_json_value(rules).map(Some),
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
            let mut game = game_state_json(path)?;
            let java_file = java_dir.join(format!("{id}.gs.json"));
            let java_text = std::fs::read_to_string(&java_file).map_err(|e| format!("{}: {e}", java_file.display()))?;
            let java: Value = serde_json::from_str(&java_text).map_err(|e| format!("java json: {e}"))?;
            if subtree == "dialogParameter" {
                // Game.toJsonValue writes the key only for a non-null parameter.
                let rust = load_dialog_parameter(&mut game)?;
                let text = |v: Option<&Value>| -> Result<String, String> {
                    Ok(match v {
                        None => "<absent>".to_string(),
                        Some(v) => canonical::canonical_copy(v, Some(subtree))?.to_json_string(),
                    })
                };
                return Ok((text(rust.as_ref())?, text(java.get(subtree))?));
            }
            let rust = load_subtree(subtree, &mut game)?;
            let rust_text = canonical::canonical_copy(&rust, Some(subtree))?.to_json_string();
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
