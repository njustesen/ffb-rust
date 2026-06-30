/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.block.StepBlockBallAndChain`.
///
/// Handles the Ball and Chain skill in a block sequence.  If the acting player has the
/// `movesRandomly` property (Ball and Chain), and the old defender state is prone/stunned,
/// the defender is set to FALLING and a pushback is initiated.  Otherwise, NEXT_STEP.
///
/// Init parameter: `GOTO_LABEL_ON_PUSHBACK` (mandatory).
/// Expects `OLD_DEFENDER_STATE` from a preceding step.
use ffb_model::enums::{PlayerState, PS_FALLING};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::action::block::util_block_sequence::init_pushback;

/// Java: `StepBlockBallAndChain` (bb2016/block).
pub struct StepBlockBallAndChain {
    /// Java: `fGotoLabelOnPushback` — init parameter (mandatory).
    goto_label_on_pushback: String,
    /// Java: `fOldDefenderState`
    old_defender_state: Option<PlayerState>,
}

impl StepBlockBallAndChain {
    pub fn new() -> Self {
        Self { goto_label_on_pushback: String::new(), old_defender_state: None }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let attacker_has_property = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::MOVES_RANDOMLY))
            .unwrap_or(false);

        let old_prone_or_stunned = self.old_defender_state
            .map(|s| s.is_prone_or_stunned())
            .unwrap_or(false);

        if attacker_has_property && self.old_defender_state.is_some() && old_prone_or_stunned {
            if let (Some(defender_id), Some(old)) = (game.defender_id.clone(), self.old_defender_state) {
                game.field_model.set_player_state(&defender_id, old.change_base(PS_FALLING));
            }
            let pushback_params = init_pushback(game);
            let mut outcome = StepOutcome::goto(&self.goto_label_on_pushback);
            outcome.published.extend(pushback_params);
            outcome
        } else {
            StepOutcome::next()
        }
    }
}

impl Default for StepBlockBallAndChain {
    fn default() -> Self { Self::new() }
}

impl Step for StepBlockBallAndChain {
    fn id(&self) -> StepId { StepId::BlockBallAndChain }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnPushback(s) => { self.goto_label_on_pushback = s.clone(); true }
            StepParameter::OldDefenderState(s)    => { self.old_defender_state = Some(*s); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_PRONE, PS_STANDING};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_block_ball_and_chain() {
        assert_eq!(StepBlockBallAndChain::new().id(), StepId::BlockBallAndChain);
    }

    #[test]
    fn no_property_returns_next() {
        // has_skill_property stub always returns false → next step
        let mut step = StepBlockBallAndChain::new();
        step.set_parameter(&StepParameter::GotoLabelOnPushback("push".into()));
        step.set_parameter(&StepParameter::OldDefenderState(PlayerState::new(PS_PRONE)));
        let mut game = make_game();
        game.acting_player.player_id = None;
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
    }

    #[test]
    fn no_old_defender_state_returns_next() {
        let mut step = StepBlockBallAndChain::new();
        step.set_parameter(&StepParameter::GotoLabelOnPushback("push".into()));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
    }

    #[test]
    fn old_state_standing_returns_next() {
        let mut step = StepBlockBallAndChain::new();
        step.set_parameter(&StepParameter::GotoLabelOnPushback("push".into()));
        step.set_parameter(&StepParameter::OldDefenderState(PlayerState::new(PS_STANDING)));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
    }

    #[test]
    fn set_parameter_stores_label() {
        let mut step = StepBlockBallAndChain::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnPushback("lbl".into())));
        assert_eq!(step.goto_label_on_pushback, "lbl");
    }

    #[test]
    fn set_parameter_old_defender_state() {
        let mut step = StepBlockBallAndChain::new();
        let state = PlayerState::new(PS_PRONE);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(state)));
        assert_eq!(step.old_defender_state, Some(state));
    }

    #[test]
    fn init_pushback_clears_squares_and_returns_params() {
        use ffb_model::types::PushbackSquare;
        use ffb_model::enums::Direction;
        let mut game = make_game();
        game.field_model.pushback_squares.push(PushbackSquare::new(FieldCoordinate::new(1, 1), Direction::North, false));
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(5, 5));
        let params = init_pushback(&mut game);
        assert!(game.field_model.pushback_squares.is_empty());
        assert!(!params.is_empty());
    }
}
