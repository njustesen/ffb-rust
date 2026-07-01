/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepUnchannelledFury`.
///
/// Handles the Unchannelled Fury skill in the block sequence (BB2020+).
/// Needs `GOTO_LABEL_ON_FAILURE` initialisation parameter.
///
/// Java state fields:
///   `goToLabelOnFailure` — label to jump to when the roll fails.
///   `status`            — skill-choice result (YES/NO) from CLIENT_USE_SKILL dialog.
///
/// Java execution delegates to `executeStepHooks(this, state)` — not yet ported.
/// When hooks land the actual D6 roll + dialog will be driven from here.
///
/// Java: `StepUnchannelledFury extends AbstractStepWithReRoll` (mixed, BB2020 + BB2025).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Mirrors Java `ActionStatus.SKILL_CHOICE_YES/NO`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillChoiceStatus {
    Yes,
    No,
}

/// Java: `StepUnchannelledFury.StepState`
#[derive(Debug, Default)]
pub struct UnchannelledFuryState {
    /// Java: `goToLabelOnFailure`
    pub goto_label_on_failure: String,
    /// Java: `status` (ActionStatus.SKILL_CHOICE_YES / SKILL_CHOICE_NO)
    pub status: Option<SkillChoiceStatus>,
}

/// Java: `StepUnchannelledFury` (mixed, BB2020 + BB2025).
pub struct StepUnchannelledFury {
    pub state: UnchannelledFuryState,
}

impl StepUnchannelledFury {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self {
            state: UnchannelledFuryState {
                goto_label_on_failure: goto_label_on_failure.into(),
                status: None,
            },
        }
    }

    fn execute_step(&mut self, _game: &mut Game) -> StepOutcome {
        // Java: getGameState().executeStepHooks(this, state)
        // DEFERRED(StepHooks port): roll D6 for Unchannelled Fury; on failure:
        //   publish END_PLAYER_ACTION=true and goto(goto_label_on_failure).
        //   If status==YES (using "canPerformTwoBlocksAfterFailedFury" skill), skip the goto.
        StepOutcome::next()
    }
}

impl Default for StepUnchannelledFury {
    fn default() -> Self { Self::new("") }
}

impl Step for StepUnchannelledFury {
    fn id(&self) -> StepId { StepId::UnchannelledFury }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL → check canPerformTwoBlocksAfterFailedFury
        if let Action::UseSkill { use_skill, .. } = action {
            self.state.status = Some(if *use_skill { SkillChoiceStatus::Yes } else { SkillChoiceStatus::No });
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.state.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;
    use ffb_mechanics::skills::SkillId;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_unchannelled_fury() {
        assert_eq!(StepUnchannelledFury::new("fail").id(), StepId::UnchannelledFury);
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepUnchannelledFury::new("fail");
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepUnchannelledFury::new("old");
        let accepted = step.set_parameter(&StepParameter::GotoLabelOnFailure("new_label".into()));
        assert!(accepted);
        assert_eq!(step.state.goto_label_on_failure, "new_label");
    }

    #[test]
    fn handle_command_use_skill_sets_status_yes() {
        let mut step = StepUnchannelledFury::new("fail");
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let action = Action::UseSkill { skill_id: SkillId::Block, use_skill: true };
        step.handle_command(&action, &mut game, &mut rng);
        assert_eq!(step.state.status, Some(SkillChoiceStatus::Yes));
    }

    #[test]
    fn handle_command_use_skill_sets_status_no() {
        let mut step = StepUnchannelledFury::new("fail");
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let action = Action::UseSkill { skill_id: SkillId::Block, use_skill: false };
        step.handle_command(&action, &mut game, &mut rng);
        assert_eq!(step.state.status, Some(SkillChoiceStatus::No));
    }
}
