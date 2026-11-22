//! The start-of-game position of a replay -- 1:1 port of the harness's
//! `FumbblReplayParser.startOfGame` + `addTeam` (ffb-ai), which are themselves 1:1 copies of
//! `ClientReplayer.createGame` + `addTeam` (ffb-client-logic).
//!
//! Input = the replay's END-of-game `Game` (the decoded serverGameState); the two `Team` objects
//! are MOVED into the new game (Java hands the same objects over), stripped of mercenaries and
//! raised players (they come back through serverAddPlayer), every remaining player boxed as a
//! reserve (MISSING when recovering from an injury).

use ffb_model::dialog::dialog_parameter_factory::DialogParameter;
use ffb_model::dialog::dialog_start_game_parameter::DialogStartGameParameter;
use ffb_model::enums::{PlayerState, PlayerType, SendToBoxReason, TurnMode, Weather, PS_MISSING, PS_RESERVE};
use ffb_model::model::game::Game;
use ffb_model::model::game_result::{GameResult, TeamResult};
use ffb_model::model::team::Team;
use ffb_model::model::turn_data::TurnData;
use ffb_model::util::util_box::UtilBox;

/// Java `new TeamResult(gameResult, home)`: `penaltyScore = -1`, everything else zero/empty.
fn new_team_result() -> TeamResult {
    TeamResult { penalty_score: -1, ..TeamResult::default() }
}

/// Java `new Game(source, manager)`: every scalar at its Java field default (`fHomePlaying = true`,
/// `fTurnMode`/`fLastTurnMode` null), fresh field model / turn data / acting player / result.
fn new_java_game(old: &Game) -> Game {
    let mut game = Game::new(Team::new(), Team::new(), old.rules);
    game.id = 0;
    game.half = 0;
    game.home_playing = true;
    game.home_first_offense = false;
    game.setup_offense = false;
    game.last_turn_mode = None;
    game.scheduled = None;
    game.started = None;
    game.finished = None;
    game.waiting_for_opponent = false;
    game.timeout_possible = false;
    game.timeout_enforced = false;
    game.concession_possible = false;
    game.conceded_legally = false;
    game.testing = false;
    game.turn_time = 0;
    game.game_time = 0;
    game.defender_id = None;
    game.defender_action = None;
    game.last_defender_id = None;
    game.thrower_id = None;
    game.thrower_action = None;
    game.pass_coordinate = None;
    game.dialog_parameter = None;
    game.dialog_id = None;
    let mut turn_data_home = TurnData::new();
    turn_data_home.home_data = true;
    game.turn_data_home = turn_data_home;
    let mut turn_data_away = TurnData::new();
    turn_data_away.home_data = false;
    game.turn_data_away = turn_data_away;
    game.game_result = GameResult { home: new_team_result(), away: new_team_result() };
    game
}

/// `FumbblReplayParser.startOfGame(oldGame)`.
pub fn start_of_game(mut old: Game) -> Result<Game, String> {
    let mut game = new_java_game(&old);
    game.id = old.id;
    game.turn_time = old.turn_time;
    game.game_time = old.game_time;
    game.home_playing = true;
    // setTurnMode(START_GAME): the old mode is null on a new Game, so lastTurnMode stays null.
    game.turn_mode = TurnMode::StartGame;
    let parameter = DialogParameter::StartGame(DialogStartGameParameter::default());
    game.dialog_id = Some(parameter.get_id());
    game.dialog_parameter = Some(parameter);
    game.field_model.weather = Weather::Nice;
    // getOptions().init(oldGame.getOptions()): createGameOption + setValue(getValueAsString) +
    // addOption per option -- the old options are already normalised and mutually consistent.
    game.options = old.options.clone();
    let old_game_result = std::mem::take(&mut old.game_result);
    let team_home = std::mem::replace(&mut old.team_home, Team::new());
    let team_away = std::mem::replace(&mut old.team_away, Team::new());
    add_team(&mut game, team_home, &old_game_result.home, true)?;
    add_team(&mut game, team_away, &old_game_result.away, false)?;
    // Both inducement sets: initFrom(new InducementSet().toJsonValue()) on the fresh sets -> empty.
    game.turn_data_home.inducement_set = TurnData::new().inducement_set;
    game.turn_data_away.inducement_set = TurnData::new().inducement_set;
    // game.initializeRules(): no state.
    Ok(game)
}

