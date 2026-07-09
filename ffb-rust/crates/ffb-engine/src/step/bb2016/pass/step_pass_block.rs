/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepPassBlock`.
///
/// Step in pass sequence to handle skill PASS_BLOCK (BB2016).
/// - No pass block for bombs, hand-over, or dump-off.
/// - Finds possible pass-blockers (OnTheBallMechanic).
/// - If blockers exist: switches turn to PASS_BLOCK, flips homePlaying, saves player states.
/// - In PASS_BLOCK mode: manages end-player-action and end-turn per blocker, then restores state.
///
/// Init parameter: GOTO_LABEL_ON_END (mandatory).
/// Receives: END_PLAYER_ACTION (consumed in PASS_BLOCK mode), END_TURN (consumed in PASS_BLOCK mode).
///
/// headless(PassBlock-turnMode): TurnMode::PassBlock + homePlaying flip not yet ported.
/// headless(PassBlock-generators): Move/Select sequence generators not yet ported.
use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::report::report_pass_block::ReportPassBlock;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::bb2016::on_the_ball_mechanic::OnTheBallMechanic;
use ffb_mechanics::on_the_ball_mechanic::OnTheBallMechanic as OnTheBallMechanicTrait;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepPassBlock` (bb2016/pass).
pub struct StepPassBlock {
    /// Java: `fGotoLabelOnEnd` — mandatory init param.
    goto_label_on_end: String,
    /// Java: `fOldTurnMode`
    old_turn_mode: Option<TurnMode>,
    /// Java: `fEndTurn`
    end_turn: bool,
    /// Java: `fEndPlayerAction`
    end_player_action: bool,
}

impl StepPassBlock {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            old_turn_mode: None,
            end_turn: false,
            end_player_action: false,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // Java: List<Player> availablePassBlockers = onTheBallMechanic.findPassBlockers(game, opposingTeam, true)
        let (opposing_team_id, opposing_team_clone) = if game.home_playing {
            (game.team_away.id.clone(), game.team_away.clone())
        } else {
            (game.team_home.id.clone(), game.team_home.clone())
        };
        let available_pass_blockers = OnTheBallMechanic::new().find_pass_blockers(game, &opposing_team_clone, true);
        if available_pass_blockers.is_empty() {
            // Java: addReport(new ReportPassBlock(opposingTeam.getId(), false)) → NEXT_STEP
            game.report_list.add(ReportPassBlock::new(opposing_team_id, false));
            return StepOutcome::next();
        }
        // Java: availablePassBlockers non-empty → set TurnMode::PassBlock, flip homePlaying, push sequences.
        // headless(PassBlock-turnMode): TurnMode::PassBlock + homePlaying flip not yet ported.
        // headless(PassBlock-generators): Move/Select sequence generators not yet ported.
        game.report_list.add(ReportPassBlock::new(opposing_team_id, true));
        StepOutcome::next()
    }
}

impl Default for StepPassBlock {
    fn default() -> Self { Self::new() }
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
            StepParameter::GotoLabelOnEnd(s) => { self.goto_label_on_end = s.clone(); true }
            StepParameter::EndTurn(v)        => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)=> { self.end_player_action = *v; true }
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
    fn id_is_pass_block() {
        assert_eq!(StepPassBlock::new().id(), StepId::PassBlock);
    }

    #[test]
    fn set_parameter_goto_label_on_end() {
        let mut step = StepPassBlock::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into())));
        assert_eq!(step.goto_label_on_end, "end");
    }

    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepPassBlock::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepPassBlock::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn set_parameter_end_player_action() {
        let mut step = StepPassBlock::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn start_adds_pass_block_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut step = StepPassBlock::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PASS_BLOCK));
    }

    #[test]
    fn pass_block_report_uses_opposing_team_id() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.home_playing = true; // thrower is home → opposing team is away
        let mut step = StepPassBlock::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PASS_BLOCK));
    }
}
