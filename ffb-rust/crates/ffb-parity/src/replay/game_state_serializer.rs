//! 1:1 port of the harness's `GameStateSerializer.serialize(Game)`
//! (`ffb-java/ffb/ffb-ai/src/main/java/com/fumbbl/ffb/ai/simulation/GameStateSerializer.java`) --
//! the `state` object of every §O action record. The result keeps minimal-json's member order
//! (insertion order, [`CanonicalJson::Object`]), so `to_json_string` prints Java's `toString()`.
//!
//! Java getters, as the replay's model holds them: `RosterPlayer.getMovement()` .. are the RAW
//! fields (no temporary modifiers), `getSkills()` is the player's own skill list (no temporary
//! skills), a `ZappedPlayer` answers with its `ZappedPosition` (frog stats + six skills) and the
//! original's nr and lasting injuries; `FieldModel.getPlayerState` never returns null (an absent
//! state is `new PlayerState(UNKNOWN)`, which prints `OTHER`).

use super::canonical::CanonicalJson;
use super::java_names;
use super::replay_report::constant_of;
use ffb_mechanics::spp_mechanic::SppMechanic as SppMechanicTrait;
use ffb_model::enums::{PlayerState, Rules};
use ffb_model::factory::serious_injury_factory;
use ffb_model::factory::skill_factory::SkillFactory;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::team::Team;
use ffb_model::model::turn_data::TurnData;
use ffb_model::types::FieldCoordinate;

/// minimal-json `JsonObject` built with `add` (insertion order, duplicates kept).
#[derive(Default)]
pub struct Obj(Vec<(String, CanonicalJson)>);

impl Obj {
    pub fn new() -> Self {
        Obj(Vec::new())
    }
    pub fn add(&mut self, name: &str, value: CanonicalJson) -> &mut Self {
        self.0.push((name.to_string(), value));
        self
    }
    pub fn int(&mut self, name: &str, v: i32) -> &mut Self {
        self.add(name, int(v))
    }
    pub fn bool(&mut self, name: &str, v: bool) -> &mut Self {
        self.add(name, CanonicalJson::Bool(v))
    }
    pub fn str(&mut self, name: &str, v: &str) -> &mut Self {
        self.add(name, CanonicalJson::String(v.to_string()))
    }
    /// `JsonValue.valueOf(String)` with null -> `JsonValue.NULL`.
    pub fn str_or_null(&mut self, name: &str, v: Option<&str>) -> &mut Self {
        self.add(name, v.map_or(CanonicalJson::Null, |s| CanonicalJson::String(s.to_string())))
    }
    pub fn null(&mut self, name: &str) -> &mut Self {
        self.add(name, CanonicalJson::Null)
    }
    pub fn build(self) -> CanonicalJson {
        CanonicalJson::Object(self.0)
    }
}

pub fn int(v: i32) -> CanonicalJson {
    CanonicalJson::Number(v.to_string())
}

pub fn string(v: &str) -> CanonicalJson {
    CanonicalJson::String(v.to_string())
}

pub const BALL_CARRIED: &str = "CARRIED";
pub const BALL_ON_GROUND: &str = "ON_GROUND";
pub const BALL_IN_AIR: &str = "IN_AIR";
pub const BALL_BOUNCING: &str = "BOUNCING";

/// Java `FieldModel.getPlayer(coordinate)`: the last player to arrive on the square, then
/// `game.getPlayerById` (null when that id is in neither team).
pub fn field_player_at<'a>(game: &'a Game, coord: FieldCoordinate) -> Option<&'a Player> {
    game.field_model.player_at(coord).and_then(|id| game.player(id))
}

/// Java `FieldModel.getPlayerState(player)`: never null.
pub fn field_player_state(game: &Game, player_id: &str) -> PlayerState {
    game.field_model.player_state(player_id).unwrap_or(PlayerState(0))
}

/// Java `Team.getPlayers()`: sorted by nr.
pub fn team_players(team: &Team) -> Vec<&Player> {
    let mut players: Vec<&Player> = team.players.iter().collect();
    players.sort_by_key(|p| p.nr);
    players
}

