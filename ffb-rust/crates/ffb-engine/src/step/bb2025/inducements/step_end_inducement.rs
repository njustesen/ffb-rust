use ffb_model::enums::{InducementPhase, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::step::util_server_steps::check_touchdown;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{SequenceStep, StepId, StepParameter};
use crate::step::generator::bb2025::{EndTurn, Select};
use crate::step::generator::bb2025::end_turn::EndTurnParams;
use crate::step::generator::bb2025::select::SelectParams;
use crate::step::generator::common::Inducement;
use crate::step::generator::common::inducement::InducementParams;

/// Final step in the inducement sequence. Consumes HomeTeam/InducementPhase/EndInducementPhase/EndTurn.
/// Routes to EndTurn, Select, or back to Inducement sequence depending on phase and flags.
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.inducements.StepEndInducement, with the
/// BB2016/BB2020 `StepCheckStalling` push edition-gated in (this one step serves all three
/// editions; `step/bb2020/inducements/step_end_inducement.rs` is a dead twin the driver never
/// dispatches).
pub struct StepEndInducement {
    pub end_inducement_phase: bool,
    pub end_turn: bool,
    pub home_team: bool,
    pub check_forgo: bool,
    pub inducement_phase: Option<InducementPhase>,
}

impl StepEndInducement {
    pub fn new(check_forgo: bool) -> Self {
        Self {
            end_inducement_phase: false,
            end_turn: false,
            home_team: false,
            check_forgo,
            inducement_phase: None,
        }
    }
}

impl Default for StepEndInducement {
    fn default() -> Self { Self::new(false) }
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
            StepParameter::CheckForgo(v) => { self.check_forgo = *v; true }
            _ => false,
        }
    }

    // Java: setParameter consume()s these keys — publish delivery stops here, so one
    // inducement window's leaveStep() publishes never clobber the other pending window.
    fn consumes_parameter(&self, param: &StepParameter) -> bool {
        matches!(param,
            StepParameter::HomeTeam(_) | StepParameter::InducementPhase(_)
            | StepParameter::EndInducementPhase(_) | StepParameter::EndTurn(_))
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

        let end_turn_seq = EndTurn::build_sequence(&EndTurnParams { check_forgo: self.check_forgo });

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
                    let seq = Select::build_sequence(&SelectParams { update_persistence: true, is_blitz_move: false, ..Default::default() });
                    // Java BB2016 (`bb2016/StepEndInducement:100-102`) and BB2020
                    // (`bb2020/inducements/StepEndInducement:113-115`) push the Select sequence and
                    // then `new StepCheckStalling(gameState)` on top of it, so the stalling check
                    // runs BEFORE Select. BB2025
                    // (`bb2025/inducements/StepEndInducement:124-126`) pushes Select only.
                    //
                    // This is the second of only two CHECK_STALLING push sites, and the one that
                    // matters: it constructs the step with no parameters, so `ignore_acted_flag`
                    // keeps its default `true` and `perform_check` does not require the acting
                    // player to have acted. The other site
                    // (`generator/bb2025/end_player_action.rs`, Java
                    // `generator/bb2020/EndPlayerAction:33`) passes `IGNORE_ACTED_FLAG = false`.
                    if game.rules != ffb_model::enums::Rules::Bb2025 {
                        let check_stalling_seq = vec![SequenceStep::new(StepId::CheckStalling)];
                        return StepOutcome::next().push_seq(seq).push_seq(check_stalling_seq);
                    }
                    return StepOutcome::next().push_seq(seq);
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
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn no_phase_returns_next_immediately() {
        let mut game = make_game();
        let mut step = StepEndInducement::new(false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.is_empty());
    }

    #[test]
    fn end_of_own_turn_sets_regular_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BetweenTurns;
        let mut step = StepEndInducement::new(false);
        step.inducement_phase = Some(InducementPhase::EndOfOwnTurn);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Regular);
    }

    #[test]
    fn end_of_opponent_turn_sets_regular_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BetweenTurns;
        let mut step = StepEndInducement::new(false);
        step.inducement_phase = Some(InducementPhase::EndOfOpponentTurn);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Regular);
    }

    #[test]
    fn end_turn_pushes_end_turn_sequence() {
        let mut game = make_game();
        let mut step = StepEndInducement::new(false);
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
        let mut step = StepEndInducement::new(false);
        step.inducement_phase = Some(InducementPhase::EndOfOpponentTurn);
        step.end_inducement_phase = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.home_playing);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::ForgoneStalling);
    }

    #[test]
    fn start_of_own_turn_with_end_phase_pushes_select_sequence() {
        let mut game = make_game();
        let mut step = StepEndInducement::new(false);
        step.inducement_phase = Some(InducementPhase::StartOfOwnTurn);
        step.end_inducement_phase = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting);
    }

    /// Java pushes `new StepCheckStalling(gameState)` on top of the Select sequence in
    /// `bb2016/StepEndInducement:102` and `bb2020/inducements/StepEndInducement:115`, and NOT in
    /// `bb2025/inducements/StepEndInducement:125`. This is the push site whose step carries the
    /// default `ignore_acted_flag = true`, which is what lets `perform_check` pass at the start of
    /// a turn when no player has acted yet -- the only way a Throw a Rock staller is ever recorded.
    ///
    /// Without the edition gate this step pushed Select alone for all three editions, so BB2016 and
    /// BB2020 reached `CHECK_STALLING` only through `EndPlayerAction` (which passes
    /// `IGNORE_ACTED_FLAG = false`): slann bb2020 seed 32 made 120 checks against Java's 164, and
    /// 0 of 32 at this site.
    #[test]
    fn start_of_own_turn_pushes_check_stalling_before_select_except_in_bb2025() {
        for (rules, expect_stalling) in [
            (Rules::Bb2016, true),
            (Rules::Bb2020, true),
            (Rules::Bb2025, false),
        ] {
            let mut game = Game::new(test_team("home", 0), test_team("away", 0), rules);
            let mut step = StepEndInducement::new(false);
            step.inducement_phase = Some(InducementPhase::StartOfOwnTurn);
            step.end_inducement_phase = true;
            let out = step.start(&mut game, &mut GameRng::new(0));
            assert_eq!(out.action, StepAction::NextStep);
            // The Select sequence is always pushed first; Java then pushes the single
            // CheckStalling step on top, so it is the one that runs first.
            assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting, "{rules:?}");
            if expect_stalling {
                assert_eq!(out.pushes.len(), 2, "{rules:?} should also push CheckStalling");
                assert_eq!(out.pushes[1].len(), 1, "{rules:?}");
                assert_eq!(out.pushes[1][0].step_id, StepId::CheckStalling, "{rules:?}");
            } else {
                assert_eq!(out.pushes.len(), 1, "{rules:?} must push Select only");
            }
        }
    }

    #[test]
    fn no_end_phase_pushes_inducement_sequence() {
        let mut game = make_game();
        let mut step = StepEndInducement::new(false);
        step.inducement_phase = Some(InducementPhase::BeforeSetup);
        step.end_inducement_phase = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitInducement);
    }

    #[test]
    fn set_parameter_home_team_accepted() {
        let mut step = StepEndInducement::new(false);
        assert!(step.set_parameter(&StepParameter::HomeTeam(true)));
        assert!(step.home_team);
    }

    #[test]
    fn set_parameter_inducement_phase_accepted() {
        let mut step = StepEndInducement::new(false);
        assert!(step.set_parameter(&StepParameter::InducementPhase(InducementPhase::BeforeSetup)));
        assert_eq!(step.inducement_phase, Some(InducementPhase::BeforeSetup));
    }

    #[test]
    fn set_parameter_check_forgo_accepted() {
        let mut step = StepEndInducement::new(false);
        assert!(step.set_parameter(&StepParameter::CheckForgo(true)));
        assert!(step.check_forgo);
    }
}
