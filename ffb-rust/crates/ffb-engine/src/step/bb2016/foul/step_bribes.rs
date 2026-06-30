/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.foul.StepBribes`.
///
/// Step in foul sequence to handle bribes (BB2016).
/// - Asks for bribes dialog if any AVOID_BAN inducements remain.
/// - On bribe choice: roll (2+); success → goto end label; failure → ask again.
/// - On no bribe: asks for Argue-the-Call dialog.
/// - On argue-the-call choice: roll; if coach banned → mark banned.
/// - Publishes FOULER_HAS_BALL, ARGUE_THE_CALL_SUCCESSFUL.
///
/// Init parameter: GOTO_LABEL_ON_END (mandatory).
///
/// TODO(Bribes-inducementSet): InducementSet.AVOID_BAN / UtilServerInducementUse deferred.
/// TODO(Bribes-argueTheCall): GameOptionId.ARGUE_THE_CALL + DiceInterpreter deferred.
/// TODO(Bribes-dialog): UtilServerDialog deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepBribes` (bb2016/foul).
pub struct StepBribes {
    /// Java: `fGotoLabelOnEnd` — mandatory init param.
    goto_label_on_end: String,
    /// Java: `fArgueTheCallChoice`
    argue_the_call_choice: Option<bool>,
    /// Java: `fArgueTheCallSuccessful`
    argue_the_call_successful: Option<bool>,
    /// Java: `fBribesChoice`
    bribes_choice: Option<bool>,
    /// Java: `fBribeSuccessful`
    bribe_successful: Option<bool>,
}

impl StepBribes {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            argue_the_call_choice: None,
            argue_the_call_successful: None,
            bribes_choice: None,
            bribe_successful: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let _ = rng;
        // TODO(Bribes-inducementSet): check for AVOID_BAN inducements.
        if self.bribes_choice.is_none() {
            // No bribes available → skip straight to argue-the-call.
            self.bribes_choice = Some(false);
        }
        // TODO(Bribes-bribeRoll): if bribes_choice == true, roll bribes.
        // TODO(Bribes-argueTheCall): if bribes_choice set and argue_the_call_choice is_none, ask.
        if self.bribes_choice == Some(false) && self.argue_the_call_choice.is_none() {
            // No bribe available — also no argue-the-call by default.
            self.argue_the_call_choice = Some(false);
        }
        if self.bribes_choice.is_some() && self.argue_the_call_choice.is_some() {
            let fouler_has_ball = game.acting_player.player_id
                .as_ref()
                .and_then(|id| game.field_model.player_coordinate(id))
                .zip(game.field_model.ball_coordinate)
                .map(|(pc, bc)| pc == bc)
                .unwrap_or(false);
            let atc_ok = self.argue_the_call_successful.unwrap_or(false);
            return StepOutcome::next()
                .publish(StepParameter::FoulerHasBall(fouler_has_ball))
                .publish(StepParameter::ArgueTheCallSuccessful(atc_ok));
        }
        StepOutcome::cont()
    }
}

impl Default for StepBribes {
    fn default() -> Self { Self::new() }
}

impl Step for StepBribes {
    fn id(&self) -> StepId { StepId::Bribes }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::ArgueTheCall { argue } => {
                self.argue_the_call_choice = Some(*argue);
                self.argue_the_call_successful = None;
            }
            Action::UseBribe { use_bribe } => {
                self.bribes_choice = Some(*use_bribe);
                self.bribe_successful = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(s) => { self.goto_label_on_end = s.clone(); true }
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_bribes() {
        assert_eq!(StepBribes::new().id(), StepId::Bribes);
    }

    #[test]
    fn no_inducements_skips_straight_to_next() {
        let mut game = make_game();
        let mut step = StepBribes::new();
        step.goto_label_on_end = "end".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // No bribes, no argue-the-call → NextStep
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn publishes_fouler_has_ball_and_argue_the_call() {
        let mut game = make_game();
        let mut step = StepBribes::new();
        step.goto_label_on_end = "end".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::FoulerHasBall(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ArgueTheCallSuccessful(_))));
    }

    #[test]
    fn set_parameter_goto_label_on_end() {
        let mut step = StepBribes::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("x".into())));
        assert_eq!(step.goto_label_on_end, "x");
    }
}
