use ffb_model::enums::InducementPhase;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::mixed::EndTurn;
use crate::step::generator::bb2016::Select;
use crate::step::generator::bb2016::select::SelectParams;
use crate::step::generator::common::Inducement;
use crate::step::generator::common::inducement::InducementParams;
use crate::step::generator::sequence::{Sequence, SequenceStep};
use crate::step::util_server_steps::check_touchdown;

/// Final step in the inducement sequence (BB2016).
/// Routes to EndTurn, Select+CheckStalling, or back to Inducement sequence.
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepEndInducement.
pub struct StepEndInducement {
    /// Java: fEndInducementPhase
    pub end_inducement_phase: bool,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fHomeTeam
    pub home_team: bool,
    /// Java: fInducementPhase
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

        let end_turn_seq = EndTurn::build_sequence();

        if self.end_turn {
            return StepOutcome::next().push_seq(end_turn_seq);
        }

        if self.end_inducement_phase {
            match phase {
                InducementPhase::EndOfOwnTurn => {
                    return StepOutcome::next().push_seq(end_turn_seq);
                }
                InducementPhase::StartOfOwnTurn => {
                    // Java: push Select, then push StepCheckStalling (LIFO → CheckStalling runs first)
                    let select_seq = Select::build_sequence(&SelectParams { update_persistence: true });
                    let mut check_seq = Sequence::new();
                    check_seq.add(StepId::CheckStalling, vec![]);
                    return StepOutcome::next()
                        .push_seq(select_seq)
                        .push_seq(check_seq.build());
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

    fn check_stalling_seq() -> Vec<SequenceStep> {
        let mut s = Sequence::new();
        s.add(StepId::CheckStalling, vec![]);
        s.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepId};
    use ffb_model::enums::{Rules, InducementPhase};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
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
    fn end_turn_flag_pushes_end_turn_sequence() {
        let mut game = make_game();
        let mut step = StepEndInducement::new();
        step.inducement_phase = Some(InducementPhase::EndOfOwnTurn);
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::EndTurn);
    }

    #[test]
    fn end_of_own_turn_with_end_phase_pushes_end_turn_sequence() {
        let mut game = make_game();
        let mut step = StepEndInducement::new();
        step.inducement_phase = Some(InducementPhase::EndOfOwnTurn);
        step.end_inducement_phase = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::EndTurn);
    }

    #[test]
    fn start_of_own_turn_with_end_phase_pushes_select_and_check_stalling() {
        let mut game = make_game();
        let mut step = StepEndInducement::new();
        step.inducement_phase = Some(InducementPhase::StartOfOwnTurn);
        step.end_inducement_phase = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Two sequences pushed: select_seq first, then check_stalling (LIFO → check runs first)
        assert_eq!(out.pushes.len(), 2);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting);
        assert_eq!(out.pushes[1][0].step_id, StepId::CheckStalling);
    }

    #[test]
    fn no_end_phase_pushes_inducement_sequence() {
        let mut game = make_game();
        let mut step = StepEndInducement::new();
        step.inducement_phase = Some(InducementPhase::BeforeSetup);
        step.end_inducement_phase = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
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
    fn set_parameter_end_turn_accepted() {
        let mut step = StepEndInducement::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepEndInducement::new();
        assert!(!step.set_parameter(&StepParameter::CheckForgo(true)));
    }
}
