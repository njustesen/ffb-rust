/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.gaze.StepSelectGazeTargetEnd` (BB2020).
///
/// Ends the gaze-target selection phase: if the turn is ending push EndPlayerAction; if there is
/// no TargetSelectionState, nothing is pushed; if the selection was cancelled push Select; if a
/// target was selected push Move (bloodlust) or Select + change action to GAZE_MOVE; otherwise
/// (TargetSelectionState present but neither canceled nor selected) push EndMoving.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.gaze.StepSelectGazeTargetEnd`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::PlayerAction;
use ffb_model::enums::PlayerState;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{SequenceStep, StepId, StepParameter};
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

        // Java: TargetSelectionState targetSelectionState = game.getFieldModel().getTargetSelectionState();
        //   ... } else if (targetSelectionState != null) { ... }
        // When there is no target selection state at all, Java's else-if chain is skipped
        // entirely — no sequence is pushed, only NEXT_STEP.
        if game.field_model.target_selection_state.is_none() {
            return StepOutcome::next();
        }

        let is_canceled = game.field_model.target_selection_state.as_ref()
            .map(|ts| ts.is_canceled())
            .unwrap_or(false);
        let is_selected = game.field_model.target_selection_state.as_ref()
            .map(|ts| ts.is_selected())
            .unwrap_or(false);
        let target_player_id = game.field_model.target_selection_state.as_ref()
            .and_then(|ts| ts.get_selected_player_id().cloned());

        let player_id = game.acting_player.player_id.clone();

        // Java isCanceled() path:
        //   changePlayerAction(null, null, false); setTargetSelectionState(null); Select(false)
        if is_canceled {
            game.field_model.target_selection_state = None;
            if let Some(ref pid) = player_id {
                util_server_steps::change_player_action(game, pid, PlayerAction::Move, false);
            }
            let seq = Select::build_sequence(&SelectParams {
                update_persistence: false,
                is_blitz_move: false,
                block_targets: vec![],
            });
            return StepOutcome::next().push_seq(seq);
        }

        let suffering_blood_lust = game.acting_player.suffering_blood_lust;

        // Java isSelected() + bloodlust path:
        //   if (actingPlayer.isSufferingBloodLust() && bloodlustAction != null) {
        //     setPlayerState(target, changeSelectedGazeTarget(false)); setDefenderId(null);
        //     changePlayerAction(actingPlayer, bloodlustAction, false); Move.pushSequence()
        //   }
        if is_selected && suffering_blood_lust && self.bloodlust_action.is_some() {
            if let Some(ref tid) = target_player_id {
                if let Some(state) = game.field_model.player_state(tid) {
                    let new_state: PlayerState = state.change_selected_gaze_target(false);
                    game.field_model.set_player_state(tid, new_state);
                }
            }
            game.defender_id = None;
            if let Some(action) = self.bloodlust_action {
                if let Some(ref pid) = player_id {
                    util_server_steps::change_player_action(game, pid, action, false);
                }
            }
            let seq = Move::build_sequence(&MoveParams::default());
            return StepOutcome::next().push_seq(seq);
        }

        // Java isSelected() + no bloodlust path:
        //   changePlayerAction(actingPlayer, PlayerAction.GAZE_MOVE, false); Select(false)
        if is_selected {
            if let Some(ref pid) = player_id {
                util_server_steps::change_player_action(game, pid, PlayerAction::GazeMove, false);
            }
            let seq = Select::build_sequence(&SelectParams {
                update_persistence: false,
                is_blitz_move: false,
                block_targets: vec![],
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java else (targetSelectionState present but neither canceled nor selected —
        //   e.g. STARTED/SKIPPED/FAILED status):
        //   Sequence sequence = new Sequence(getGameState());
        //   sequence.add(StepId.END_MOVING, StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true));
        //   getGameState().getStepStack().push(sequence.getSequence());
        let seq = vec![SequenceStep::with_params(
            StepId::EndMoving,
            vec![StepParameter::EndPlayerAction(true)],
        )];
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
    fn no_target_selection_state_pushes_nothing() {
        // Java: `else if (targetSelectionState != null)` is skipped entirely when
        // there is no TargetSelectionState — no sequence is pushed, just NEXT_STEP.
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        assert!(game.field_model.target_selection_state.is_none());
        let mut step = StepSelectGazeTargetEnd::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.is_empty(), "no target selection state means no sequence is pushed");
    }

    #[test]
    fn no_target_selection_state_does_not_change_action() {
        // Java: without a TargetSelectionState, changePlayerAction is never called.
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        let mut step = StepSelectGazeTargetEnd::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::Move));
    }

    #[test]
    fn target_selection_state_not_canceled_or_selected_pushes_end_moving() {
        // Java else branch: TargetSelectionState present but neither canceled() nor
        // selected() (e.g. still STARTED) → push a bare Sequence with
        // StepId.END_MOVING and END_PLAYER_ACTION=true — NOT the GAZE_MOVE/Select fallback.
        use ffb_model::model::target_selection_state::TargetSelectionState;
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.field_model.target_selection_state = Some(TargetSelectionState::default());
        let mut step = StepSelectGazeTargetEnd::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0].len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::EndMoving);
        assert!(out.pushes[0][0].params.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
        // Player action must NOT have been changed to GazeMove in this branch.
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::Move));
    }

    #[test]
    fn bloodlust_with_action_pushes_move_sequence() {
        use ffb_model::model::target_selection_state::TargetSelectionState;
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.suffering_blood_lust = true;
        // Java: isSelected() branch requires TargetSelectionState with SELECTED status
        let mut ts = TargetSelectionState::new("opponent");
        ts.select();
        game.field_model.target_selection_state = Some(ts);
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
        use ffb_model::model::target_selection_state::TargetSelectionState;
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.suffering_blood_lust = true;
        let mut ts = TargetSelectionState::new("opponent3");
        ts.select();
        game.field_model.target_selection_state = Some(ts);
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
        use ffb_model::model::target_selection_state::TargetSelectionState;
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        let mut ts = TargetSelectionState::new("opp");
        ts.select();
        game.field_model.target_selection_state = Some(ts);
        let mut step = StepSelectGazeTargetEnd::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        // isSelected(), no bloodlust → Select
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting);
    }

    #[test]
    fn canceled_selection_state_clears_state_and_pushes_select() {
        use ffb_model::model::target_selection_state::TargetSelectionState;
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        let mut ts = TargetSelectionState::default();
        ts.cancel();
        game.field_model.target_selection_state = Some(ts);
        let mut step = StepSelectGazeTargetEnd::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // Cancel clears target_selection_state
        assert!(game.field_model.target_selection_state.is_none());
        // Pushes Select sequence
        assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting);
    }

    #[test]
    fn selected_no_bloodlust_changes_action_to_gaze_move_and_pushes_select() {
        use ffb_model::model::target_selection_state::TargetSelectionState;
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.suffering_blood_lust = false;
        let mut ts = TargetSelectionState::new("opp");
        ts.select();
        game.field_model.target_selection_state = Some(ts);
        let mut step = StepSelectGazeTargetEnd::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::GazeMove));
        assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting);
    }

    #[test]
    fn selected_bloodlust_clears_defender_and_pushes_move() {
        use ffb_model::model::target_selection_state::TargetSelectionState;
        let mut game = make_game();
        game.acting_player.player_id = Some("actor".into());
        game.acting_player.suffering_blood_lust = true;
        game.defender_id = Some("old_defender".into());
        let mut ts = TargetSelectionState::new("opp2");
        ts.select();
        game.field_model.target_selection_state = Some(ts);
        let mut step = StepSelectGazeTargetEnd::new();
        step.bloodlust_action = Some(PlayerAction::Move);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // defender_id cleared
        assert!(game.defender_id.is_none());
        // Pushes Move sequence (InitMoving)
        assert_eq!(out.pushes[0][0].step_id, StepId::InitMoving);
    }
}
