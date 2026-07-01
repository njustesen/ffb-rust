/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.multiblock.StepApothecaryMultiple`.
///
/// Handles apothecary (and Igor/Mortuary-assistant regeneration) for all injuries
/// accumulated during a multi-block action.
///
/// Initialisation (from StepParameterSet):
///   - ACTING_TEAM (bool): if true, handle injuries to the acting team; if false, handle other team.
///
/// setParameter:
///   - INJURY_RESULT: if the injured player belongs to `teamId`, add to `injury_results`.
///
/// executeStep flow (abbreviated):
///   1. If injuryResults.isEmpty → NEXT_STEP immediately.
///   2. hideDialog.
///   3. Group injuryResults by ApothecaryStatus.
///   4. DO_REQUEST group → for each: report injury; syncGameModel; build InjuryDescription list.
///      If remainingApos > 0 && apoTypes not empty → WAIT_FOR_APOTHECARY_USE;
///      showDialog(DialogUseApothecariesParameter) → Continue.
///   5. USE_APOTHECARY group → rollApothecary; if apothecaryChoice needed → WAIT_FOR_APOTHECARY_CHOICE;
///      showDialog(DialogApothecaryChoiceParameter) → Continue.
///   6. DO_NOT_USE_APOTHECARY group → addReport(ReportApothecaryRoll null null null null modifiers).
///   7. NO_APOTHECARY group → injuryResult.report(this).
///   8. Igor/regeneration handling (WAIT_FOR_IGOR_USE, USE_IGOR, DO_NOT_USE_IGOR).
///      If remaining igor uses left → showDialog(DialogUseMortuaryAssistantsParameter) → Continue.
///   9. Double-attacker-down special case: reset player state; reapply injuries.
///   10. Handle injury side effects → NEXT_STEP.
///
/// handleCommand handles:
///   - CLIENT_APOTHECARY_CHOICE → handleApothecaryChoice
///   - CLIENT_USE_APOTHECARIES → set all WAIT_FOR_APOTHECARY_USE to DO_NOT_USE_APOTHECARY
///   - CLIENT_USE_APOTHECARY → mark specific player's apo; decrement remaining apos
///   - CLIENT_USE_IGORS → process igor selections
///
/// DEFERRED(apo-multiple-injury): InjuryResult, ApothecaryStatus, ApothecaryType fully deferred.
/// DEFERRED(apo-multiple-dialog): All dialog parameters (DialogUseApothecariesParameter etc.) deferred.
/// DEFERRED(apo-multiple-roll): rollApothecary(), remainingApos(), useApo() deferred.
/// DEFERRED(apo-multiple-regen): UtilServerInjury.handleRegeneration() / side effects deferred.
/// DEFERRED(apo-multiple-igor): DialogUseMortuaryAssistantsParameter / Igor inducement handling deferred.
/// DEFERRED(apo-multiple-init): ACTING_TEAM → teamId resolution from game state deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::injury::InjuryResult;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepApothecaryMultiple` (bb2020/multiblock).
/// Note: BB2020 version differs from BB2025 in regeneration: uses `InducementType.REGENERATION`
/// usage check and `DialogUseMortuaryAssistantsParameter` for Igor selection.
pub struct StepApothecaryMultiple {
    /// Java: `teamId` — resolved from ACTING_TEAM init param at start().
    pub team_id: Option<String>,
    /// Java: `injuryResults` (List<InjuryResult>)
    pub injury_results: Vec<Box<InjuryResult>>,
    /// Java: `regenerationFailedResults` (List<InjuryResult>)
    pub regeneration_failed_results: Vec<Box<InjuryResult>>,
    /// Java: `apothecaryMode` (ApothecaryMode) — deferred.
    pub apothecary_mode: Option<String>,
    /// Whether ACTING_TEAM was set (determines which team's injuries to handle).
    pub acting_team: Option<bool>,
}

impl StepApothecaryMultiple {
    pub fn new() -> Self {
        Self {
            team_id: None,
            injury_results: Vec::new(),
            regeneration_failed_results: Vec::new(),
            apothecary_mode: None,
            acting_team: None,
        }
    }

