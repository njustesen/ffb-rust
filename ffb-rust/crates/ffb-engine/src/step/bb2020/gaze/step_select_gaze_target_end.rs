/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.gaze.StepSelectGazeTargetEnd` (BB2020).
///
/// Ends the gaze-target selection phase: if the turn is ending push EndPlayerAction; if the
/// selection was cancelled push Select; if a target was selected push Move (bloodlust) or
/// Select + change action to GAZE_MOVE; otherwise push EndMoving.
///
/// TODOs:
///  - TODO(target-selection-state): TargetSelectionState from game.field_model not translated;
///    the full isCanceled() / isSelected() branching is approximated (see comments).
///  - TODO(target-selection-state): changePlayerAction(null, null) on cancel path not translated.
///  - TODO(target-selection-state): setTargetSelectionState(null) on cancel path not translated.
///  - TODO(target-selection-state): setPlayerState (changeSelectedGazeTarget(false)) on Move path not translated.
///  - TODO(target-selection-state): setDefenderId(null) on Move path not translated.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.gaze.StepSelectGazeTargetEnd`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::PlayerAction;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2020::EndPlayerAction;
use crate::step::generator::bb2020::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2020::Select;
use crate::step::generator::bb2020::select::SelectParams;
use crate::step::generator::bb2020::Move;
use crate::step::generator::bb2020::move_::MoveParams;
use crate::step::util_server_steps;

pub struct StepSelectGazeTargetEnd {
    pub end_turn: bool,
    pub bloodlust_action: Option<PlayerAction>,
}

impl StepSelectGazeTargetEnd {
    pub fn new() -> Self {
        Self {
            end_turn: false,
            bloodlust_action: None,
        }
    }
}

impl Default for StepSelectGazeTargetEnd {
    fn default() -> Self { Self::new() }
}

impl Step for StepSelectGazeTargetEnd {
    fn id(&self) -> StepId { StepId::SelectGazeTargetEnd }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::BloodLustAction(v) => { self.bloodlust_action = *v; true }
            _ => false,
        }
    }
}

impl StepSelectGazeTargetEnd {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: TargetSelectionState targetSelectionState = game.getFieldModel().getTargetSelectionState();

        // Case 1: end turn
        // Java: EndPlayerAction.pushSequence(new EndPlayerAction.SequenceParams(true, true, endTurn))
        //       game.setDefenderId(null)
        if self.end_turn {
            game.defender_id = None;
            let params = EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: true,
                end_turn: self.end_turn,
            };
            let seq = EndPlayerAction::build_sequence(&params);
            return StepOutcome::next().push_seq(seq);
        }

        // TODO(target-selection-state): targetSelectionState is not yet exposed on game.field_model.
        // The full Java logic branches on targetSelectionState.isCanceled() / isSelected() / else.
        // Since we cannot read TargetSelectionState here, we approximate with the most-common path:
        // push Select(update_persistence=false) — this matches the isSelected() + non-bloodlust branch
        // (change action to GAZE_MOVE, push Select). The bloodlust and EndMoving paths require
        // TargetSelectionState to be implemented.
        //
        // Java isCanceled() path:
        //   changePlayerAction(null, null, false)
        //   setTargetSelectionState(null)
        //   Select(update_persistence=false)
        //
        // Java isSelected() + bloodlust path:
        //   setPlayerState(target, changeSelectedGazeTarget(false))
        //   setDefenderId(null)
        //   changePlayerAction(actingPlayer, bloodlustAction, false)
        //   Move.pushSequence()
        //
        // Java isSelected() + no bloodlust path:
        //   changePlayerAction(actingPlayer, GAZE_MOVE, false)
        //   Select(update_persistence=false)
        //
        // Java else (neither canceled nor selected, i.e. targetSelectionState != null but status unknown):
        //   push Sequence { EndMoving with END_PLAYER_ACTION=true }

        // Approximation: check bloodlust first (can check without TargetSelectionState).
        let suffering_blood_lust = game.acting_player.suffering_blood_lust;
        if suffering_blood_lust {
            if let Some(action) = self.bloodlust_action {
                // Java: Move.pushSequence(); changePlayerAction(actingPlayer, bloodlustAction, false)
                // TODO(target-selection-state): setPlayerState(target, changeSelectedGazeTarget(false)) not translated.
                // TODO(target-selection-state): setDefenderId(null) not translated.
                let player_id = game.acting_player.player_id.clone();
                if let Some(ref pid) = player_id {
                    util_server_steps::change_player_action(game, pid, action, false);
                }
                let seq = Move::build_sequence(&MoveParams::default());
                return StepOutcome::next().push_seq(seq);
            }
        }

        // Default: non-bloodlust selected path → change action to GAZE_MOVE, push Select(false)
        // Java: changePlayerAction(actingPlayer, PlayerAction.GAZE_MOVE, false)
        //       Select(update_persistence=false)
        let player_id = game.acting_player.player_id.clone();
        if let Some(ref pid) = player_id {
            util_server_steps::change_player_action(game, pid, PlayerAction::GazeMove, false);
        }
        let seq = Select::build_sequence(&SelectParams {
            update_persistence: false,
            is_blitz_move: false,
            block_targets: vec![],
        });
        StepOutcome::next().push_seq(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn end_turn_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepSelectGazeTargetEnd::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        // Should start with RemoveTargetSelectionState (from EndPlayerAction BB2020)
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn end_turn_clears_defender_id() {
        let mut game = make_game();
        game.defender_id = Some("defender".into());
        let mut step = StepSelectGazeTargetEnd::new();
        step.end_turn = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.defender_id.is_none());
    }

    #[test]
    fn default_path_pushes_select_sequence() {
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        let mut step = StepSelectGazeTargetEnd::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        // Select sequence starts with InitSelecting
        assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting);
    }

    #[test]
    fn default_path_changes_action_to_gaze_move() {
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        let mut step = StepSelectGazeTargetEnd::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::GazeMove));
    }

    #[test]
    fn bloodlust_with_action_pushes_move_sequence() {
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.suffering_blood_lust = true;
        let mut step = StepSelectGazeTargetEnd::new();
        step.bloodlust_action = Some(PlayerAction::Move);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        // Move sequence starts with InitMoving
        assert_eq!(out.pushes[0][0].step_id, StepId::InitMoving);
    }

    #[test]
    fn bloodlust_without_action_pushes_select_sequence() {
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.suffering_blood_lust = true;
        let mut step = StepSelectGazeTargetEnd::new();
        step.bloodlust_action = None;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // bloodlust=true but no action → fallthrough to Select path
        assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting);
    }

    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepSelectGazeTargetEnd::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_bloodlust_action() {
        let mut step = StepSelectGazeTargetEnd::new();
        assert!(step.set_parameter(&StepParameter::BloodLustAction(Some(PlayerAction::Move))));
        assert_eq!(step.bloodlust_action, Some(PlayerAction::Move));
    }

    #[test]
    fn set_parameter_bloodlust_action_none() {
        let mut step = StepSelectGazeTargetEnd::new();
        step.bloodlust_action = Some(PlayerAction::Blitz);
        assert!(step.set_parameter(&StepParameter::BloodLustAction(None)));
        assert_eq!(step.bloodlust_action, None);
    }

    #[test]
    fn handle_command_delegates_to_execute_step() {
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        let mut step = StepSelectGazeTargetEnd::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        // Default path → Select
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting);
    }
}