/// Java `GameStateSerializer.serialize(game)`.
pub fn serialize(game: &Game) -> Result<CanonicalJson, String> {
    let mut out = Obj::new();
    out.int("half", game.half);
    let td = game.turn_data();
    out.int("turn_nr", td.turn_nr);
    if game.turn_mode_null {
        out.str("turn_mode", "UNKNOWN");
    } else {
        out.str("turn_mode", constant_of(java_names::TURN_MODE, game.turn_mode.name())?);
    }
    out.bool("home_playing", game.home_playing);
    out.str("weather", constant_of(java_names::WEATHER, game.field_model.weather.name())?);
    out.add("ball", serialize_ball(game));
    out.add("board", serialize_board(game));
    out.add("players", serialize_players(game)?);
    out.add("home", serialize_team_data(game, &game.team_home, &game.turn_data_home, true)?);
    out.add("away", serialize_team_data(game, &game.team_away, &game.turn_data_away, false)?);
    out.int("score_home", game.game_result.home.score);
    out.int("score_away", game.game_result.away.score);
    out.bool("blitz_used", td.blitz_used);
    out.bool("foul_used", td.foul_used);
    out.bool("pass_used", td.pass_used);
    out.bool("handoff_used", td.hand_over_used);
    out.bool("ttm_used", td.ttm_used);
    out.bool("reroll_used", td.reroll_used);
    out.str_or_null("defender_id", game.defender_id.as_deref());
    out.str_or_null("thrower_id", game.thrower_id.as_deref());
    out.bool("first_turn_home", game.turn_data_home.first_turn_after_kickoff);
    out.bool("first_turn_away", game.turn_data_away.first_turn_after_kickoff);
    out.add("acting_player", serialize_acting_player(game)?);
    Ok(out.build())
}

fn serialize_ball(game: &Game) -> CanonicalJson {
    let fm = &game.field_model;
    let mut ball = Obj::new();
    let ball_coord = fm.ball_coordinate;
    let mut carrier_id: Option<&str> = None;
    let state = if let Some(landing) = game.pass_coordinate {
        ball.int("target_x", landing.x);
        ball.int("target_y", landing.y);
        BALL_IN_AIR
    } else if fm.ball_moving {
        BALL_BOUNCING
    } else if let Some(bc) = ball_coord {
        match field_player_at(game, bc) {
            Some(p) => {
                carrier_id = Some(p.id.as_str());
                BALL_CARRIED
            }
            None => BALL_ON_GROUND,
        }
    } else {
        BALL_ON_GROUND
    };
    ball.str("state", state);
    match ball_coord {
        Some(bc) => {
            ball.int("x", bc.x);
            ball.int("y", bc.y);
        }
        None => {
            ball.null("x");
            ball.null("y");
        }
    }
    ball.str_or_null("carrier_id", carrier_id);
    ball.build()
}

fn serialize_board(game: &Game) -> CanonicalJson {
    let mut board = Vec::with_capacity(26 * 15);
    for x in 0..26 {
        for y in 0..15 {
            board.push(match field_player_at(game, FieldCoordinate::new(x, y)) {
                Some(p) => string(&p.id),
                None => CanonicalJson::Null,
            });
        }
    }
    CanonicalJson::Array(board)
}

fn serialize_players(game: &Game) -> Result<CanonicalJson, String> {
    let mut players = Obj::new();
    for (team, key) in [(&game.team_home, "home"), (&game.team_away, "away")] {
        for p in team_players(team) {
            players.add(&p.id, serialize_player(game, p, key)?);
        }
    }
    Ok(players.build())
}

/// The six `ZappedPosition` skills (`ZappedPosition(RosterPosition, IFactorySource)`): Dodge, the
/// ruleset's `preventHoldBall` skill (bb2016/bb2020 `NoHands` "No Hands", bb2025 `NoBall` "No Ball"),
/// Titchy, Stunty, Very Long Legs, Leap -- `forName` of the game's factory (null if unknown).
fn zapped_position_skill_names(rules: Rules) -> Vec<Option<&'static str>> {
    let prevent_hold_ball = if rules == Rules::Bb2025 { "No Ball" } else { "No Hands" };
    ["Dodge", prevent_hold_ball, "Titchy", "Stunty", "Very Long Legs", "Leap"]
        .iter()
        .map(|n| SkillFactory::for_name_in(rules, n).and_then(|id| SkillFactory::name_in(rules, id)))
        .collect()
}

