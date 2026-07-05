/// 1:1 translation of `com.fumbbl.ffb.server.step.StepResetToMove`.
///
/// @RulesCollection(RulesCollection.Rules.COMMON)
///
/// When `RESET_PLAYER_ACTION` is set:
///   - clears the step stack
///   - pushes a Move sequence (same as Move generator with default params)
///   - calls `UtilServerSteps.changePlayerAction` for the acting player
///   - clears `game.defender_id`
/// When not set: just advances to next step (NEXT_STEP).
///
/// The stack-clear is expressed via `StepOutcome::with_clear_stack()` — the driver
/// must honour this flag by emptying the stack before pushing the Move sub-sequence.
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_steps::change_player_action;

/// Java: `StepResetToMove` (common ruleset).
pub struct StepResetToMove {
    /// Java: `action` — the PlayerAction to reset to (from RESET_PLAYER_ACTION param).
    action: Option<PlayerAction>,
}

impl StepResetToMove {
    pub fn new() -> Self {
        Self { action: None }
    }
}

impl Default for StepResetToMove {
    fn default() -> Self { Self::new() }
}

impl Step for StepResetToMove {
    fn id(&self) -> StepId { StepId::ResetToMove }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: getResult().setNextAction(StepAction.NEXT_STEP);
        // Java: if (action != null) { stack.clear(); pushMoveSequence(); changePlayerAction(); game.setDefenderId(null); }
        if let Some(action) = self.action {
            // Resolve acting player info before mutating game
            let player_id = game.acting_player.player_id.clone();
            let jumping = game.acting_player.jumping;

            // Java: UtilServerSteps.changePlayerAction(this, actingPlayer.getPlayerId(), action, actingPlayer.isJumping())
            if let Some(ref pid) = player_id {
                change_player_action(game, pid, action, jumping);
            }

            // Java: game.setDefenderId(null)
            game.defender_id = None;

            // Build and push a Move sequence. The clear_stack flag tells the driver
            // to empty the stack before pushing (Java: gameState.getStepStack().clear()).
            let seq = build_move_sequence_for_rules(game.rules);
            return StepOutcome::next()
                .with_clear_stack()
                .push_seq(seq);
        }

        StepOutcome::next()
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ResetPlayerAction(a) => {
                self.action = Some(*a);
                // Java: consume(parameter) — mark consumed so it is not propagated further
                true // consumed
            }
            _ => false,
        }
    }
}

/// Build a Move sequence appropriate for the game's ruleset.
///
/// Java: `factory.forName(SequenceGenerator.Type.Move.name()).pushSequence(new Move.SequenceParams(gameState))`
/// The factory picks the rules-specific generator at runtime; we dispatch statically.
fn build_move_sequence_for_rules(rules: ffb_model::enums::Rules) -> Vec<crate::step::framework::SequenceStep> {
    use ffb_model::enums::Rules;
    match rules {
        Rules::Bb2016 | Rules::Common => {
            use crate::step::generator::bb2016::move_::{Move, MoveParams};
            Move::build_sequence(&MoveParams::default())
        }
        Rules::Bb2020 => {
            use crate::step::generator::bb2020::move_::{Move, MoveParams};
            Move::build_sequence(&MoveParams::default())
        }
        Rules::Bb2025 => {
            use crate::step::generator::bb2025::move_::{Move, MoveParams};
            Move::build_sequence(&MoveParams::default())
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use crate::action::Action;
    use ffb_model::enums::{Rules, PlayerAction};
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;

    fn make_game(rules: Rules) -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), rules)
    }

    #[test]
    fn id_is_reset_to_move() {
        assert_eq!(StepResetToMove::new().id(), StepId::ResetToMove);
    }

    #[test]
    fn no_action_returns_next_step_without_clear() {
        let mut step = StepResetToMove::new();
        let mut game = make_game(Rules::Bb2025);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.clear_stack);
        assert!(out.pushes.is_empty());
    }

    #[test]
    fn reset_player_action_clears_stack_and_pushes_move() {
        let mut step = StepResetToMove::new();
        let mut game = make_game(Rules::Bb2025);

        // Set up acting player
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.defender_id = Some("def".into());

        let consumed = step.set_parameter(&StepParameter::ResetPlayerAction(PlayerAction::Move));
        assert!(consumed, "RESET_PLAYER_ACTION must be consumed");

        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);

        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.clear_stack, "stack must be cleared when action is set");
        assert_eq!(out.pushes.len(), 1, "must push exactly one Move sequence");
        assert!(game.defender_id.is_none(), "defender_id must be cleared");
    }

    #[test]
    fn reset_player_action_consumed_returns_true() {
        let mut step = StepResetToMove::new();
        assert!(step.set_parameter(&StepParameter::ResetPlayerAction(PlayerAction::Block)));
        assert_eq!(step.action, Some(PlayerAction::Block));
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepResetToMove::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn handle_command_returns_next() {
        let mut step = StepResetToMove::new();
        let mut game = make_game(Rules::Bb2025);
        let mut rng = GameRng::new(0);
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn move_sequence_pushed_for_bb2016() {
        let mut step = StepResetToMove::new();
        let mut game = make_game(Rules::Bb2016);
        game.acting_player.player_id = Some("p1".into());
        step.set_parameter(&StepParameter::ResetPlayerAction(PlayerAction::Move));
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert!(!out.pushes.is_empty());
    }

    #[test]
    fn move_sequence_pushed_for_bb2020() {
        let mut step = StepResetToMove::new();
        let mut game = make_game(Rules::Bb2020);
        game.acting_player.player_id = Some("p1".into());
        step.set_parameter(&StepParameter::ResetPlayerAction(PlayerAction::Move));
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert!(!out.pushes.is_empty());
    }
}
