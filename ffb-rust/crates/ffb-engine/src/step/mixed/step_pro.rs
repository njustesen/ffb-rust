/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepPro`.
///
/// Handles the Pro skill re-roll.  Given a `PLAYER_ID` init parameter, the step:
///   1. If a re-roll source is set, attempts to consume it (unsets `usedPro` on the player).
///   2. Rolls the Pro re-roll via `UtilServerReRoll.useReRoll(this, ReRollSources.PRO, player)`.
///   3. If unsuccessful and OLD_PRO wasn't already re-rolled, asks for a team/skill re-roll.
///   4. Publishes `SUCCESSFUL_PRO(bool)` and advances.
///
/// `UtilServerReRoll` is not yet fully ported; the roll and dialog logic is stubbed.
///
/// Java: `StepPro extends AbstractStepWithReRoll` (mixed, BB2020 + BB2025).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;

/// Java: `StepPro` (mixed, BB2020 + BB2025).
pub struct StepPro {
    /// Java: `playerId` (init param PLAYER_ID)
    pub player_id: Option<String>,
    /// AbstractStepWithReRoll state
    pub re_roll: ReRollState,
}

impl StepPro {
    pub fn new() -> Self {
        Self { player_id: None, re_roll: ReRollState::new() }
    }

    fn execute_step(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: full logic requires UtilServerReRoll.useReRoll and DiceRoller.rollSkill.
        // TODO(UtilServerReRoll port):
        //   1. If re_roll.re_roll_source is Some → UtilServerReRoll.useReRoll(…)
        //   2. Roll PRO skill (D6 >= 3)
        //   3. If failed and not OLD_PRO → ask for re-roll dialog → StepOutcome::cont()
        //   4. publish StepParameter::SuccessfulPro(result)
        StepOutcome::next()
            .publish(StepParameter::PlayerId(self.player_id.clone().unwrap_or_default()))
    }
}

impl Default for StepPro {
    fn default() -> Self { Self::new() }
}

impl Step for StepPro {
    fn id(&self) -> StepId { StepId::Pro }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: super.handleCommand handles re-roll commands → sets re_roll_source → EXECUTE_STEP
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerId(v) => { self.player_id = Some(v.clone()); true }
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

    #[test]
    fn id_is_pro() {
        assert_eq!(StepPro::new().id(), StepId::Pro);
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepPro::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_player_id() {
        let mut step = StepPro::new();
        let accepted = step.set_parameter(&StepParameter::PlayerId("pid".into()));
        assert!(accepted);
        assert_eq!(step.player_id, Some("pid".into()));
    }

    #[test]
    fn set_parameter_rejects_unknown() {
        let mut step = StepPro::new();
        let rejected = !step.set_parameter(&StepParameter::EndTurn(true));
        assert!(rejected);
    }
}
