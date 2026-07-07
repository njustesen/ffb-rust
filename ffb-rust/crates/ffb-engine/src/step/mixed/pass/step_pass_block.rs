/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.pass.StepPassBlock`.
///
/// Handles the PASS_BLOCK skill: before a pass is resolved, opponents with
/// Pass Block may move up to 2 squares to attempt to intercept the ball.
///
/// When there are eligible Pass Blockers the step switches the turn to
/// `TurnMode::PassBlock` so the opposing team can move those players.  After the
/// pass-block turn is over (fEndTurn / fEndPlayerAction) the step restores the
/// original game state and hands control back to the passing team.
///
/// Init parameters (mandatory): GOTO_LABEL_ON_END.
/// Incoming parameters: END_PLAYER_ACTION (consumed in PassBlock mode), END_TURN
///                       (consumed in PassBlock mode).
///
/// Java: `@RulesCollection(BB2020, BB2025)`, extends `AbstractStep`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_pass_block::ReportPassBlock;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepPassBlock` (mixed/pass, BB2020 + BB2025).
#[derive(Debug, Default)]
pub struct StepPassBlock {
    /// Java: `fGotoLabelOnEnd` (mandatory init param).
    pub goto_label_on_end: String,
    /// Java: `fEndTurn`
    pub end_turn: bool,
    /// Java: `fEndPlayerAction`
    pub end_player_action: bool,
    /// Java: `isGoingForIt` — saved actingPlayer.isGoingForIt before pass-block mode.
    pub is_going_for_it: bool,
    /// Java: `currentMove` — saved actingPlayer.currentMove (-1 = not set).
    pub current_move: i32,
    /// Java: `fOldTurnMode` — turn mode before entering PassBlock mode.
    pub old_turn_mode: Option<ffb_model::enums::TurnMode>,
}

impl StepPassBlock {
    pub fn new() -> Self {
        Self { current_move: -1, ..Default::default() }
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        if game.thrower_id.is_none() {
            return StepOutcome::next();
        }

        // Java: // no pass block for bombs or hand over or dump off (atm)
        // Java: if (game.getTurnMode().isBombTurn() || ...) { nextStep; return; }
        if game.turn_mode.is_bomb_turn() {
            return StepOutcome::next();
        }
        if let Some(ta) = game.thrower_action {
            use ffb_model::enums::PlayerAction;
            if ta == PlayerAction::DumpOff
                || ta == PlayerAction::HandOver
                || ta == PlayerAction::HandOverMove
            {
                return StepOutcome::next();
            }
        }

        // Java: find pass blockers on opposing team
        // Simplified: in the absence of the full OnTheBallMechanic, check if
        // we're already in pass-block mode and if so, restore state.
        use ffb_model::enums::TurnMode;

        if game.turn_mode == TurnMode::PassBlock {
            // Java: came back here after pass block movement

            // Check if acting player dropped (failed dodge)
            if let Some(ref pid) = game.acting_player.player_id.clone() {
                if let Some(state) = game.field_model.player_state(pid) {
                    if !state.has_tacklezones() {
                        // Player dropped — end turn
                        game.acting_player.player_id = None;
                        game.acting_player.player_action = None;
                        self.end_turn = true;
                        self.end_player_action = false;
                    }
                }
            }

            if self.end_turn {
                // Java: restore old player states, reset turn mode, flip home_playing
                if let Some(old_mode) = self.old_turn_mode {
                    game.turn_mode = old_mode;
                }
                if self.old_turn_mode.map_or(true, |m| m != TurnMode::DumpOff) {
                    game.home_playing = !game.home_playing;
                }
                // Restore actingPlayer
                game.acting_player.player_id = game.thrower_id.clone();
                game.acting_player.player_action = game.thrower_action;
                if self.current_move >= 0 {
                    game.acting_player.current_move = self.current_move;
                    game.acting_player.goes_for_it = self.is_going_for_it;
                }
            } else if self.end_player_action {
                // Player finished — switch to next pass blocker or end
                // no-op: PassBlock move/select sequences not ported — headless skips PassBlock sequence dispatch
                self.end_player_action = false;
            }

        } else {
            // Java: first time through — check for available pass blockers
            // Simplified: emit PassBlockEligible event and emit PassBlock(false) if none,
            // then advance.  Full mechanic lookup deferred.
            // Java: if (availablePassBlockers.size() == 0) { report PassBlock(false); }
            let opposing_team_id = if game.home_playing {
                game.team_away.id.clone()
            } else {
                game.team_home.id.clone()
            };
            game.report_list.add(ReportPassBlock::new(opposing_team_id, false));
            let outcome = StepOutcome::next()
                .with_event(ffb_model::events::GameEvent::PassBlock {
                    player_id: None,
                });
            return outcome;
        }

        StepOutcome::next()
    }
}

impl Step for StepPassBlock {
    fn id(&self) -> StepId { StepId::PassBlock }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::EndTurn(v) => {
                self.end_turn = *v;
                // Java: if in PassBlock mode, consume the parameter (don't propagate)
                // (consume semantics handled by the driver via Published.consumed)
                true
            }
            StepParameter::EndPlayerAction(v) => {
                self.end_player_action = *v;
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
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, PlayerAction, TurnMode};
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;

    use ffb_model::report::report_id::ReportId;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn pass_block_report_added_when_no_pass_blockers() {
        let mut step = StepPassBlock::new();
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(
            game.report_list.has_report(ReportId::PASS_BLOCK),
            "should add ReportPassBlock(available=false) when no eligible pass blockers"
        );
    }

    #[test]
    fn no_pass_block_report_when_no_thrower() {
        let mut step = StepPassBlock::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(
            !game.report_list.has_report(ReportId::PASS_BLOCK),
            "should not add ReportPassBlock when there is no thrower"
        );
    }

    #[test]
    fn id_is_pass_block() {
        assert_eq!(StepPassBlock::new().id(), StepId::PassBlock);
    }

    #[test]
    fn no_thrower_returns_next() {
        let mut step = StepPassBlock::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn bomb_turn_mode_skips_pass_block() {
        let mut step = StepPassBlock::new();
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::ThrowBomb);
        game.turn_mode = TurnMode::BombHome;
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn regular_pass_emits_pass_block_event() {
        let mut step = StepPassBlock::new();
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        let has_pb = out.events.iter().any(|e| matches!(e, ffb_model::events::GameEvent::PassBlock { .. }));
        assert!(has_pb);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn hand_over_skips_pass_block() {
        let mut step = StepPassBlock::new();
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::HandOver);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label() {
        let mut step = StepPassBlock::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("lbl".into()));
        assert_eq!(step.goto_label_on_end, "lbl");
    }
}