/// Java `RosterPlayer.getSkills()` names: `fSkills` in list order (the order `to_json_value`
/// writes `skillArray`, nulls at `unknown_skill_slots`). A null element NPEs on `getName()`.
fn roster_skill_names(game: &Game, p: &Player) -> Result<Vec<&'static str>, String> {
    let name = |id| SkillFactory::name_in(game.rules, id).ok_or_else(|| format!("player {}: skill {id:?} not in the ruleset", p.id));
    let mut names: Vec<Option<&'static str>> = Vec::new();
    for sw in &p.extra_skills {
        names.push(Some(name(sw.skill_id)?));
    }
    for sw in &p.starting_skills {
        if !p.extra_skills.iter().any(|e| e.skill_id == sw.skill_id) {
            names.push(Some(name(sw.skill_id)?));
        }
    }
    for &slot in &p.unknown_skill_slots {
        if slot > names.len() {
            return Err(format!("player {}: unknown skill slot {slot}", p.id));
        }
        names.insert(slot, None);
    }
    names
        .into_iter()
        .map(|n| n.ok_or_else(|| format!("NullPointerException: player {} has a null skill", p.id)))
        .collect()
}

/// Java `player.getSkills()` names for any team player (a zapped one answers with its
/// `ZappedPosition`'s skills); a null skill NPEs on `getName()`.
pub fn java_skill_names(game: &Game, p: &Player) -> Result<Vec<&'static str>, String> {
    if game.zapped_players.iter().any(|z| z.original_player.id == p.id) {
        zapped_position_skill_names(game.rules)
            .into_iter()
            .map(|n| n.ok_or_else(|| "NullPointerException: zapped position skill unknown to the ruleset".to_string()))
            .collect()
    } else {
        roster_skill_names(game, p)
    }
}

fn lasting_injury_names(game: &Game, p: &Player) -> Result<Vec<String>, String> {
    p.lasting_injuries
        .iter()
        .map(|li| {
            let kind = li.ok_or_else(|| format!("NullPointerException: player {} has a null lasting injury", p.id))?;
            serious_injury_factory::name_of_kind_in(game.rules, kind)
                .ok_or_else(|| format!("player {}: injury {kind:?} not in the ruleset", p.id))
        })
        .collect()
}

fn serialize_player(game: &Game, p: &Player, team_key: &str) -> Result<CanonicalJson, String> {
    let zapped = game.zapped_players.iter().find(|z| z.original_player.id == p.id);
    let mut obj = Obj::new();
    obj.str("team", team_key);
    let (skills, lasting): (Vec<&'static str>, Vec<String>) = match zapped {
        Some(z) => {
            obj.int("nr", z.original_player.nr);
            obj.int("ma", z.position.movement);
            obj.int("st", z.position.strength);
            obj.int("ag", z.position.agility);
            obj.int("av", z.position.armour);
            obj.int("pa", z.position.passing);
            (java_skill_names(game, p)?, lasting_injury_names(game, &z.original_player)?)
        }
        None => {
            obj.int("nr", p.nr);
            obj.int("ma", p.movement);
            obj.int("st", p.strength);
            obj.int("ag", p.agility);
            obj.int("av", p.armour);
            obj.int("pa", p.passing);
            (roster_skill_names(game, p)?, lasting_injury_names(game, p)?)
        }
    };
    obj.add("skills", CanonicalJson::Array(skills.iter().map(|s| string(s)).collect()));
    match game.field_model.player_coordinate(&p.id) {
        Some(c) if c.x >= 0 && c.x < 26 && c.y >= 0 && c.y < 15 => {
            obj.int("x", c.x);
            obj.int("y", c.y);
        }
        _ => {
            obj.null("x");
            obj.null("y");
        }
    }
    let ps = field_player_state(game, &p.id);
    obj.str("state", player_state_string(ps.base()));
    obj.bool("active", ps.is_active());
    obj.add("lasting_injuries", CanonicalJson::Array(lasting.iter().map(|s| string(s)).collect()));
    Ok(obj.build())
}

/// Java `GameStateSerializer.playerStateString(base)`.
pub fn player_state_string(base: u32) -> &'static str {
    match base {
        0x01 => "STANDING",
        0x02 => "MOVING",
        0x03 => "PRONE",
        0x04 => "STUNNED",
        0x05 => "KO",
        0x06 => "BH",
        0x07 => "SI",
        0x08 => "RIP",
        0x09 => "RESERVE",
        0x0a => "MISSING",
        0x0b => "FALLING",
        0x0c => "BLOCKED",
        0x0d => "BANNED",
        0x0e => "EXHAUSTED",
        _ => "OTHER",
    }
}

