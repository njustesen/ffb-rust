use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
#[cfg(test)]
use crate::step::framework::StepAction;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepBloodLust.
///
/// Handles the Vampire blood-lust mechanic.  The Java source delegates the
/// entire execution to `getGameState().executeStepHooks(this, state)`, which
/// dispatches to registered hook handlers (not yet translated).  The only
/// client command handled directly is `CLIENT_BLOODLUST_ACTION`, which switches
/// the player action to an "alternate" move-type action so the vampire can feed.
///
/// Java state fields: goToLabelOnFailure, bloodlustAction, status (ActionStatus).
pub struct StepBloodLust {
    /// Java: state.goToLabelOnFailure (init param GOTO_LABEL_ON_FAILURE)
    pub goto_label_on_failure: String,
    /// Java: state.bloodlustAction — alternate action chosen by the client
    pub bloodlust_action: Option<PlayerAction>,
    /// Java: state.status (ActionStatus) — set by hooks, not yet ported
    pub status: Option<String>,
    // AbstractStepWithReRoll stubs (unused until hooks are ported)
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepBloodLust {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self {
            goto_label_on_failure: goto_label_on_failure.into(),
            bloodlust_action: None,
            status: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepBloodLust {
    fn default() -> Self { Self::new("") }
}

impl Step for StepBloodLust {
    fn id(&self) -> StepId { StepId::BloodLust }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_BLOODLUST_ACTION → if (change) state.bloodlustAction = getAlternateAction(currentAction)
        //       → commandStatus = EXECUTE_STEP → executeStep()
        //
        // The Rust Action enum does not yet have a BloodlustAction variant (Java
        // ClientCommandBloodlustAction).  When that variant is added, wire it here:
        //   if let Action::BloodlustAction { change } = action { … }
        // For now any Acknowledge acts as "no change" and falls through.
        let _ = action;
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => {
                self.goto_label_on_failure = v.clone();
                true
            }
            StepParameter::BloodLustAction(v) => {
                self.bloodlust_action = *v;
                true
            }
            _ => false,
        }
    }
}

impl StepBloodLust {
    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: getGameState().executeStepHooks(this, state)
        // The Java body is entirely delegated to step hooks (ActionStatus state machine).
        // Hook handlers drive: blood-lust roll, offer bite, handle feeding, goto failure label.
        // TODO(StepHooks port): executeStepHooks(this, state) — full blood-lust state machine.
        //
        // Until hooks are translated, fall through to NEXT_STEP (same as "hooks return false").
        StepOutcome::next()
    }

    /// Java: private PlayerAction getAlternateAction(PlayerAction currentAction)
    /// Converts an action to its move-phase equivalent for blood-lust feeding.
    fn get_alternate_action(current: PlayerAction) -> PlayerAction {
        match current {
            PlayerAction::Pass => PlayerAction::PassMove,
            PlayerAction::HandOver => PlayerAction::HandOverMove,
            PlayerAction::Foul => PlayerAction::FoulMove,
            PlayerAction::StandUpBlitz => PlayerAction::BlitzSelect,
            PlayerAction::ThrowTeamMate => PlayerAction::ThrowTeamMateMove,
            PlayerAction::KickTeamMate => PlayerAction::KickTeamMateMove,
            _ => PlayerAction::Move,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{PlayerAction, Rules};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next() {
        let mut game = make_game();
        let mut step = StepBloodLust::new("fail_label");
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn goto_label_on_failure_parameter_accepted() {
        let mut step = StepBloodLust::default();
        step.set_parameter(&StepParameter::GotoLabelOnFailure("my_label".to_string()));
        assert_eq!(step.goto_label_on_failure, "my_label");
    }

    #[test]
    fn blood_lust_action_parameter_accepted() {
        let mut step = StepBloodLust::default();
        step.set_parameter(&StepParameter::BloodLustAction(Some(PlayerAction::Move)));
        assert_eq!(step.bloodlust_action, Some(PlayerAction::Move));
    }

    #[test]
    fn get_alternate_action_pass_becomes_pass_move() {
        // Java: PASS → PASS_MOVE
        assert_eq!(StepBloodLust::get_alternate_action(PlayerAction::Pass), PlayerAction::PassMove);
    }

    #[test]
    fn get_alternate_action_foul_becomes_foul_move() {
        // Java: FOUL → FOUL_MOVE
        assert_eq!(StepBloodLust::get_alternate_action(PlayerAction::Foul), PlayerAction::FoulMove);
    }

    #[test]
    fn get_alternate_action_default_becomes_move() {
        // Java: default → MOVE
        assert_eq!(StepBloodLust::get_alternate_action(PlayerAction::Block), PlayerAction::Move);
    }
}
