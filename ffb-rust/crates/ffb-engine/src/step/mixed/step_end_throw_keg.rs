/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepEndThrowKeg`.
///
/// Ends the throw-keg sequence by always pushing an `EndPlayerAction` sequence onto the
/// stack (endPlayerAction=true, endTurn from the `END_TURN` parameter).
///
/// Java: `StepEndThrowKeg` — extends `AbstractStep`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepEndThrowKeg` (mixed, BB2020 + BB2025).
pub struct StepEndThrowKeg {
    /// Java: `endTurn`
    end_turn: bool,
}

impl StepEndThrowKeg {
    pub fn new() -> Self {
        Self { end_turn: false }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // Java: endPlayerActionGenerator.pushSequence(
        //           new EndPlayerAction.SequenceParams(gameState, false, true, endTurn))
        // i.e. (feedingAllowed=false, endPlayerAction=true, endTurn), then NEXT_STEP. Java does
        // NOT clear the stack here (unlike StepEndThenIStartedBlastin), so neither do we.
        //
        // This used to merely PUBLISH EndPlayerAction/EndTurn with a comment claiming "the driver
        // owns the sequence stack". Nothing consumes those parameters, and EndThrowKeg is the LAST
        // step of the keg sequence — so the stack emptied with the activation still open and the
        // driver stalled (Continue with no prompt), ending the game at the first keg (dwarf bb2025
        // seed 1 i=88). Same publish-only shape as StepEndThenIStartedBlastin's fix.
        use crate::step::generator::bb2025::EndPlayerAction;
        use crate::step::generator::bb2025::end_player_action::EndPlayerActionParams;
        let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
            feeding_allowed: false,
            end_player_action: true,
            end_turn: self.end_turn,
            check_forgo: false,
            rules: game.rules,
        });
        StepOutcome::next().push_seq(seq)
    }
}

impl Default for StepEndThrowKeg {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndThrowKeg {
    fn id(&self) -> StepId { StepId::EndThrowKeg }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
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

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    /// Java PUSHES the EndPlayerAction sequence here; it does not merely publish parameters.
    /// EndThrowKeg is the LAST step of the keg sequence, so publish-only left the stack empty
    /// with the activation still open and the driver stalled — every dwarf bb2025 game ended at
    /// the first keg (seed 1 i=88, rust_total collapsing from ~9s to 1.4s). Same publish-only
    /// shape as StepEndThenIStartedBlastin's fix; unlike that one, Java does NOT clear the stack.
    #[test]
    fn start_pushes_the_end_player_action_sequence() {
        let mut step = StepEndThrowKeg::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(),
            "must push an EndPlayerAction sequence, not merely publish parameters");
        assert!(!out.clear_stack, "Java's StepEndThrowKeg does not clear the step stack");
    }

    /// `end_turn` is threaded into the PUSHED sequence (EndPlayerAction.SequenceParams' third
    /// argument), not published as a bare parameter. These two tests used to assert the published
    /// form, which is exactly the publish-only behaviour that stalled the driver — they passed
    /// while the mechanic was broken because the keg had never executed.
    fn pushed_end_turn(out: &StepOutcome) -> bool {
        out.pushes.iter().flatten().any(|s| {
            s.params.iter().any(|p| matches!(p, StepParameter::EndTurn(true)))
        })
    }

    #[test]
    fn end_turn_false_by_default() {
        let mut step = StepEndThrowKeg::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert!(!pushed_end_turn(&out), "default end_turn should be false");
    }

    #[test]
    fn set_parameter_end_turn_updates_state() {
        let mut step = StepEndThrowKeg::new();
        let accepted = step.set_parameter(&StepParameter::EndTurn(true));
        assert!(accepted);
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert!(pushed_end_turn(&out),
            "end_turn must reach the pushed EndPlayerAction sequence");
    }

    #[test]
    fn set_parameter_rejects_unknown() {
        let mut step = StepEndThrowKeg::new();
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }
}
