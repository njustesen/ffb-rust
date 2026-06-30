/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.blitz.StepRemoveTargetSelectionState`.
///
/// Removes the target selection state from the field model.
/// If `retain_model_data` is false, also clears the selection state entirely and
/// marks `has_triggered_effect` if the state was committed.
/// If `retain_model_data` is true, only removes the target-selection flags from the
/// selected player's `PlayerState` — the selection state object itself is kept.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepRemoveTargetSelectionState` (mixed/blitz, BB2020 + BB2025).
#[derive(Debug, Default)]
pub struct StepRemoveTargetSelectionState {
    /// Java: `retainModelData` — init parameter.
    retain_model_data: bool,
}

impl StepRemoveTargetSelectionState {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // Java: TargetSelectionState targetSelectionState = game.getFieldModel().getTargetSelectionState()
        if let Some(tss) = game.field_model.target_selection_state.clone() {
            // Java: String playerId = targetSelectionState.getSelectedPlayerId()
            if let Some(player_id) = tss.selected_player_id.as_deref() {
                // Java: Player<?> player = game.getPlayerById(playerId)
                let player_exists = game.player(player_id).is_some();
                if player_exists {
                    // Java: game.getFieldModel().setPlayerState(player, playerState.removeAllTargetSelections())
                    if let Some(state) = game.field_model.player_state(player_id) {
                        let new_state = state.remove_all_target_selections();
                        game.field_model.set_player_state(player_id, new_state);
                    }
                }
            }

            if self.retain_model_data {
                // Java: keeping empty block — do nothing, leave target selection state intact
                // (before Frenzied Rush fix, we removed player id here — see Java comment)
            } else {
                // Java: markSkillsTrackedOutsideOfActivationAndRemoveEffects(game) — not yet ported
                // Java: if (targetSelectionState.isCommitted()) actingPlayer.setHasTriggeredEffect(true)
                // TODO: ActingPlayer.set_has_triggered_effect not yet ported
                let _ = tss.is_committed();
                // Java: game.getFieldModel().setTargetSelectionState(null)
                game.field_model.target_selection_state = None;
            }
        }

        // Java: getResult().setNextAction(StepAction.NEXT_STEP)
        StepOutcome::next()
    }
}

impl Step for StepRemoveTargetSelectionState {
    fn id(&self) -> StepId { StepId::RemoveTargetSelectionState }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::Retain(v) => { self.retain_model_data = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::model::target_selection_state::TargetSelectionState;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_remove_target_selection_state() {
        assert_eq!(StepRemoveTargetSelectionState::new().id(), StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn no_target_selection_state_returns_next() {
        let mut step = StepRemoveTargetSelectionState::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        assert!(game.field_model.target_selection_state.is_none());
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn clears_target_selection_state_when_retain_false() {
        let mut step = StepRemoveTargetSelectionState::new();
        step.retain_model_data = false;

        let mut game = make_game();
        let mut rng = GameRng::new(0);
        game.field_model.target_selection_state = Some(TargetSelectionState::default());

        step.start(&mut game, &mut rng);

        assert!(game.field_model.target_selection_state.is_none());
    }

    #[test]
    fn retains_target_selection_state_when_retain_true() {
        let mut step = StepRemoveTargetSelectionState::new();
        step.retain_model_data = true;

        let mut game = make_game();
        let mut rng = GameRng::new(0);
        game.field_model.target_selection_state = Some(TargetSelectionState::default());

        step.start(&mut game, &mut rng);

        assert!(game.field_model.target_selection_state.is_some());
    }

    #[test]
    fn set_parameter_retain_stores_value() {
        let mut step = StepRemoveTargetSelectionState::new();
        step.set_parameter(&StepParameter::Retain(true));
        assert!(step.retain_model_data);
    }

    #[test]
    fn committed_state_is_cleared() {
        let mut step = StepRemoveTargetSelectionState::new();
        step.retain_model_data = false;

        let mut game = make_game();
        let mut rng = GameRng::new(0);

        let mut tss = TargetSelectionState::default();
        tss.commit();
        game.field_model.target_selection_state = Some(tss);

        step.start(&mut game, &mut rng);

        assert!(game.field_model.target_selection_state.is_none());
    }
}
