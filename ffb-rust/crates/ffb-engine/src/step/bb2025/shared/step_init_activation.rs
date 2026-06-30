use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepInitActivation.
///
/// Recovers tackle zones and clears eye gouge on the acting player's state at activation start.
/// If a TargetSelectionState exists on the field model, records the old player state on it.
pub struct StepInitActivation;

impl StepInitActivation {
    pub fn new() -> Self { Self }
}

impl Default for StepInitActivation {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitActivation {
    fn id(&self) -> StepId { StepId::InitActivation }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepInitActivation {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        if let Some(pid) = game.acting_player.player_id.clone() {
            if let Some(player_state) = game.field_model.player_state(&pid) {
                // Java: if (targetSelectionState != null) targetSelectionState.setOldActingPlayerState(playerState)
                if let Some(tss) = game.field_model.target_selection_state.as_mut() {
                    tss.set_old_acting_player_state(Some(player_state));
                }
                // Java: game.getFieldModel().setPlayerState(player, playerState.recoverTacklezones().clearEyeGouge())
                let new_state = player_state.recover_tacklezones().clear_eye_gouge();
                game.field_model.set_player_state(&pid, new_state);
            }
        }
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PlayerState};
    use ffb_model::model::target_selection_state::TargetSelectionState;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next() {
        let mut game = make_game();
        let mut step = StepInitActivation::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_acting_player_still_returns_next() {
        let mut game = make_game();
        game.acting_player.player_id = None;
        let mut step = StepInitActivation::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepInitActivation::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    #[test]
    fn clears_confused_on_acting_player() {
        let mut game = make_game();
        let pid = "player1".to_string();
        game.acting_player.player_id = Some(pid.clone());
        // PlayerState(1) = PS_STANDING; BIT_CONFUSED=0x200, BIT_EYE_GOUGED=0x20000
        let confused = PlayerState(0x00001 | 0x00200 | 0x20000); // standing + confused + eye_gouged
        game.field_model.set_player_state(&pid, confused);

        let mut step = StepInitActivation::new();
        step.start(&mut game, &mut GameRng::new(0));

        let result = game.field_model.player_state(&pid).unwrap();
        assert!(!result.is_confused(), "confused should be cleared");
        assert!(!result.is_eye_gouged(), "eye gouge should be cleared");
    }

    #[test]
    fn records_old_state_in_target_selection_state() {
        let mut game = make_game();
        let pid = "player1".to_string();
        game.acting_player.player_id = Some(pid.clone());
        let ps = PlayerState(0x00001 | 0x00800); // standing + hypnotized (BIT_HYPNOTIZED=0x800)
        game.field_model.set_player_state(&pid, ps);
        game.field_model.target_selection_state = Some(TargetSelectionState::default());

        let mut step = StepInitActivation::new();
        step.start(&mut game, &mut GameRng::new(0));

        let tss = game.field_model.target_selection_state.as_ref().unwrap();
        assert_eq!(tss.get_old_acting_player_state(), Some(ps));
    }
}