    fn execute_step(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (injuryResults.isEmpty()) { setNextAction(NEXT_STEP); }
        if self.injury_results.is_empty() {
            return StepOutcome::next();
        }

        // DEFERRED(apo-multiple-dialog): UtilServerDialog.hideDialog(getGameState())
        // DEFERRED(apo-multiple-injury): group injuryResults by ApothecaryStatus
        // DEFERRED(apo-multiple-injury): DO_REQUEST → report; syncGameModel; build descriptions
        //   if remainingApos > 0 → WAIT_FOR_APOTHECARY_USE → showDialog → Continue
        // DEFERRED(apo-multiple-roll): USE_APOTHECARY → rollApothecary → if choice → Continue
        // DEFERRED(apo-multiple-injury): DO_NOT_USE_APOTHECARY → addReport(ReportApothecaryRoll)
        // DEFERRED(apo-multiple-injury): NO_APOTHECARY → injuryResult.report(this)
        // DEFERRED(apo-multiple-regen): regeneration inducement types; igor/mortuary-assistants dialog
        // DEFERRED(apo-multiple-injury): apply all non-igor results; doubleAttackerDown edge case
        // DEFERRED(apo-multiple-regen): UtilServerInjury.handleInjurySideEffects

        // Stub: advance immediately until injury infrastructure is ported.
        StepOutcome::next()
    }
}

impl Default for StepApothecaryMultiple {
    fn default() -> Self { Self::new() }
}

impl Step for StepApothecaryMultiple {
    fn id(&self) -> StepId { StepId::ApothecaryMultiple }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: init(StepParameterSet) runs at construction; we resolve teamId here at start()
        // since we need the game state.
        // DEFERRED(apo-multiple-init): resolve teamId from acting_team flag + game.getActingTeam()
        if self.team_id.is_none() {
            if let Some(acting) = self.acting_team {
                if acting {
                    // DEFERRED: self.team_id = Some(game.acting_team_id().clone())
                } else {
                    // DEFERRED: self.team_id = Some(game.other_team_id().clone())
                }
            }
        }
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_APOTHECARY_CHOICE → handleApothecaryChoice (update injury state)
        // Java: CLIENT_USE_APOTHECARIES → mark all WAIT_FOR_APOTHECARY_USE as DO_NOT_USE_APOTHECARY
        // Java: CLIENT_USE_APOTHECARY → mark specific player apo; useApo() to decrement counts
        // Java: CLIENT_USE_IGORS → process igor injury descriptions
        match action {
            Action::UseApothecary { player_id, use_apothecary } => {
                // DEFERRED(apo-multiple-injury): find matching injury result by player_id;
                //   if use_apothecary: check remainingApos; set USE_APOTHECARY / DO_NOT_USE_APOTHECARY
                //   else: set DO_NOT_USE_APOTHECARY
                let _ = (player_id, use_apothecary);
            }
            Action::Acknowledge => {
                // Java: CLIENT_USE_APOTHECARIES → all WAIT_FOR_APOTHECARY_USE → DO_NOT_USE_APOTHECARY
                // DEFERRED(apo-multiple-injury): update all matching injury results
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: StepParameterKey.ACTING_TEAM (from init StepParameterSet)
            StepParameter::ActingTeam(v) => {
                self.acting_team = Some(*v);
                true
            }
            // Java: StepParameterKey.INJURY_RESULT → add if player belongs to teamId
            StepParameter::InjuryResult(r) => {
                // DEFERRED(apo-multiple-injury): check r.injuryContext().getDefenderId() ∈ teamId's players
                self.injury_results.push(r.clone());
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
    fn id_is_apothecary_multiple() {
        assert_eq!(StepApothecaryMultiple::new().id(), StepId::ApothecaryMultiple);
    }

    #[test]
    fn start_no_injuries_returns_next_step() {
        let mut game = make_game();
        let mut step = StepApothecaryMultiple::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_acting_team_accepted() {
        let mut step = StepApothecaryMultiple::new();
        assert!(step.set_parameter(&StepParameter::ActingTeam(true)));
        assert_eq!(step.acting_team, Some(true));
    }

    #[test]
    fn set_injury_result_accepted() {
        let mut step = StepApothecaryMultiple::new();
        let ir = Box::new(InjuryResult::new(ApothecaryMode::HitPlayer));
        assert!(step.set_parameter(&StepParameter::InjuryResult(ir)));
        assert_eq!(step.injury_results.len(), 1);
    }

    #[test]
    fn start_with_injuries_returns_next_step_stub() {
        // Stub: until injury infrastructure is ported, step advances even with injuries.
        let mut game = make_game();
        let mut step = StepApothecaryMultiple::new();
        step.injury_results.push(Box::new(InjuryResult::new(ApothecaryMode::HitPlayer)));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepApothecaryMultiple::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn handle_command_no_injuries_returns_next_step() {
        let mut game = make_game();
        let mut step = StepApothecaryMultiple::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn acting_team_false_sets_flag() {
        let mut step = StepApothecaryMultiple::new();
        step.set_parameter(&StepParameter::ActingTeam(false));
        assert_eq!(step.acting_team, Some(false));
    }
}
