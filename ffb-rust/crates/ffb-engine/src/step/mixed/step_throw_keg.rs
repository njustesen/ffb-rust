/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepThrowKeg`.
///
/// Handles the BeerBarrelBash "throw keg" attack skill.
///
/// Java logic (executeStep):
///   1. If acting player has an unused `canThrowKeg` skill (or we're re-rolling THROW_KEG):
///   2. On re-roll path: consume re-roll source; if unavailable → `fail()`.
///   3. Otherwise: mark skill used.
///   4. Roll D6; success = roll >= 3.
///   5. Report `ReportThrownKeg`.
///   6. On success: animate + call `hitPlayer(target, false)`.
///   7. On failure (not re-rolling): ask for re-roll; if available → CONTINUE.
///      Otherwise → `fail()`.
///
/// `fail()`: if roll == 1 → animate fumbled keg + `hitPlayer(thrower, true)`.
///
/// `hitPlayer(p, endTurn)`: run injury pipeline (InjuryTypeKegHit) → publish DROP_PLAYER_CONTEXT.
///
/// Skill property / injury / re-roll infrastructure not yet fully ported — stubbed.
///
/// Java: `StepThrowKeg extends AbstractStepWithReRoll` (mixed, BB2020 + BB2025).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;

/// Java: `StepThrowKeg` (mixed, BB2020 + BB2025).
pub struct StepThrowKeg {
    /// Java: `playerId` (init param TARGET_PLAYER_ID) — the target player.
    pub player_id: Option<String>,
    /// Java: `roll` — D6 result stored for fail check (roll==1 → fumbled).
    pub roll: i32,
    /// AbstractStepWithReRoll state.
    pub re_roll: ReRollState,
}

impl StepThrowKeg {
    pub fn new() -> Self {
        Self { player_id: None, roll: 0, re_roll: ReRollState::new() }
    }

    fn execute_step(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: full logic requires UtilCards.getUnusedSkillWithProperty, DiceRoller, injury pipeline.
        // TODO(skills/injury port):
        //   1. Check actingPlayer.hasUnusedSkillWithProperty(canThrowKeg)
        //   2. Roll D6, report ReportThrownKeg
        //   3. On success: animate + hitPlayer(target, false)
        //   4. On failure: re-roll check or fail()
        //   5. fail(): if roll==1 → hitPlayer(thrower, true)
        StepOutcome::next()
    }
}

impl Default for StepThrowKeg {
    fn default() -> Self { Self::new() }
}

impl Step for StepThrowKeg {
    fn id(&self) -> StepId { StepId::ThrowKeg }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: super.handleCommand sets re_roll_source when CLIENT_USE_RE_ROLL → EXECUTE_STEP
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::TargetPlayerId(v) => { self.player_id = v.clone(); true }
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
    fn id_is_throw_keg() {
        assert_eq!(StepThrowKeg::new().id(), StepId::ThrowKeg);
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepThrowKeg::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_target_player_id() {
        let mut step = StepThrowKeg::new();
        let accepted = step.set_parameter(&StepParameter::TargetPlayerId(Some("tgt".into())));
        assert!(accepted);
        assert_eq!(step.player_id, Some("tgt".into()));
    }

    #[test]
    fn set_parameter_rejects_unknown() {
        let mut step = StepThrowKeg::new();
        let rejected = !step.set_parameter(&StepParameter::EndTurn(true));
        assert!(rejected);
    }
}
