//! The setup-formation record of the behavioural-cloning tooling (ml/setup_bc): one full snapshot
//! of a game at a kick-off setup. Shared by both binaries so the training data (`ffb-replay
//! setups`, human formations from replays) and the viewer's input (`ffb-parity --setup-scenario`,
//! two drafted teams before their first setup) go through the same code and carry the same
//! fields. Rust-only tooling: no Java counterpart.

use ffb_model::factory::skill_factory::SkillFactory;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::team::Team;
use ffb_model::model::turn_data::TurnData;
use serde_json::{json, Value};

/// The player-state bases a setup can meet; a player without a state yet (before the first
/// setup) is in reserve.
fn state_name(base: u32) -> &'static str {
    match base {
        0x01 => "STANDING",
        0x03 => "PRONE",
        0x04 => "STUNNED",
        0x05 => "KO",
        0x06 => "BH",
        0x07 => "SI",
        0x08 => "RIP",
        0x09 => "RESERVE",
        0x0a => "MISSING",
        0x0d => "BANNED",
        _ => "OTHER",
    }
}

fn player_json(game: &Game, p: &Player, side: &str) -> Value {
    let mut skills: Vec<String> = Vec::new();
    for sw in p.starting_skills.iter().chain(&p.extra_skills).chain(&p.temporary_skills) {
        let name = SkillFactory::name_in(game.rules, sw.skill_id).map_or_else(|| format!("{:?}", sw.skill_id), str::to_string);
        if !skills.contains(&name) {
            skills.push(name);
        }
    }
    skills.sort();
    let state = game.field_model.player_state(&p.id).map_or(0x09, |s| s.base());
    let (x, y) = match game.field_model.player_coordinate(&p.id) {
        Some(c) if c.x >= 0 && c.x < 26 && c.y >= 0 && c.y < 15 => (json!(c.x), json!(c.y)),
        _ => (Value::Null, Value::Null),
    };
    json!({
        "id": p.id,
        "side": side,
        "nr": p.nr,
        "position_id": p.position_id,
        "ma": p.movement_with_modifiers(),
        "st": p.strength_with_modifiers(),
        "ag": p.agility_with_modifiers(),
        "pa": p.passing_with_modifiers(),
        "av": p.armour_with_modifiers(),
        "skills": skills,
        "cost": p.position_cost,
        "niggling": p.niggling_injuries,
        "injuries": p.stat_injuries.iter().map(|k| format!("{k:?}")).collect::<Vec<_>>(),
        "big_guy": p.is_big_guy,
        "lineman": p.is_lineman,
        "state": state_name(state),
        "x": x,
        "y": y,
    })
}

fn team_json(team: &Team, td: &TurnData, score: i32) -> Value {
    json!({
        "race": team.race,
        "team_value": team.team_value,
        "rerolls": td.rerolls,
        "apothecaries": td.apothecaries + td.wandering_apothecaries,
        "bribes": td.inducement_set.get("bribes").map_or(0, |i| (i.value - i.uses).max(0)),
        "turn": td.turn_nr,
        "score": score,
    })
}

/// The snapshot record; `kind` is "defense" / "offense" / "scenario", `setting_home` the side
/// whose formation it is.
pub fn snapshot(game: &Game, replay_id: &str, command: usize, drive: u32, kind: &str, setting_home: bool) -> Value {
    let mut players: Vec<Value> = Vec::new();
    for (team, side) in [(&game.team_home, "home"), (&game.team_away, "away")] {
        let mut ps: Vec<&Player> = team.players.iter().collect();
        ps.sort_by_key(|p| p.nr);
        players.extend(ps.into_iter().map(|p| player_json(game, p, side)));
    }
    json!({
        "replay_id": replay_id,
        "command": command,
        "drive": drive,
        "kind": kind,
        "setting_home": setting_home,
        "edition": format!("{:?}", game.rules).to_lowercase(),
        "half": game.half,
        "weather": game.field_model.weather.name(),
        "home": team_json(&game.team_home, &game.turn_data_home, game.game_result.home.score),
        "away": team_json(&game.team_away, &game.turn_data_away, game.game_result.away.score),
        "players": players,
    })
}
