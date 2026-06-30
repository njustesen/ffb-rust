use ffb_model::enums::{InducementPhase, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::step::util_server_steps::check_touchdown;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter, SequenceStep};
use crate::step::generator::bb2025::EndTurn;
use crate::step::generator::bb2025::end_turn::EndTurnParams;
use crate::step::generator::bb2020::Select;
use crate::step::generator::bb2020::select::SelectParams;
use crate::step::generator::common::Inducement;
use crate::step::generator::common::inducement::InducementParams;

/// Final step in the inducement sequence (BB2020). Consumes HomeTeam/InducementPhase/EndInducementPhase/EndTurn.
/// Routes to EndTurn, Select + StepCheckStalling, or back to Inducement sequence depending on phase and flags.
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.inducements.StepEndInducement.
///
/// BB2020 differs from BB2025 in the StartOfOwnTurn/end_inducement_phase branch:
/// Java pushes both a Select sequence AND a StepCheckStalling step on the stack.
/// There is also no check_forgo field (EndTurn is always called with check_forgo=false).
pub struct StepEndInducement {
    pub end_inducement_phase: bool,
    pub end_turn: bool,
    pub home_team: bool,
    pub inducement_phase: Option<InducementPhase>,
}

impl StepEndInducement {
    pub fn new() -> Self {
        Self {
            end_inducement_phase: false,
            end_turn: false,
            home_team: false,
            inducement_phase: None,
        }
    }
}

impl Default for StepEndInducement {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndInducement {
    fn id(&self) -> StepId { StepId::EndInducement }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::HomeTeam(v) => { self.home_team = *v; true }
            StepParameter::InducementPhase(v) => { self.inducement_phase = Some(*v); true }
            StepParameter::EndInducementPhase(v) => { self.end_inducement_phase = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            _ => false,
        }
    }
}

impl StepEndInducement {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(getGameState()) — no-op in headless Rust
        let phase = match self.inducement_phase {
            Some(p) => p,
            None => return StepOutcome::next(),
        };

        // Java: fEndTurn |= UtilServerSteps.checkTouchdown(getGameState())
        self.end_turn |= check_touchdown(game);

        match phase {
            InducementPhase::EndOfOwnTurn | InducementPhase::EndOfOpponentTurn => {
                game.turn_mode = TurnMode::Regular;
            }
            _ => {}
        }

        // BB2020: EndTurn is always called with check_forgo=false (no check_forgo field)
        let end_turn_seq = EndTurn::build_sequence(&EndTurnParams { check_forgo: false });

        if self.end_turn {
            return StepOutcome::next().push_seq(end_turn_seq);
        }

        if self.end_inducement_phase {
            match phase {
                InducementPhase::EndOfOpponentTurn => {
                    game.home_playing = !game.home_playing;
                    return StepOutcome::next().push_seq(end_turn_seq);
                }
                InducementPhase::StartOfOwnTurn => {
                    // Java BB2020:
                    //   ((Select) factory.forName(...)).pushSequence(new Select.SequenceParams(getGameState(), true));
                    //   getGameState().getStepStack().push(new StepCheckStalling(getGameState()));
                    // Both are pushed; StepCheckStalling runs first (stack is LIFO), then Select.
                    let select_seq = Select::build_sequence(&SelectParams {
                        update_persistence: true,
                        is_blitz_move: false,
                        block_targets: vec![],
                    });
                    let check_stalling_seq = vec![SequenceStep::new(StepId::CheckStalling)];
                    return StepOutcome::next()
                        .push_seq(select_seq)
                        .push_seq(check_stalling_seq);
                }
                _ => {}
            }
        } else {
            // Java: push another Inducement sequence (re-enter for remaining inducements)
            let seq = Inducement::build_sequence(&InducementParams {
                inducement_phase: phase,
                home_team: self.home_team,
                check_forgo: false,
            });
            return StepOutcome::next().push_seq(seq);
        }

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepId};
    use ffb_model::enums::{Rules, InducementPhase, TurnMode};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn no_phase_returns_next_immediately() {
        let mut game = make_game();
        let mut step = StepEndInducement::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.is_empty());
    }

    #[test]
    fn end_of_own_turn_sets_regular_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BetweenTurns;
        let mut step = StepEndInducement::new();
        step.inducement_phase = Some(InducementPhase::EndOfOwnTurn);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Regular);
    }

    #[test]
    fn end_of_opponent_turn_sets_regular_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BetweenTurns;
        let mut step = StepEndInducement::new();
        step.inducement_phase = Some(InducementPhase::EndOfOpponentTurn);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Regular);
    }

    #[test]
    fn end_turn_pushes_end_turn_sequence() {
        let mut game = make_game();
        let mut step = StepEndInducement::new();
        step.inducement_phase = Some(InducementPhase::EndOfOwnTurn);
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::ForgoneStalling);
    }

    #[test]
    fn end_of_opponent_turn_with_end_phase_flips_home_playing_and_pushes_end_turn() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepEndInducement::new();
        step.inducement_phase = Some(InducementPhase::EndOfOpponentTurn);
        step.end_inducement_phase = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.home_playing);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::ForgoneStalling);
    }

    #[test]
    fn start_of_own_turn_with_end_phase_pushes_select_and_check_stalling() {
        let mut game = make_game();
        let mut step = StepEndInducement::new();
        step.inducement_phase = Some(InducementPhase::StartOfOwnTurn);
        step.end_inducement_phase = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // Two pushes: select sequence and check_stalling sequence
        assert_eq!(out.pushes.len(), 2);
        // First push is the Select sequence (starts with InitSelecting)
        assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting);
        // Second push is CheckStalling
        assert_eq!(out.pushes[1][0].step_id, StepId::CheckStalling);
    }

    #[test]
    fn no_end_phase_pushes_inducement_sequence() {
        let mut game = make_game();
        let mut step = StepEndInducement::new();
        step.inducement_phase = Some(InducementPhase::BeforeSetup);
        step.end_inducement_phase = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitInducement);
    }

    #[test]
    fn set_parameter_home_team_accepted() {
        let mut step = StepEndInducement::new();
        assert!(step.set_parameter(&StepParameter::HomeTeam(true)));
        assert!(step.home_team);
    }

    #[test]
    fn set_parameter_inducement_phase_accepted() {
        let mut step = StepEndInducement::new();
        assert!(step.set_parameter(&StepParameter::InducementPhase(InducementPhase::BeforeSetup)));
        assert_eq!(step.inducement_phase, Some(InducementPhase::BeforeSetup));
    }

    #[test]
    fn set_parameter_check_forgo_not_accepted() {
        // BB2020 StepEndInducement has no check_forgo field; the parameter is not handled
        let mut step = StepEndInducement::new();
        assert!(!step.set_parameter(&StepParameter::CheckForgo(true)));
    }
}
