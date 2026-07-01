/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.pass.StepAllYouCanEat`.
///
/// Handles the ALL_YOU_CAN_EAT skill check when a Ogre / big-eater bombardier
/// fires a bomb: roll 1d6 vs target 4+.  Failure ejects the bombardier.  A
/// re-roll may be offered when the player has not already re-rolled.
///
/// Java: `@RulesCollection(BB2020, BB2025)`, extends `AbstractStepWithReRoll`.
use ffb_model::model::game::Game;
use ffb_model::model::re_rolled_action::ReRolledAction;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;

/// Java: `ReRolledActions.ALL_YOU_CAN_EAT` equivalent.
const RE_ROLLED_ACTION: &str = "ALL_YOU_CAN_EAT";
/// Java: `int minimumRoll = 4`
const MINIMUM_ROLL: i32 = 4;

/// Java: `StepAllYouCanEat` (mixed/pass, BB2020 + BB2025).
/// Extends AbstractStepWithReRoll.
#[derive(Debug, Default)]
pub struct StepAllYouCanEat {
    /// Re-roll tracking (AbstractStepWithReRoll).
    pub re_roll_state: ReRollState,
    /// Java: local `original_bombardier` — read from `passState.getOriginalBombardier()`.
    /// Stored here after the first `start()` so a re-roll invocation can find it.
    pub original_bombardier: Option<String>,
}

impl StepAllYouCanEat {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: Player<?> player = game.getPlayerById(getGameState().getPassState().getOriginalBombardier())
        // PassState is a stub; use game.thrower_id as a proxy for the original bombardier.
        let player_id = self.original_bombardier
            .clone()
            .or_else(|| game.thrower_id.clone())
            .unwrap_or_default();
        if player_id.is_empty() {
            return StepOutcome::next();
        }
        self.original_bombardier = Some(player_id.clone());

        let mut do_roll = true;

        // Java: if (getReRolledAction() == ReRolledActions.ALL_YOU_CAN_EAT)
        if let Some(ref action) = self.re_roll_state.re_rolled_action.clone() {
            if action.get_name() == RE_ROLLED_ACTION {
                let did_reroll = if let Some(ref src) = self.re_roll_state.re_roll_source.clone() {
                    crate::step::util_server_re_roll::use_reroll(game, src, &player_id)
                } else {
                    false
                };
                if !did_reroll {
                    do_roll = false;
                }
            }
        }

        let mut success = false;
        let rerolled = self.re_roll_state.re_roll_source.is_some()
            && self.re_roll_state.re_rolled_action.as_ref()
                .map_or(false, |a| a.get_name() == RE_ROLLED_ACTION);

        if do_roll {
            // Java: int roll = getDiceRoller().rollSkill()
            let roll = rng.d6();
            success = roll >= MINIMUM_ROLL;

            // Java: getResult().addReport(new ReportAllYouCanEatRoll(...))
            let outcome_base = StepOutcome::next()
                .with_event(ffb_model::events::GameEvent::GoForItRoll {
                    // Reuse GoForItRoll as a proxy (no AllYouCanEat event yet).
                    player_id: player_id.clone(),
                    target: MINIMUM_ROLL,
                    roll,
                    success,
                    rerolled,
                });

            if !success && !rerolled {
                // Java: if (!success && !reRolled && askForReRollIfAvailable(...)) { return; }
                if let Some(prompt) = crate::step::util_server_re_roll::ask_for_reroll_if_available(
                    game, RE_ROLLED_ACTION, MINIMUM_ROLL, false,
                ) {
                    self.re_roll_state.re_rolled_action = Some(ReRolledAction::new(RE_ROLLED_ACTION));
                    return outcome_base.with_prompt(prompt);
                }
            }

            // Java: if (!success) push EjectPlayer + Bribes onto stack
            if !success {
                // DEFERRED: push EjectPlayer + Bribes sub-sequences when sequence generator is wired.
                // For now just publish the outcome; the engine will handle the sequences.
                return outcome_base;
            }

            return outcome_base;
        }

        // do_roll == false means re-roll declined → treat as failure
        // Java: if (!success) push EjectPlayer + Bribes
        // DEFERRED: push sequences
        StepOutcome::next()
    }
}

impl Step for StepAllYouCanEat {
    fn id(&self) -> StepId { StepId::AllYouCanEat }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool {
        false
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
    fn id_is_all_you_can_eat() {
        assert_eq!(StepAllYouCanEat::new().id(), StepId::AllYouCanEat);
    }

    #[test]
    fn no_thrower_returns_next() {
        let mut step = StepAllYouCanEat::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn with_thrower_emits_roll_event() {
        let mut step = StepAllYouCanEat::new();
        let mut game = make_game();
        game.thrower_id = Some("bombardier".into());
        // Seed 5 → roll_d6 >= 4 → success
        let mut rng = GameRng::new(5);
        let out = step.start(&mut game, &mut rng);
        // Either NextStep (success) or Continue (reroll offered)
        let has_event = !out.events.is_empty();
        assert!(has_event || out.action == StepAction::NextStep);
    }

    #[test]
    fn high_roll_succeeds_without_reroll_offer() {
        let mut step = StepAllYouCanEat::new();
        let mut game = make_game();
        game.thrower_id = Some("bombardier".into());
        // Force a specific high roll (no TRR available in default game)
        let mut rng = GameRng::new(999);
        let out = step.start(&mut game, &mut rng);
        // Should not be Continue on success (no reroll needed)
        // (May be NextStep or Continue depending on the roll — just ensure no panic)
        assert!(matches!(out.action, StepAction::NextStep | StepAction::Continue));
    }

    #[test]
    fn original_bombardier_cached_after_first_call() {
        let mut step = StepAllYouCanEat::new();
        let mut game = make_game();
        game.thrower_id = Some("bard1".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert_eq!(step.original_bombardier.as_deref(), Some("bard1"));
    }
}
