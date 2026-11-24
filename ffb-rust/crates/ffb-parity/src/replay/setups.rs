//! `ffb-replay setups <out.jsonl> <dir|file>...` -- the behavioural-cloning dataset of human
//! kick-off formations (ml/setup_bc). Rust-only tooling: no Java counterpart, not a parity gate.
//!
//! Every replay is replayed sync by sync (`states::replay_observe`). Within one `setup` turn mode
//! the kicking side sets up first (`setupOffense == false`); when it ends the server flips
//! `homePlaying` and sets `setupOffense`, and the receiving side sets up until the turn mode
//! leaves `setup`. One record (`setup_record::snapshot`) is written at each of those two moments:
//!   kind "defense": the kicking side's finished formation, the receivers still in reserve
//!   kind "offense": the receiving side's finished formation, facing the kickers' formation
//! `setting_home` names the side whose formation it is.

use super::setup_record::snapshot;
use super::{ffbr, states};
use ffb_model::model::game::Game;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

fn in_setup(game: &Game) -> bool {
    !game.turn_mode_null && game.turn_mode.name() == "setup"
}

/// Number of `side` players on the pitch in a record.
fn fielded(record: &Value, side: &str) -> usize {
    record["players"]
        .as_array()
        .map_or(0, |ps| ps.iter().filter(|p| p["side"] == side && !p["x"].is_null()).count())
}

/// The baseline the learned policy is measured against: what the heuristic agent's setup scorer
/// (`ffb_engine::agent::setup_heuristic`) would field in the same situation, always taking its
/// best-scored option. The side's players go back to reserve on a copy of the game and are then
/// placed one option at a time, exactly as the agent's `TeamSetup` prompt loop does.
/// Returns `[player id, x, y]` (pitch coordinates) per fielded player.
fn heuristic_formation(game: &Game, setting_home: bool, offense: bool) -> Vec<Value> {
    use ffb_engine::agent::setup_heuristic::setup_options;
    use ffb_engine::util::util_server_setup::UtilServerSetup;
    use ffb_model::enums::{TurnMode, PS_RESERVE};
    use ffb_model::types::FieldCoordinate;

    let mut g = game.clone();
    g.home_playing = setting_home;
    g.setup_offense = offense;
    g.turn_mode = TurnMode::Setup;
    let (team_id, ids): (String, Vec<String>) = {
        let team = if setting_home { &g.team_home } else { &g.team_away };
        (team.id.clone(), team.players.iter().map(|p| p.id.clone()).collect())
    };
    for (n, id) in ids.iter().enumerate() {
        if g.field_model.player_coordinate(id).is_some_and(|c| c.is_on_pitch()) {
            let state = g.field_model.player_state(id).unwrap_or(ffb_model::enums::PlayerState(PS_RESERVE));
            g.field_model.set_player_state(id, state.change_base(PS_RESERVE));
            g.field_model.set_player_coordinate(id, FieldCoordinate::new(if setting_home { -1 } else { 30 }, n as i32));
        }
    }
    for _ in 0..ids.len() {
        let Some((candidates, options)) = setup_options(&g, &team_id) else { break };
        // the first of the best-scored options, in the scorer's contract order
        let Some(best) = options.iter().reduce(|a, b| if b.w > a.w { b } else { a }) else { break };
        let id = candidates[best.player].id.clone();
        UtilServerSetup::setup_player(&mut g, &id, FieldCoordinate::new(best.x, best.y));
        if !g.field_model.player_coordinate(&id).is_some_and(|c| c.is_on_pitch()) {
            break;
        }
    }
    ids.iter()
        .filter_map(|id| {
            let c = g.field_model.player_coordinate(id)?;
            c.is_on_pitch().then(|| serde_json::json!([id, c.x, c.y]))
        })
        .collect()
}

fn record(game: &Game, id: &str, command: usize, drive: u32, kind: &str, setting_home: bool) -> Value {
    let mut r = snapshot(game, id, command, drive, kind, setting_home);
    r["heuristic"] = Value::Array(heuristic_formation(game, setting_home, kind == "offense"));
    r
}

/// The setup records of one replay, in order.
pub fn replay_setups(id: &str, path: &Path) -> (Vec<Value>, Option<String>) {
    let mut records: Vec<Value> = Vec::new();
    // (in setup, setupOffense, homePlaying) after the previous sync
    let mut prev: Option<(bool, bool, bool)> = None;
    let mut defender: Option<bool> = None;
    let mut drive = 0u32;
    let mut observer = |i: usize, game: &Game| {
        let now = (in_setup(game), game.setup_offense, game.home_playing);
        if let Some((was_setup, was_offense, was_home)) = prev {
            if now.0 && !was_setup {
                defender = None;
            }
            if now.0 && was_setup && !was_offense && now.1 && defender.is_none() {
                defender = Some(was_home);
                records.push(record(game, id, i, drive, "defense", was_home));
            }
            if !now.0 && was_setup {
                if let Some(d) = defender.take() {
                    records.push(record(game, id, i, drive, "offense", !d));
                    drive += 1;
                }
            }
        }
        prev = Some(now);
    };
    let result = states::replay_observe(path, &mut observer);
    (records, result.error)
}

pub fn setups(out: &Path, inputs: &[PathBuf]) -> i32 {
    let replays = ffbr::collect_inputs(inputs);
    let file = match std::fs::File::create(out) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("cannot create {}: {e}", out.display());
            return 2;
        }
    };
    let mut w = std::io::BufWriter::new(file);
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let (mut errors, mut empty) = (0u64, 0u64);
    for (n, (id, path)) in replays.iter().enumerate() {
        let (records, error) = replay_setups(id, path);
        if let Some(e) = error {
            errors += 1;
            println!("ERROR {id}: {e}");
        }
        for r in &records {
            let side = if r["setting_home"] == true { "home" } else { "away" };
            // nobody placed: the end-of-game artefact (setup entered, game over)
            if fielded(r, side) == 0 {
                empty += 1;
                continue;
            }
            *counts.entry(format!("{} {}", r["edition"].as_str().unwrap_or("?"), r["kind"].as_str().unwrap_or("?"))).or_default() += 1;
            if writeln!(w, "{r}").is_err() {
                eprintln!("write failed");
                return 2;
            }
        }
        if (n + 1) % 100 == 0 {
            eprintln!("  [{}] {} records, {errors} errors", n + 1, counts.values().sum::<u64>());
        }
    }
    println!("setups: {} replays, {errors} replay errors, {empty} empty formations dropped", replays.len());
    for (k, c) in &counts {
        println!("  {c:6} {k}");
    }
    if counts.is_empty() { 1 } else { 0 }
}
