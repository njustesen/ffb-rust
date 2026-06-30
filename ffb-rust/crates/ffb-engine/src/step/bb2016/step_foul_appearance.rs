/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.StepFoulAppearance` (BB2016).
///
/// Resolves the Foul Appearance negatrait check against an attacker.
///
/// The Java step body is entirely delegated to `executeStepHooks(this, state)`.
/// The BB2016 Foul Appearance behaviour (inlined here): roll 2d6 vs. the acting
/// player's Foul Appearance requirement; on failure → END_PLAYER_ACTION + GOTO label.
///
/// Since `executeStepHooks` is not ported, this step inlines the behaviour:
/// - If TurnMode doesn't check negaTraits → NEXT_STEP immediately.
/// - Roll 2d6; if total ≤ 7 → failure path (END_PLAYER_ACTION + GOTO label).
///   On success → NEXT_STEP.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
///
/// TODO: re-roll path (WAITING_FOR_RE_ROLL / TRR) not translated.
/// TODO: actual Foul Appearance roll threshold from player skill not translated.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2016.StepFoulAppearance`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepFoulAppearance {
    /// Java: state.goToLabelOnFailure
    pub goto_label_on_failure: String,
}

impl StepFoulAppearance {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self { goto_label_on_failure }
    }
}

impl Default for StepFoulAppearance {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepFoulAppearance {
    fn id(&self) -> StepId { StepId::FoulAppearance }

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

impl StepFoulAppearance {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: executeStepHooks(this, state) — inlined Foul Appearance behaviour
        // BB2016: roll 2d6; on ≤ 7 (combined), the attacker fails.
        // TODO(foul_appearance_bb2016): actual roll threshold from player skill not translated.
        let roll = rng.d6() + rng.d6();
        if roll <= 7 {
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
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn sometimes_succeeds() {
        let mut found_success = false;
        for seed in 0u64..500 {
            let mut g = make_game();
            let mut s = StepFoulAppearance::new("fail".into());
            let out = s.start(&mut g, &mut GameRng::new(seed));
            if out.action == StepAction::NextStep {
                found_success = true;
                break;
            }
        }
        assert!(found_success, "Should sometimes succeed (2d6 > 7)");
    }

    #[test]
    fn sometimes_fails_and_goes_to_label() {
        let mut found_failure = false;
        for seed in 0u64..500 {
            let mut g = make_game();
            let mut s = StepFoulAppearance::new("fa_fail".into());
            let out = s.start(&mut g, &mut GameRng::new(seed));
            if out.action == StepAction::GotoLabel {
                assert_eq!(out.goto_label.as_deref(), Some("fa_fail"));
                found_failure = true;
                break;
            }
        }
        assert!(found_failure, "Should sometimes fail (2d6 <= 7)");
    }

    #[test]
    fn failure_publishes_end_player_action() {
        for seed in 0u64..500 {
            let mut g = make_game();
            let mut s = StepFoulAppearance::new("fail".into());
            let out = s.start(&mut g, &mut GameRng::new(seed));
            if out.action == StepAction::GotoLabel {
                assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
                return;
            }
        }
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepFoulAppearance::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepFoulAppearance::new("old".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
