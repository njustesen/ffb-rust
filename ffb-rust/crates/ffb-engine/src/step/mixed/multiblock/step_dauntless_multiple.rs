/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.multiblock.StepDauntlessMultiple`.
///
/// Rolls Dauntless for all current block targets and drives re-roll dialogs.
/// The actual roll logic lives in `executeStepHooks` on the Java side (not yet ported).
/// Client command `CLIENT_USE_RE_ROLL_FOR_TARGET(DAUNTLESS)` triggers a re-roll for a specific
/// target; `CLIENT_PLAYER_CHOICE(LORD_OF_CHAOS)` chooses the single-use team re-roll player.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::mixed::multiblock::abstract_step_multiple::{AbstractStepMultiple, SingleReRollUseState};

/// Java: `StepDauntlessMultiple` (mixed/multiblock, BB2020 + BB2025).
///
/// State (Java `StepStateMultipleRolls`):
///   - `block_targets`: player IDs that are the current block targets
///   - `re_roll_target`: target ID for which a re-roll was requested
///   - `base`: shared `SingleReRollUseState` for LORD_OF_CHAOS handling
pub struct StepDauntlessMultiple {
    /// Java: `state.blockTargets` — active block target player IDs
    pub block_targets: Vec<String>,
    /// Java: `state.reRollTarget` — player ID being re-rolled right now
    pub re_roll_target: Option<String>,
    /// Java: base `AbstractStepMultiple` / `SingleReRollUseState`
    base: AbstractStepMultiple,
}

impl StepDauntlessMultiple {
    pub fn new() -> Self {
        Self {
            block_targets: Vec::new(),
            re_roll_target: None,
            base: AbstractStepMultiple::new(),
        }
    }

    /// Java: `state()` — exposes the re-roll use state to the abstract base
    fn state(&mut self) -> &mut SingleReRollUseState {
        &mut self.base.state
    }

    fn execute_step(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: getGameState().executeStepHooks(this, state)
        // The roll and dialog logic live entirely in step hooks (not yet ported).
        // TODO(StepHooks port): executeStepHooks — Dauntless roll, re-roll dialogs,
        //   publishParameter(PLAYER_ID_DAUNTLESS_SUCCESS) per target that succeeds.
        StepOutcome::next()
    }
}

impl Default for StepDauntlessMultiple {
    fn default() -> Self { Self::new() }
}

impl Step for StepDauntlessMultiple {
    fn id(&self) -> StepId { StepId::DauntlessMultiple }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java CLIENT_USE_RE_ROLL_FOR_TARGET (DAUNTLESS):
        //   if reRollSourceSuccessfully(command.getReRollSource()) → EXECUTE_STEP
        //   state.reRollTarget = command.getTargetId()
        //
        // Java CLIENT_PLAYER_CHOICE (LORD_OF_CHAOS) → handled by abstract base
        match action {
            Action::UseReRollForTarget { re_rolled_action, re_roll_source, target_id }
                if re_rolled_action.as_deref() == Some("DAUNTLESS") =>
            {
                self.re_roll_target = target_id.clone();
                let lords: Vec<String> = vec![]; // TODO: gather from game when property system is ported
                let proceed = crate::step::mixed::multiblock::abstract_step_multiple::re_roll_source_successfully(
                    &mut self.base.state,
                    re_roll_source.as_deref().unwrap_or(""),
                    &lords,
                );
                if proceed {
                    return self.execute_step(game, rng);
                }
                StepOutcome::cont()
            }
            Action::LordOfChaosChoice { player_id } => {
                self.base.apply_lord_of_chaos_command(game, player_id.as_deref());
                self.execute_step(game, rng)
            }
            _ => StepOutcome::next(),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::BlockTargets(ids) => {
                self.block_targets.extend(ids.iter().cloned());
                true
            }
            StepParameter::PlayerIdToRemove(id) => {
                self.block_targets.retain(|t| t != id);
                true
            }
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

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_dauntless_multiple() {
        assert_eq!(StepDauntlessMultiple::new().id(), StepId::DauntlessMultiple);
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepDauntlessMultiple::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn block_targets_set_via_parameter() {
        let mut step = StepDauntlessMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into(), "p2".into()]));
        assert_eq!(step.block_targets, vec!["p1", "p2"]);
    }

    #[test]
    fn player_id_to_remove_shrinks_targets() {
        let mut step = StepDauntlessMultiple::new();
        step.set_parameter(&StepParameter::BlockTargets(vec!["p1".into(), "p2".into()]));
        step.set_parameter(&StepParameter::PlayerIdToRemove("p1".into()));
        assert_eq!(step.block_targets, vec!["p2"]);
    }

    #[test]
    fn handle_command_acknowledge_returns_next() {
        let mut step = StepDauntlessMultiple::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }
}
