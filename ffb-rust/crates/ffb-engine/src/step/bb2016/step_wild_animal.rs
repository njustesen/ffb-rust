/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepWildAnimal` (BB2016).
///
/// Resolves the Wild Animal negatrait check.
///
/// The Java step body is entirely delegated to `executeStepHooks(this, state)`.
/// In BB2016, Wild Animal behaviour rolls d6: on 1-3 the player fails (END_PLAYER_ACTION).
/// On 4-6 the player passes and can act.
///
/// The `state` carries:
///   - `status` (ActionStatus): SUCCESS | FAILURE | WAITING_FOR_RE_ROLL
///   - `goToLabelOnFailure` (String): label to jump to when Wild Animal fires
///
/// Since `executeStepHooks` is not ported, this step inlines the behaviour:
/// - If TurnMode doesn't check negaTraits → NEXT_STEP immediately.
/// - Otherwise: roll d6; on 1-3 → publish END_PLAYER_ACTION + GOTO failure label.
///   On 4-6 → NEXT_STEP.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
///
/// DEFERRED(wildAnimalReroll): re-roll path via executeStepHooks (WildAnimalHook) not yet ported.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2016.StepWildAnimal`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepWildAnimal {
    /// Java: state.goToLabelOnFailure
    pub goto_label_on_failure: String,
}

impl StepWildAnimal {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self { goto_label_on_failure }
    }
}

impl Default for StepWildAnimal {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepWildAnimal {
    fn id(&self) -> StepId { StepId::WildAnimal }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepWildAnimal {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (!game.getTurnMode().checkNegatraits()) { NEXT_STEP; return; }
        if !game.turn_mode.check_negatraits() {
            return StepOutcome::next();
        }

        // Java: executeStepHooks(this, state) — inlined Wild Animal behaviour
        // BB2016 Wild Animal: roll d6; 1-3 = fail (END_PLAYER_ACTION + GOTO label), 4-6 = pass.
        let roll = rng.d6();
        if roll <= 3 {
            let label = self.goto_label_on_failure.clone();
            StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true))
        } else {
            StepOutcome::next()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn no_negatrait_check_returns_next_step() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn; // KickoffReturn doesn't check negatraits
        let mut step = StepWildAnimal::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn high_roll_returns_next_step() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepWildAnimal::new("fail".into());
        // Find a seed that gives d6 > 3
        let mut found_success = false;
        for seed in 0u64..200 {
            let mut g = make_game();
            g.turn_mode = TurnMode::Regular;
            let mut s = StepWildAnimal::new("fail".into());
            let out = s.start(&mut g, &mut GameRng::new(seed));
            if out.action == StepAction::NextStep {
                found_success = true;
                break;
            }
        }
        assert!(found_success, "Should sometimes succeed (d6 > 3)");
        let _ = (&mut game, &mut step);
    }

    #[test]
    fn low_roll_goes_to_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        // Find a seed that gives d6 <= 3
        let mut found_failure = false;
        for seed in 0u64..200 {
            let mut g = make_game();
            g.turn_mode = TurnMode::Regular;
            let mut s = StepWildAnimal::new("fail_label".into());
            let out = s.start(&mut g, &mut GameRng::new(seed));
            if out.action == StepAction::GotoLabel {
                assert_eq!(out.goto_label.as_deref(), Some("fail_label"));
                assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
                found_failure = true;
                break;
            }
        }
        assert!(found_failure, "Should sometimes fail (d6 <= 3)");
    }

    #[test]
    fn failure_publishes_end_player_action() {
        // Find a seed that causes failure
        for seed in 0u64..200 {
            let mut g = make_game();
            g.turn_mode = TurnMode::Regular;
            let mut s = StepWildAnimal::new("fail".into());
            let out = s.start(&mut g, &mut GameRng::new(seed));
            if out.action == StepAction::GotoLabel {
                assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
                return;
            }
        }
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepWildAnimal::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepWildAnimal::new("old".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
