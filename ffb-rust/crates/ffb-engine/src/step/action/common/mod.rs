pub mod step_bone_head;
pub mod step_really_stupid;

pub use step_bone_head::StepBoneHead;
pub use step_really_stupid::StepReallyStupid;

use ffb_model::enums::{PlayerAction, PS_PRONE};
use ffb_model::model::game::Game;

/// Shared helper: cancels the acting player's current action on a negatrait failure.
/// Java: BoneHeadBehaviour.cancelPlayerAction (identical body to ReallyStupidBehaviour.cancelPlayerAction)
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
            // Java: changeBase(PRONE).changeActive(false)
            state.change_base(PS_PRONE).change_active(false)
        } else {
            // Java: changeConfused(true).changeActive(false)
            state.change_confused(true).change_active(false)
        };
        game.field_model.set_player_state(player_id, new_state);
    }

    game.pass_coordinate = None;
}
