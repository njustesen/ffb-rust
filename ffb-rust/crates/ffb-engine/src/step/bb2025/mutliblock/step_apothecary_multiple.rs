use std::collections::HashMap;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepAction, StepId, StepParameter};

/// Handles apothecary (and Raise Dead / Getting Even) across multiple injuries
/// from a multiblock.
///
/// Java executeStep flow (abbreviated):
///   if injuryResults.isEmpty -> NEXT_STEP; return
///   hideDialog
///
///   // Phase 1: regeneration (first call sets up regenerationFailedResults)
///   if !regenerationHandled -> Continue; return
///
///   // Phase 2: apothecary
///   groupedInjuries = regenerationFailedResults grouped by ApothecaryStatus
///   DO_REQUEST group: build injuryDescriptions; if remainingApos>0 && apoTypes not empty:
///     setStatus(WAIT_FOR_APOTHECARY_USE); showDialog(DialogUseApothecariesParameter) -> Continue
///   USE_APOTHECARY group: rollApothecary for each; if choice needed: showDialog -> Continue
///
///   // Phase 3: apply
///   handle doubleAttackerDown edge case; applyTo each result; syncGameModel
///
///   // Phase 4: Getting Even (keyword selection)
///   for SI results with available keywords: gettingEvenResults
///
///   // Phase 5: Raise Dead
///   for dead results with raiseable opponents: deadResults; checkRaiseDead
///
///   // Dispatch
///   if deadResults.isEmpty: checkGettingEven; else checkRaiseDead
///
/// handleCommand handles:
///   CLIENT_USE_INDUCEMENT -> regeneration re-roll via inducement
///   CLIENT_USE_RE_ROLL -> regeneration re-roll via team re-roll
///   CLIENT_APOTHECARY_CHOICE -> confirm apo choice -> executeStep
///   CLIENT_USE_APOTHECARIES -> skip apo use -> executeStep
///   CLIENT_USE_APOTHECARY -> mark specific apo use -> executeStep
///   CLIENT_KEYWORD_SELECTION -> Getting Even keyword chosen -> push GettingEven sequence
///   CLIENT_POSITION_SELECTION -> Raise Dead position chosen -> raisePlayer
///
/// setParameter: INJURY_RESULT -> add to injuryResults if team matches teamId
///
/// init parameters: ACTING_TEAM (bool) -> determines teamId from actingTeam or otherTeam.
///
/// Unported utilities:
///   TODO: InjuryResult type (injury context, apo status, regeneration)
///   TODO: regenerationHandled() logic
///   TODO: ApothecaryStatus enum
///   TODO: rollApothecary() / handleApothecaryChoice()
///   TODO: InjuryMechanic.canRaiseDead / raisePositions / raiseType
///   TODO: GettingEven sequence push
///   TODO: Dialog parameters (all apothecary / getting even / raise dead dialogs)
///   TODO: UtilServerInjury.handleRegeneration
///   TODO: UtilServerInducementUse / InducementType / Usage.REGENERATION
///   TODO: StateMechanic.handlePumpUp
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.mutliblock.StepApothecaryMultiple`.
pub struct StepApothecaryMultiple {
    /// Java: teamId (resolved from init param ACTING_TEAM)
    pub team_id: Option<String>,
    /// Java: injuryResults (List<InjuryResult>) — stored as names until InjuryResult is ported
    pub injury_results: Vec<String>,
    /// Java: regenerationFailedResults (initially null until first executeStep call)
    pub regeneration_failed_results: Vec<String>,
    /// Java: gettingEvenResults
    pub getting_even_results: Vec<String>,
    /// Java: deadResults
    pub dead_results: Vec<String>,
    /// Java: availableKeyWordsMap (Map<String, List<Keyword>>)
    pub available_keywords_map: HashMap<String, Vec<String>>,
}

impl StepApothecaryMultiple {
    pub fn new(team_id: String) -> Self {
        Self {
            team_id: Some(team_id),
            injury_results: Vec::new(),
            regeneration_failed_results: Vec::new(),
            getting_even_results: Vec::new(),
            dead_results: Vec::new(),
            available_keywords_map: HashMap::new(),
        }
    }
}

impl Default for StepApothecaryMultiple {
    fn default() -> Self {
        Self {
            team_id: None,
            injury_results: Vec::new(),
            regeneration_failed_results: Vec::new(),
            getting_even_results: Vec::new(),
            dead_results: Vec::new(),
            available_keywords_map: HashMap::new(),
        }
    }
}

impl Step for StepApothecaryMultiple {
    fn id(&self) -> StepId { StepId::ApothecaryMultiple }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java handles many CLIENT_* commands; map what we can via Action variants.
        match action {
            Action::UseApothecary { player_id: _, use_apothecary: _ } => {
                // CLIENT_USE_APOTHECARY -> mark apo use for the specific player
                // DEFERRED: find matching injuryResult; update apothecaryStatus
            }
            Action::Acknowledge => {
                // CLIENT_USE_APOTHECARIES (skip all apos)
                // DEFERRED: set all WAIT_FOR_APOTHECARY_USE results to DO_NOT_USE_APOTHECARY
            }
            Action::UseReRoll { use_reroll: _ } => {
                // CLIENT_USE_RE_ROLL for regeneration
                // DEFERRED: find preRegen result; use team re-roll for regeneration
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ActingTeam(_v) => {
                // Java: ACTING_TEAM boolean -> resolve teamId from actingTeam or otherTeam
                // DEFERRED: game not available here; defer to start()
                true
            }
            _ => false,
        }
    }
}

impl StepApothecaryMultiple {
    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // No injury results -> nothing to do.
        if self.injury_results.is_empty() {
            return StepOutcome::next();
        }

        // DEFERRED: hideDialog
        // DEFERRED: if !regenerationHandled() -> Continue; return
        // DEFERRED: group regenerationFailedResults by ApothecaryStatus
        // DEFERRED: DO_REQUEST: show DialogUseApothecariesParameter -> Continue
        // DEFERRED: USE_APOTHECARY: rollApothecary; if choice needed -> Continue
        // DEFERRED: apply all results; handle doubleAttackerDown; syncGameModel
        // DEFERRED: collect Getting Even results; collect dead/raiseable results
        // DEFERRED: checkRaiseDead / checkGettingEven

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_no_injuries_returns_next_step() {
        let mut game = make_game();
        let mut step = StepApothecaryMultiple::new("home_lineman".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn default_no_injuries_returns_next_step() {
        let mut game = make_game();
        let mut step = StepApothecaryMultiple::default();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn new_stores_team_id() {
        let step = StepApothecaryMultiple::new("team_x".into());
        assert_eq!(step.team_id.as_deref(), Some("team_x"));
    }

    #[test]
    fn set_acting_team_accepted() {
        let mut step = StepApothecaryMultiple::default();
        assert!(step.set_parameter(&StepParameter::ActingTeam(true)));
    }

    #[test]
    fn handle_command_no_injuries_returns_next_step() {
        let mut game = make_game();
        let mut step = StepApothecaryMultiple::new("team".into());
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
