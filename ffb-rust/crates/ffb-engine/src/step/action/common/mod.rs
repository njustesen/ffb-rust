pub mod step_bone_head;
pub mod step_really_stupid;

pub use step_bone_head::StepBoneHead;
pub use step_really_stupid::StepReallyStupid;

use ffb_model::enums::{PlayerAction, PS_PRONE};
use ffb_model::model::game::Game;

/// Shared helper: cancels the acting player's current action on a negatrait failure.
///
/// Java: `bb2025.BoneHeadBehaviour.cancelPlayerAction` (identical body to
/// `bb2025.ReallyStupidBehaviour.cancelPlayerAction` and `bb2025.WildAnimalBehaviour`'s use of the
/// same pattern) — this helper is only used by the BB2025 skill-hook translations (bb2016/bb2020
/// each inline their own, slightly different, copy).
///
/// NOTE: unlike bb2016/bb2020's version, BB2025's standing-up branch ALSO sets `changeConfused(true)`:
/// `playerState.changeBase(PlayerState.PRONE).changeConfused(true).changeActive(false)` — bb2016/2020
/// only do `changeBase(PRONE).changeActive(false)` (no confused) when standing up.
pub fn cancel_negatrait_player_action(game: &mut Game, player_id: &str) {
    match game.acting_player.player_action {
        Some(PlayerAction::Blitz) | Some(PlayerAction::BlitzMove)
        | Some(PlayerAction::KickEmBlitz) | Some(PlayerAction::StandUpBlitz) => {
            game.turn_data_mut().blitz_used = true;
        }
        Some(PlayerAction::KickTeamMate) | Some(PlayerAction::KickTeamMateMove) => {
            game.turn_data_mut().ktm_used = true;
        }
        Some(PlayerAction::Pass) | Some(PlayerAction::PassMove) => {
            game.turn_data_mut().pass_used = true;
        }
        Some(PlayerAction::ThrowTeamMate) | Some(PlayerAction::ThrowTeamMateMove) => {
            game.turn_data_mut().ttm_used = true;
        }
        Some(PlayerAction::HandOver) | Some(PlayerAction::HandOverMove) => {
            game.turn_data_mut().hand_over_used = true;
        }
        Some(PlayerAction::Foul) | Some(PlayerAction::FoulMove) => {
            // Java: skip if player has allowsAdditionalFoul property (stub = false → always mark used)
            game.turn_data_mut().foul_used = true;
        }
        Some(PlayerAction::SecureTheBall) => {
            game.turn_data_mut().secure_the_ball_used = true;
        }
        Some(PlayerAction::Punt) | Some(PlayerAction::PuntMove) => {
            game.turn_data_mut().punt_used = true;
        }
        _ => {}
    }

    if let Some(state) = game.field_model.player_state(player_id) {
        let new_state = if game.acting_player.standing_up {
            // Java (bb2025): changeBase(PRONE).changeConfused(true).changeActive(false)
            state.change_base(PS_PRONE).change_confused(true).change_active(false)
        } else {
            // Java: changeConfused(true).changeActive(false)
            state.change_confused(true).change_active(false)
        };
        game.field_model.set_player_state(player_id, new_state);
    }

    game.pass_coordinate = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{PlayerState, Rules, PS_STANDING};
    use ffb_model::types::FieldCoordinate;

    fn make_game(standing_up: bool) -> (Game, String) {
        let pid = "p1".to_string();
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.standing_up = standing_up;
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        (game, pid)
    }

    #[test]
    fn standing_up_failure_is_prone_and_confused() {
        // Real bug: Java's bb2025 cancelPlayerAction ALSO sets changeConfused(true) in the
        // standing-up branch (unlike bb2016/bb2020, which only set changeBase(PRONE)). A
        // previous translation dropped `.changeConfused(true)` here.
        let (mut game, pid) = make_game(true);
        cancel_negatrait_player_action(&mut game, &pid);
        let state = game.field_model.player_state(&pid).unwrap();
        assert_eq!(state.base(), PS_PRONE);
        assert!(state.is_confused(), "bb2025 standing-up failure must also be confused");
        assert!(!state.is_active());
    }

    #[test]
    fn not_standing_up_failure_is_confused_but_not_prone() {
        let (mut game, pid) = make_game(false);
        let before_base = game.field_model.player_state(&pid).unwrap().base();
        cancel_negatrait_player_action(&mut game, &pid);
        let state = game.field_model.player_state(&pid).unwrap();
        assert_eq!(state.base(), before_base);
        assert!(state.is_confused());
        assert!(!state.is_active());
    }
}
