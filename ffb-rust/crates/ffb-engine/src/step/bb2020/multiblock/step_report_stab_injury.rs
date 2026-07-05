/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.multiblock.StepReportStabInjury`.
///
/// Reports a stab injury result in the multi-block context.
///
/// Initialisation (mandatory):
///   - `target` (PLAYER_ID): the stabbed player's ID.
///
/// Start:
///   - If `injury_result` is set, calls `injuryResult.report(this)` to emit the injury report.
///   - Always → NEXT_STEP.
///
/// setParameter:
///   - INJURY_RESULT: stores the incoming InjuryResult.
///
/// This step does not handle any commands — it runs once on start().
///
/// headless: InjuryResult.report() — blocked on report system infrastructure.
/// headless: StepException when PLAYER_ID not provided — blocked on report system infrastructure.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::injury::InjuryResult;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepReportStabInjury` (bb2020/multiblock).
pub struct StepReportStabInjury {
    /// Java: `target` — the stabbed player's ID (mandatory init param).
    pub target: Option<String>,
    /// Java: `injuryResult` — the injury that was sustained.
    pub injury_result: Option<Box<InjuryResult>>,
}

impl StepReportStabInjury {
    pub fn new() -> Self {
        Self {
            target: None,
            injury_result: None,
        }
    }
}

impl Default for StepReportStabInjury {
    fn default() -> Self { Self::new() }
}

impl Step for StepReportStabInjury {
    fn id(&self) -> StepId { StepId::ReportStabInjury }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (injuryResult != null) { injuryResult.report(this); }
        // headless: call injuryResult.report() when report system is ported.
        if self.injury_result.is_some() {
            // report deferred
        }
        // Java: getResult().setNextAction(StepAction.NEXT_STEP)
        StepOutcome::next()
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // This step does not handle commands — Java has no handleCommand override.
        StepOutcome::next()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: StepParameterKey.PLAYER_ID (init via StepParameterSet in init())
            StepParameter::PlayerId(id) => {
                self.target = Some(id.clone());
                true
            }
            // Java: StepParameterKey.INJURY_RESULT (set via setParameter)
            StepParameter::InjuryResult(r) => {
                self.injury_result = Some(r.clone());
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
    use ffb_model::enums::{ApothecaryMode, Rules};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_report_stab_injury() {
        assert_eq!(StepReportStabInjury::new().id(), StepId::ReportStabInjury);
    }

    #[test]
    fn start_returns_next_step_when_no_injury() {
        let mut game = make_game();
        let mut step = StepReportStabInjury::new();
        step.target = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn start_returns_next_step_with_injury() {
        let mut game = make_game();
        let mut step = StepReportStabInjury::new();
        step.target = Some("p2".into());
        step.injury_result = Some(Box::new(InjuryResult::new(ApothecaryMode::HitPlayer)));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_player_id() {
        let mut step = StepReportStabInjury::new();
        assert!(step.set_parameter(&StepParameter::PlayerId("target_p".into())));
        assert_eq!(step.target, Some("target_p".into()));
    }

    #[test]
    fn set_parameter_injury_result() {
        let mut step = StepReportStabInjury::new();
        let ir = Box::new(InjuryResult::new(ApothecaryMode::HitPlayer));
        assert!(step.set_parameter(&StepParameter::InjuryResult(ir)));
        assert!(step.injury_result.is_some());
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepReportStabInjury::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepReportStabInjury::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