/// `FumbblReplayParser.addTeam(game, team, oldTeamResult, home)`.
fn add_team(game: &mut Game, team: Team, old_team_result: &TeamResult, home: bool) -> Result<(), String> {
    // Player<?>[] players = pTeam.getPlayers() -- sorted by nr, taken BEFORE any removal.
    // (the Rust team holds one player per nr, as Java's fPlayerByNr)
    let mut by_nr: Vec<&ffb_model::model::player::Player> = team.players.iter().collect();
    by_nr.sort_by_key(|p| p.nr);
    let players: Vec<(String, PlayerType, bool)> =
        by_nr.iter().map(|p| (p.id.clone(), p.player_type, p.recovering_injury.is_some())).collect();
    // fieldModel.remove(oldTeam) is a no-op on the new game's empty default team.
    if home {
        game.team_home = team;
    } else {
        game.team_away = team;
    }
    for (player_id, player_type, recovering) in players {
        // remove mercs, stars and raised players, they will be added via command later
        // (the code removes null / MERCENARY / RAISED_FROM_DEAD only)
        if matches!(player_type, PlayerType::Mercenary | PlayerType::RaisedFromDead) {
            game.field_model.player_coordinates.remove(&player_id);
            let team = if home { &mut game.team_home } else { &mut game.team_away };
            team.players.retain(|p| p.id != player_id);
        } else {
            let state = if recovering { PS_MISSING } else { PS_RESERVE };
            game.field_model.set_player_state(&player_id, PlayerState::new(state));
            let player_result = game.game_result.team_result_mut(home).player_result_mut(&player_id);
            if recovering {
                player_result.send_to_box_reason = Some(SendToBoxReason::Mng);
            }
            player_result.serious_injury = None;
            player_result.serious_injury_decay = None;
            player_result.current_spps =
                old_team_result.player_result(&player_id).map_or(0, |r| r.current_spps);
            let team = if home { &mut game.team_home } else { &mut game.team_away };
            let player = team
                .players
                .iter_mut()
                .find(|p| p.id == player_id)
                .ok_or_else(|| format!("startOfGame: player {player_id} vanished"))?;
            remove_all_enhancements(player);
            UtilBox::put_player_into_box(game, &player_id);
        }
    }
    Ok(())
}

/// `player.getEnhancementSources().forEach(player::removeEnhancements)` on a `RosterPlayer`: every
/// source key of the three temporary maps is removed from all three (empty-set sources included),
/// together with the engine caches the Rust loader fills from them.
fn remove_all_enhancements(player: &mut ffb_model::model::player::Player) {
    let mut sources: Vec<String> = Vec::new();
    sources.extend(player.temporary_modifiers_map.iter().map(|(s, _)| s.clone()));
    sources.extend(player.temporary_skills_map.iter().map(|(s, _)| s.clone()));
    sources.extend(player.temporary_properties_map.iter().map(|(s, _)| s.clone()));
    for source in &sources {
        player.temporary_modifiers_map.retain(|(s, _)| s != source);
        player.temporary_skills_map.retain(|(s, _)| s != source);
        player.temporary_properties_map.retain(|(s, _)| s != source);
        player.remove_enhancements(source);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SeriousInjuryKind};
    use ffb_model::model::player::Player;
    use ffb_model::types::{FieldCoordinate, MNG_HOME_X, RSV_HOME_X};

    fn player(id: &str, nr: i32, player_type: PlayerType) -> Player {
        Player { id: id.into(), nr, player_type, ..Default::default() }
    }

    /// Expectations read off `FumbblReplayParser.startOfGame` / `addTeam`.
    #[test]
    fn start_of_game_boxes_reserves_drops_mercenaries_and_keeps_spps() {
        let mut old = Game::new(Team::new(), Team::new(), Rules::Bb2025);
        old.id = 77;
        old.turn_time = 5;
        old.half = 2;
        old.turn_mode = TurnMode::EndGame;
        let mut recovering = player("p2", 2, PlayerType::Regular);
        recovering.recovering_injury = Some(SeriousInjuryKind::SeriouslyHurt);
        let mut enhanced = player("p1", 1, PlayerType::Regular);
        enhanced.temporary_skills_map.push(("Iron Man".into(), vec![]));
        enhanced.temporary_modifiers_map.push(("Dodgy Snack".into(), vec![]));
        old.team_home.players = vec![recovering, player("p3", 3, PlayerType::Mercenary), enhanced];
        old.game_result.home.player_result_mut("p1").current_spps = 11;
        old.game_result.home.player_result_mut("p1").serious_injury = Some(SeriousInjuryKind::SeriouslyHurt);

        let game = start_of_game(old).unwrap();
        assert_eq!((game.id, game.turn_time, game.half), (77, 5, 0));
        assert_eq!((game.turn_mode, game.last_turn_mode), (TurnMode::StartGame, None));
        assert_eq!(game.dialog_id, Some(ffb_model::dialog::dialog_id::DialogId::START_GAME));
        assert!(!game.home_first_offense);
        let ids: Vec<&str> = game.team_home.players.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, ["p2", "p1"], "the mercenary is removed");
        assert_eq!(game.field_model.player_state("p1"), Some(PlayerState::new(PS_RESERVE)));
        assert_eq!(game.field_model.player_state("p2"), Some(PlayerState::new(PS_MISSING)));
        assert_eq!(game.field_model.player_coordinate("p1"), Some(FieldCoordinate::new(RSV_HOME_X, 0)));
        assert_eq!(game.field_model.player_coordinate("p2"), Some(FieldCoordinate::new(MNG_HOME_X, 0)));
        let result = &game.game_result.home;
        assert_eq!(result.penalty_score, -1);
        assert_eq!(result.player_result("p1").unwrap().current_spps, 11);
        assert_eq!(result.player_result("p1").unwrap().serious_injury, None);
        assert_eq!(result.player_result("p2").unwrap().send_to_box_reason, Some(SendToBoxReason::Mng));
        let p1 = game.team_home.players.iter().find(|p| p.id == "p1").unwrap();
        assert!(p1.temporary_skills_map.is_empty() && p1.temporary_modifiers_map.is_empty());
    }
}
