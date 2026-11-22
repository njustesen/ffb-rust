//! `serverAddPlayer` in a replay -- 1:1 port of the harness's `FumbblReplayParser.handleAddPlayer`,
//! itself the model part of `ClientCommandHandlerAddPlayer.handleNetCommand` (ffb-client-logic),
//! with `ServerCommandAddPlayer.initFrom` as the decoder. Stars, mercenaries and raised players
//! enter the game this way (`startOfGame` removed the mercenaries and raised players).

use ffb_model::enums::{PlayerState, SendToBoxReason};
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::util::java_json;
use ffb_model::util::util_box::UtilBox;
use serde_json::Value;

/// The decoded `ServerCommandAddPlayer`.
pub struct ServerCommandAddPlayer {
    pub team_id: Option<String>,
    pub player: Option<Player>,
    pub player_state: Option<PlayerState>,
    pub send_to_box_reason: Option<SendToBoxReason>,
    pub send_to_box_turn: i32,
    pub send_to_box_half: i32,
}

impl ServerCommandAddPlayer {
    /// Java: `ServerCommandAddPlayer.initFrom(source, json)`: the player is
    /// `new RosterPlayer().initFrom(..)` when the `player` object is present.
    pub fn init_from(game: &Game, json: &Value) -> Result<Self, String> {
        let o = java_json::to_json_object(json)?;
        let team_id = java_json::get_string(o, "teamId")?;
        let player = match java_json::get_object(o, "player")? {
            Some(_) => {
                let mut player = Player::default();
                player.init_from(game.rules, &o["player"])?;
                Some(player)
            }
            None => None,
        };
        let player_state = java_json::get_player_state(o, "playerState")?;
        let send_to_box_reason = java_json::get_enum_name(o, "sendToBoxReason")?.and_then(SendToBoxReason::from_name);
        let send_to_box_turn = java_json::get_int(o, "sendToBoxTurn")?;
        let send_to_box_half = java_json::get_int(o, "sendToBoxHalf")?;
        Ok(ServerCommandAddPlayer { team_id, player, player_state, send_to_box_reason, send_to_box_turn, send_to_box_half })
    }
}

