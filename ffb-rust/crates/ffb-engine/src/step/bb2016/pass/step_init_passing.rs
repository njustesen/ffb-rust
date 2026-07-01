/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepInitPassing`.
///
/// Initialization step of the pass sequence (BB2016).
/// - CLIENT_PASS: sets pass coordinate + catcher + thrower.
/// - CLIENT_HAND_OVER: sets catcher + pass coordinate from player.
/// - CLIENT_ACTING_PLAYER (no id): end player action.
/// - CLIENT_END_TURN: end turn.
///
/// Init parameters: GOTO_LABEL_ON_END (mandatory), TARGET_COORDINATE (opt), CATCHER_ID (opt).
/// Publishes: CATCHER_ID, END_TURN, END_PLAYER_ACTION, TARGET_COORDINATE.
///
/// DEFERRED(InitPassing-passCoordinate): DUMP_OFF / TurnMode.DUMP_OFF check not yet ported.
/// DEFERRED(InitPassing-rangeRuler): UtilRangeRuler.createRangeRuler not yet ported.
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepInitPassing` (bb2016/pass).
pub struct StepInitPassing {
    /// Java: `fGotoLabelOnEnd` — mandatory init param.
    goto_label_on_end: String,
    /// Java: `fCatcherId`
    catcher_id: Option<String>,
    /// Java: `fEndTurn`
    end_turn: bool,
    /// Java: `fEndPlayerAction`
    end_player_action: bool,
}

impl StepInitPassing {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            catcher_id: None,
            end_turn: false,
            end_player_action: false,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        if game.thrower_id.is_none() || game.thrower_action.is_none() {
            return StepOutcome::cont();
        }
        let catcher_id = self.catcher_id.clone();
        let mut out = StepOutcome::next();
        if let Some(ref id) = catcher_id {
            out = out.publish(StepParameter::CatcherId(Some(id.clone())));
        }
        if self.end_turn {
            return StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndTurn(true));
        }
        if self.end_player_action {
            return StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndPlayerAction(true));
        }
        // Java: if thrower==actingPlayer && isSufferingBloodLust && !hasFed → goto end
        let thrower_is_acting = game.thrower_id.is_some()
            && game.thrower_id == game.acting_player.player_id;
        if thrower_is_acting
            && game.acting_player.suffering_blood_lust
            && !game.acting_player.has_fed
        {
            return StepOutcome::goto(&self.goto_label_on_end);
        }
        // Java: actingPlayer.setHasPassed(true); turnData flags; game.setConcessionPossible(false)
        let thrower_action = game.thrower_action;
        game.acting_player.has_passed = true;
        game.concession_possible = false;
        game.turn_data_mut().turn_started = true;
        match thrower_action {
            Some(PlayerAction::HandOver) => {
                game.turn_data_mut().hand_over_used = true;
            }
            Some(PlayerAction::Pass) => {
                game.turn_data_mut().pass_used = true;
                // DEFERRED(InitPassing-rangeRuler): UtilRangeRuler.createRangeRuler not yet ported.
            }
            _ => {} // ThrowBomb, HailMaryBomb etc. — no extra TurnData flag
        }
        out
    }
}

impl Default for StepInitPassing {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitPassing {
    fn id(&self) -> StepId { StepId::InitPassing }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::Pass { coord } => {
                game.pass_coordinate = Some(*coord);
                let catcher = game.field_model.player_at(*coord).map(|id| id.clone());
                self.catcher_id = catcher.clone();
                game.thrower_id = game.acting_player.player_id.clone();
                game.thrower_action = game.acting_player.player_action;
                self.execute_step(game)
            }
            Action::EndTurn => {
                self.end_turn = true;
                self.execute_step(game)
            }
            // Java: CLIENT_ACTING_PLAYER (no id) → end player action
            // Note: Action::EndPlayerAction is not in the Rust Action enum — deferred
            // DEFERRED(InitPassing): EndPlayerAction command path not ported
            _ => StepOutcome::cont(),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(s) => { self.goto_label_on_end = s.clone(); true }
            StepParameter::CatcherId(v)      => { self.catcher_id = v.clone(); true }
            StepParameter::EndTurn(v)        => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)=> { self.end_player_action = *v; true }
            StepParameter::TargetCoordinate(c) => {
                // init-time TARGET_COORDINATE sets pass coordinate directly
                // DEFERRED(InitPassing-targetCoord): full catcher lookup from field_model
                let _ = c;
                true
            }
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
    fn id_is_init_passing() {
        assert_eq!(StepInitPassing::new().id(), StepId::InitPassing);
    }

    #[test]
    fn no_thrower_returns_continue() {
        let mut game = make_game();
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::Continue));
    }

    #[test]
    fn end_turn_goto_label() {
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(ffb_model::enums::PlayerAction::Pass);
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::GotoLabel));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn end_player_action_goto_label() {
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(ffb_model::enums::PlayerAction::Pass);
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::GotoLabel));
    }

    #[test]
    fn set_parameter_goto_label_on_end() {
        let mut step = StepInitPassing::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("x".into())));
        assert_eq!(step.goto_label_on_end, "x");
    }

    #[test]
    fn blood_lust_thrower_not_fed_goto_label() {
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(ffb_model::enums::PlayerAction::Pass);
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_fed = false;
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::GotoLabel));
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn blood_lust_thrower_already_fed_continues() {
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(ffb_model::enums::PlayerAction::Pass);
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_fed = true;
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // has_fed=true → does NOT goto label; falls through to NextStep
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn pass_action_sets_has_passed_and_pass_used() {
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(ffb_model::enums::PlayerAction::Pass);
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.acting_player.has_passed);
        assert!(game.turn_data().pass_used);
        assert!(!game.concession_possible);
    }

    #[test]
    fn hand_over_action_sets_hand_over_used() {
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(ffb_model::enums::PlayerAction::HandOver);
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.acting_player.has_passed);
        assert!(game.turn_data().hand_over_used);
    }
}
