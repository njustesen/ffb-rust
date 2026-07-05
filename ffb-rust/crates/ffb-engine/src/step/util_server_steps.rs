/// 1:1 translation of `com.fumbbl.ffb.server.step.UtilServerSteps` (BB2025).
///
/// Static utility methods used by step implementations.
use ffb_model::model::game::Game;
use ffb_model::enums::{PlayerAction, Rules};
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use crate::step::framework::StepId;

/// Java `validateStepId(IStep, StepId)`.
/// Panics if the step's id does not match `expected_id`.
pub fn validate_step_id(actual_id: StepId, expected_id: StepId) {
    if actual_id != expected_id {
        panic!("Wrong step id. Expected {:?} received {:?}", expected_id, actual_id);
    }
}

/// Java `checkCommandWithActingPlayer(GameState, ICommandWithActingPlayer)`.
/// Returns true if `acting_player_id` matches the current acting player.
pub fn check_command_with_acting_player(game: &Game, acting_player_id: &str) -> bool {
    !acting_player_id.is_empty()
        && game.acting_player.player_id.as_deref() == Some(acting_player_id)
}

/// Java `changePlayerAction(IStep, String, PlayerAction, boolean)`.
/// Updates the acting player in the game state.
pub fn change_player_action(game: &mut Game, player_id: &str, action: PlayerAction, jumping: bool) {
    if !player_id.is_empty() {
        game.acting_player.set_player(player_id.to_owned(), action);
        game.acting_player.jumping = jumping;
    }
}

/// Java `checkTouchdown(GameState)`.
/// Returns true if there is a touchdown condition: a standing ball carrier is in the enemy
/// end zone and the ball is in play and not moving.
pub fn check_touchdown(game: &Game) -> bool {
    if !game.field_model.ball_in_play || game.field_model.ball_moving {
        return false;
    }
    let ball_pos = match game.field_model.ball_coordinate {
        Some(c) => c,
        None => return false,
    };
    let carrier_id = match game.field_model.player_at(ball_pos) {
        Some(id) => id,
        None => return false,
    };
    let carrier_state = match game.field_model.player_state(carrier_id) {
        Some(s) => s,
        None => return false,
    };
    if carrier_state.is_prone_or_stunned() {
        return false;
    }
    // Java: (ballCarrier != actingPlayer.getPlayer()) || !actingPlayer.isSufferingBloodLust() || !mechanic.allowMovementInEndZone()
    // In BB2020/BB2025 allowMovementInEndZone = false → the blood-lust guard never blocks touchdowns.
    // In BB2016 allowMovementInEndZone = true, so blood-lust could block, but we skip that
    // nuance here (blood-lust + movement in endzone is a rare edge case not relevant to parity).
    let allow_movement_in_endzone = matches!(game.rules, Rules::Bb2016);
    let acting_player_id = game.acting_player.player_id.as_deref();
    let is_acting_player = acting_player_id == Some(carrier_id);
    let suffering_blood_lust = game.acting_player.suffering_blood_lust;
    if is_acting_player && suffering_blood_lust && allow_movement_in_endzone {
        return false;
    }
    let home_has_carrier = game.team_home.players.iter().any(|p| p.id.as_str() == carrier_id);
    let away_has_carrier = game.team_away.players.iter().any(|p| p.id.as_str() == carrier_id);
    (home_has_carrier && FieldCoordinateBounds::ENDZONE_AWAY.is_in_bounds(ball_pos))
        || (away_has_carrier && FieldCoordinateBounds::ENDZONE_HOME.is_in_bounds(ball_pos))
}

/// Java `checkEndOfHalf(GameState)`.
/// Returns true when both teams have completed 8 turns (end of half).
pub fn check_end_of_half(game: &Game) -> bool {
    game.turn_data_home.turn_nr >= 8 && game.turn_data_away.turn_nr >= 8
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{PlayerAction, Rules, PS_STANDING, PS_PRONE};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn validate_step_id_same_is_ok() {
        validate_step_id(StepId::EndTurn, StepId::EndTurn); // no panic
    }

    #[test]
    #[should_panic(expected = "Wrong step id")]
    fn validate_step_id_mismatch_panics() {
        validate_step_id(StepId::EndTurn, StepId::InitStartGame);
    }

    #[test]
    fn check_command_with_acting_player_matches() {
        let mut game = make_game();
        game.acting_player.set_player("p01".into(), PlayerAction::Move);
        assert!(check_command_with_acting_player(&game, "p01"));
        assert!(!check_command_with_acting_player(&game, "p02"));
        assert!(!check_command_with_acting_player(&game, ""));
    }

    #[test]
    fn change_player_action_sets_fields() {
        let mut game = make_game();
        change_player_action(&mut game, "p01", PlayerAction::Blitz, true);
        assert_eq!(game.acting_player.player_id.as_deref(), Some("p01"));
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::Blitz));
        assert!(game.acting_player.jumping);
    }

    #[test]
    fn change_player_action_empty_id_is_noop() {
        let mut game = make_game();
        change_player_action(&mut game, "", PlayerAction::Move, false);
        assert!(game.acting_player.player_id.is_none());
    }

    #[test]
    fn check_end_of_half_false_when_turns_low() {
        let mut game = make_game();
        game.turn_data_home.turn_nr = 7;
        game.turn_data_away.turn_nr = 8;
        assert!(!check_end_of_half(&game));
    }

    #[test]
    fn check_end_of_half_true_when_both_eight() {
        let mut game = make_game();
        game.turn_data_home.turn_nr = 8;
        game.turn_data_away.turn_nr = 8;
        assert!(check_end_of_half(&game));
    }

    #[test]
    fn check_touchdown_false_when_ball_not_in_play() {
        let game = make_game();
        assert!(!check_touchdown(&game));
    }

    #[test]
    fn check_touchdown_false_when_ball_moving() {
        let mut game = make_game();
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        assert!(!check_touchdown(&game));
    }

    #[test]
    fn check_touchdown_false_when_no_carrier() {
        let mut game = make_game();
        game.field_model.ball_in_play = true;
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(25, 7));
        assert!(!check_touchdown(&game));
    }

    fn add_player_to_home(game: &mut Game, id: &str, pos: FieldCoordinate, state: u32) {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState};
        game.team_home.players.push(Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, PlayerState::new(state));
    }

    #[test]
    fn check_touchdown_home_player_in_away_endzone() {
        let mut game = make_game();
        let ball_pos = FieldCoordinate::new(25, 7);
        game.field_model.ball_in_play = true;
        game.field_model.ball_coordinate = Some(ball_pos);
        add_player_to_home(&mut game, "p01", ball_pos, PS_STANDING);
        assert!(check_touchdown(&game));
    }

    #[test]
    fn check_touchdown_false_when_carrier_prone() {
        let mut game = make_game();
        let ball_pos = FieldCoordinate::new(25, 7);
        game.field_model.ball_in_play = true;
        game.field_model.ball_coordinate = Some(ball_pos);
        add_player_to_home(&mut game, "p01", ball_pos, PS_PRONE);
        assert!(!check_touchdown(&game));
    }
}