/// `FumbblReplayParser.handleAddPlayer(game, command)`.
pub fn handle_add_player(game: &mut Game, command: ServerCommandAddPlayer) -> Result<(), String> {
    let new_player = command.player.ok_or("serverAddPlayer without player (Java NPE)")?;
    let player_id = new_player.id.clone();
    // team = home if its id equals the command's team id, else away.
    let home = command.team_id.as_deref() == Some(game.team_home.id.as_str());
    let rules = game.rules;
    let team = if home { &mut game.team_home } else { &mut game.team_away };
    if team.has_player(&player_id) {
        // oldPlayer.init(command.getPlayer(), rules) -- every Rust player is a RosterPlayer
        // (init_from refuses ZappedPlayer JSON, the only other kind).
        let old = team.player_mut(&player_id).ok_or("serverAddPlayer: old player vanished")?;
        old.init_roster_player(&new_player);
        // Rust-only cache refresh of the position copies (no Java-visible change: the player's
        // icon index was already resolved when the team was loaded).
        let position_id = old.position_id.clone();
        if let Some(roster) = team.roster.as_mut() {
            let old = team.players.iter_mut().find(|p| p.id == player_id).ok_or("serverAddPlayer: old player vanished")?;
            old.update_position_json(roster.position_mut(&position_id));
        }
    } else {
        // team.addPlayer(player); player.updatePosition(team.getRoster().getPositionById(..), ..)
        let position_id = new_player.position_id.clone();
        team.add_player(new_player)?;
        let roster = team.roster.as_mut().ok_or("serverAddPlayer: team without roster (Java NPE)")?;
        let player = team.players.iter_mut().find(|p| p.id == player_id).ok_or("serverAddPlayer: added player vanished")?;
        player.update_position_with_stats(rules, roster.position_mut(&position_id))?;
    }
    // game.getFieldModel().setPlayerState(player, state): stored when there is no state yet, or
    // when the new one is non-null with a different id.
    let old_state = game.field_model.player_states.get(&player_id).copied();
    match (old_state, command.player_state) {
        (None, None) => return Err("serverAddPlayer: null player state stored as a null entry (not representable)".into()),
        (None, Some(state)) => {
            game.field_model.player_states.insert(player_id.clone(), state);
        }
        (Some(old), Some(state)) if old.0 != state.0 => {
            game.field_model.player_states.insert(player_id.clone(), state);
        }
        _ => {}
    }
    UtilBox::put_player_into_box(game, &player_id);
    // game.getGameResult().getPlayerResult(player): home when the home team holds the id.
    let result_home = game.team_home.has_player(&player_id);
    let player_result = game.game_result.team_result_mut(result_home).player_result_mut(&player_id);
    player_result.send_to_box_reason = command.send_to_box_reason;
    player_result.send_to_box_turn = command.send_to_box_turn;
    player_result.send_to_box_half = command.send_to_box_half;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, Rules, SeriousInjuryKind, PS_RESERVE};
    use ffb_model::model::roster::Roster;
    use ffb_model::model::roster_position::RosterPosition;
    use ffb_model::model::skill_def::{SkillId, SkillWithValue};
    use ffb_model::model::team::Team;
    use ffb_model::types::{FieldCoordinate, RSV_AWAY_X};

    fn game_with_roster(rules: Rules) -> Game {
        let mut game = Game::new(Team::new(), Team::new(), rules);
        game.team_home.id = "home".into();
        game.team_away.id = "away".into();
        let mut position = RosterPosition::default();
        position.id = "star".into();
        position.movement = 6;
        position.strength = 4;
        position.agility = 3;
        position.passing = 4;
        position.armour = 9;
        position.nr_of_icons = 2;
        position.current_icon_set_index = -1;
        position.skills = vec![SkillWithValue::new(SkillId::Block), SkillWithValue::new(SkillId::MovementIncrease)];
        let mut roster = Roster::default();
        roster.positions.push(position);
        game.team_away.roster = Some(Box::new(roster));
        game.team_away.players.push(Player { id: "a1".into(), nr: 1, ..Default::default() });
        game.field_model.player_states.insert("a1".into(), PlayerState::new(PS_RESERVE));
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(RSV_AWAY_X, 0));
        game
    }

    fn command(player: Player) -> ServerCommandAddPlayer {
        ServerCommandAddPlayer {
            team_id: Some("away".into()),
            player: Some(player),
            player_state: Some(PlayerState::new(PS_RESERVE)),
            send_to_box_reason: None,
            send_to_box_turn: 0,
            send_to_box_half: 0,
        }
    }

    /// Expectations read off `ClientCommandHandlerAddPlayer` + `RosterPlayer.updatePosition`.
    #[test]
    fn new_player_takes_position_stats_and_skills_and_is_boxed() {
        let mut game = game_with_roster(Rules::Bb2025);
        let mut star = Player { id: "s1".into(), nr: 20, position_id: "star".into(), player_type: PlayerType::Star, ..Default::default() };
        star.icon_set_index = -1;
        star.movement = 1; // overwritten by the position
        star.extra_skills = vec![SkillWithValue::new(SkillId::Block), SkillWithValue::new(SkillId::Dodge)];
        star.lasting_injuries = vec![Some(SeriousInjuryKind::DislocatedHipAg)];
        handle_add_player(&mut game, command(star)).unwrap();
        let s1 = game.team_away.player("s1").unwrap();
        // bb2025 AG injury: min(3 + 1, 6)
        assert_eq!((s1.movement, s1.strength, s1.agility, s1.passing, s1.armour), (6, 4, 4, 4, 9));
        let skills: Vec<SkillId> = s1.extra_skills.iter().map(|sw| sw.skill_id).collect();
        // Block already there (not re-added), +MA appended (stat increase: always appended)
        assert_eq!(skills, [SkillId::Block, SkillId::Dodge, SkillId::MovementIncrease]);
        assert_eq!(s1.icon_set_index, 0);
        assert_eq!(game.field_model.player_coordinate("s1"), Some(FieldCoordinate::new(RSV_AWAY_X, 1)));
        assert!(game.game_result.away.player_result("s1").is_some());
    }

    #[test]
    fn known_player_is_reinitialised_in_place() {
        let mut game = game_with_roster(Rules::Bb2016);
        let mut again = Player { id: "a1".into(), nr: 99, movement: 7, ..Default::default() };
        again.extra_skills = vec![SkillWithValue::new(SkillId::Block)];
        handle_add_player(&mut game, command(again)).unwrap();
        let a1 = game.team_away.player("a1").unwrap();
        assert_eq!((a1.nr, a1.movement), (1, 7), "nr stays, stats are copied");
        assert_eq!(a1.extra_skills.len(), 1);
        assert_eq!(game.field_model.player_coordinate("a1"), Some(FieldCoordinate::new(RSV_AWAY_X, 0)));
    }

    #[test]
    fn bb2016_injury_reduction_stops_at_two() {
        let mut game = game_with_roster(Rules::Bb2016);
        let mut star = Player { id: "s1".into(), nr: 20, position_id: "star".into(), ..Default::default() };
        // bb2016 SMASHED_ANKLE = MA
        star.lasting_injuries = vec![Some(SeriousInjuryKind::SmashedAnkle); 3];
        handle_add_player(&mut game, command(star)).unwrap();
        // 6 - 1 - 1, then (6 - 4) < 2 fails
        assert_eq!(game.team_away.player("s1").unwrap().movement, 4);
    }
}
