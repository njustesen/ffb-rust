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
/// client-only: UtilRangeRuler.createRangeRuler — range ruler is client-side display only.
use ffb_model::enums::{PlayerAction, TurnMode};
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
    /// Stores the TARGET_COORDINATE init param for resolution in start() when game is available.
    pending_target_coordinate: Option<ffb_model::types::FieldCoordinate>,
}

impl StepInitPassing {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            catcher_id: None,
            end_turn: false,
            end_player_action: false,
            pending_target_coordinate: None,
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
                // client-only: UtilRangeRuler.createRangeRuler — client display only
            }
            Some(PlayerAction::DumpOff) => {
                // Java: DumpOff treated same as ThrowBomb; thrower.setHasPassed only if thrower==actingPlayer
                if game.thrower_id == game.acting_player.player_id {
                    game.acting_player.has_passed = true;
                }
                // client-only: range ruler not set for DumpOff
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
        // Java init(): if TARGET_COORDINATE set, resolve pass coordinate + catcher + thrower
        if let Some(coord) = self.pending_target_coordinate.take() {
            game.pass_coordinate = Some(coord);
            self.catcher_id = game.field_model.player_at(coord).map(|id| id.clone());
            game.thrower_id = game.acting_player.player_id.clone();
            game.thrower_action = game.acting_player.player_action;
        }
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::Pass { coord } => {
                game.pass_coordinate = Some(*coord);
                let catcher = game.field_model.player_at(*coord).map(|id| id.clone());
                self.catcher_id = catcher.clone();
                // Java: if defender != null && defenderAction == DUMP_OFF → thrower = defender
                if game.defender_id.is_some()
                    && matches!(game.defender_action, Some(PlayerAction::DumpOff))
                {
                    game.thrower_id = game.defender_id.clone();
                    game.thrower_action = game.defender_action;
                } else {
                    game.thrower_id = game.acting_player.player_id.clone();
                    game.thrower_action = game.acting_player.player_action;
                }
                self.execute_step(game)
            }
            Action::EndTurn => {
                self.end_turn = true;
                self.execute_step(game)
            }
            // Java: CLIENT_ACTING_PLAYER with no player_id → fEndPlayerAction = true → executeStep
            Action::ActivatePlayer { player_id, .. } if player_id.is_empty() => {
                self.end_player_action = true;
                self.execute_step(game)
            }
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
                // Store for resolution in start() when game is available.
                self.pending_target_coordinate = Some(*c);
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

    #[test]
    fn dump_off_defender_becomes_thrower() {
        let mut game = make_game();
        game.turn_mode = TurnMode::DumpOff;
        game.defender_id = Some("d1".into());
        game.defender_action = Some(ffb_model::enums::PlayerAction::DumpOff);
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        use ffb_model::types::FieldCoordinate;
        let action = crate::action::Action::Pass { coord: FieldCoordinate::new(8, 7) };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        // Thrower should be the defender, not the acting player
        assert_eq!(game.thrower_id.as_deref(), Some("d1"));
        assert_eq!(game.thrower_action, Some(ffb_model::enums::PlayerAction::DumpOff));
    }

    #[test]
    fn pass_action_non_dump_off_uses_acting_player() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(ffb_model::enums::PlayerAction::Pass);
        game.defender_id = None;
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        use ffb_model::types::FieldCoordinate;
        let action = crate::action::Action::Pass { coord: FieldCoordinate::new(10, 7) };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(game.thrower_id.as_deref(), Some("p1"));
    }
}