/// `PlayerResult.totalEarnedSpps()` with the game's `SppMechanic`.
fn total_earned_spps(rules: Rules, team: &Team, r: &ffb_model::model::PlayerResult) -> i32 {
    fn total(spp: &dyn SppMechanicTrait, team: &Team, r: &ffb_model::model::PlayerResult) -> i32 {
        r.player_awards * spp.mvp_spp()
            + r.touchdowns * spp.touchdown_spp(team)
            + r.casualties * spp.casualty_spp(team)
            + r.interceptions * spp.interception_spp()
            + r.completions * spp.completion_spp()
            + r.deflections * spp.deflection_spp()
            + r.completions_with_additional_spp * spp.additional_completion_spp()
            + r.casualties_with_additional_spp * spp.additional_casualty_spp()
            + r.catches_with_additional_spp * spp.additional_catch_spp()
            + r.landings * spp.landing_spp()
    }
    match rules {
        Rules::Bb2016 | Rules::Common => total(&ffb_mechanics::bb2016::spp_mechanic::SppMechanic, team, r),
        Rules::Bb2020 => total(&ffb_mechanics::bb2020::spp_mechanic::SppMechanic, team, r),
        Rules::Bb2025 => total(&ffb_mechanics::bb2025::spp_mechanic::SppMechanic, team, r),
    }
}

fn serialize_team_data(game: &Game, team: &Team, td: &TurnData, is_home: bool) -> Result<CanonicalJson, String> {
    let mut obj = Obj::new();
    obj.str("team_id", &team.id);
    obj.str("race", &team.race);
    obj.int("fan_factor", team.fan_factor);
    obj.int("dedicated_fans", team.dedicated_fans);
    obj.int("rerolls", td.rerolls);
    obj.int("turn", td.turn_nr);
    obj.int("apothecaries", td.apothecaries + td.wandering_apothecaries);
    obj.int("bribes", td.inducement_set.get("bribes").map_or(0, |i| (i.value - i.uses).max(0)));
    let tr = if is_home { &game.game_result.home } else { &game.game_result.away };
    let mut spp_so_far = 0;
    for p in team_players(team) {
        if let Some(r) = tr.player_results.get(&p.id) {
            spp_so_far += total_earned_spps(game.rules, team, r);
        }
    }
    obj.int("spp_earned", spp_so_far);
    Ok(obj.build())
}

fn serialize_acting_player(game: &Game) -> Result<CanonicalJson, String> {
    let ap = &game.acting_player;
    let mut obj = Obj::new();
    let Some(pid) = ap.player_id.as_deref() else {
        obj.null("player_id");
        obj.null("action");
        obj.int("current_move", 0);
        obj.bool("has_moved", false);
        obj.bool("has_blocked", false);
        obj.bool("has_fouled", false);
        obj.bool("has_passed", false);
        return Ok(obj.build());
    };
    obj.str("player_id", pid);
    match ap.player_action {
        Some(a) => obj.str("action", constant_of(java_names::PLAYER_ACTION, a.name())?),
        None => obj.null("action"),
    };
    obj.int("current_move", ap.current_move);
    obj.bool("has_moved", ap.has_moved);
    obj.bool("has_blocked", ap.has_blocked);
    obj.bool("has_fouled", ap.has_fouled);
    obj.bool("has_passed", ap.has_passed);
    Ok(obj.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_strings_follow_the_java_switch() {
        assert_eq!(player_state_string(0), "OTHER"); // UNKNOWN: the absent-state default
        assert_eq!(player_state_string(5), "KO");
        assert_eq!(player_state_string(0x0f), "OTHER"); // BEING_DRAGGED
    }

    #[test]
    fn zapped_position_skills_per_ruleset() {
        let b25 = zapped_position_skill_names(Rules::Bb2025);
        assert_eq!(b25[1], Some("No Ball"));
        let b20 = zapped_position_skill_names(Rules::Bb2020);
        assert_eq!(b20[0], Some("Dodge"));
        assert_eq!(b20[1], Some("No Hands"));
    }

    #[test]
    fn insertion_order_and_duplicates_kept() {
        let mut o = Obj::new();
        o.int("b", 1).str_or_null("a", None).bool("b", true);
        assert_eq!(o.build().to_json_string(), "{\"b\":1,\"a\":null,\"b\":true}");
    }
}
