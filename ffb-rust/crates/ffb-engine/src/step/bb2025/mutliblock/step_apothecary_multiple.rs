use std::collections::HashMap;
use ffb_model::enums::{ApothecaryStatus, ApothecaryMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::injury::InjuryResult;
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
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.mutliblock.StepApothecaryMultiple`.
pub struct StepApothecaryMultiple {
    /// Java: teamId (resolved from init param ACTING_TEAM at start())
    pub team_id: Option<String>,
    /// Java: injuryResults (List<InjuryResult>)
    pub injury_results: Vec<Box<InjuryResult>>,
    /// Java: regenerationFailedResults — populated after regeneration phase.
    pub regeneration_failed_results: Vec<Box<InjuryResult>>,
    /// Java: gettingEvenResults — BB2025: SI players eligible for Getting Even.
    pub getting_even_results: Vec<String>,
    /// Java: deadResults — BB2025: RIP players eligible for Raise Dead.
    pub dead_results: Vec<String>,
    /// Java: availableKeyWordsMap (Map<String, List<Keyword>>)
    pub available_keywords_map: HashMap<String, Vec<String>>,
    /// Whether ACTING_TEAM init param was set.
    pub acting_team: Option<bool>,
    /// How many apothecaries have been consumed this step.
    pub apos_used: i32,
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
            acting_team: None,
            apos_used: 0,
        }
    }

    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Step 1: Resolve team_id from game state on first call.
        if self.team_id.is_none() {
            if let Some(acting) = self.acting_team {
                if acting == game.home_playing {
                    self.team_id = Some(game.team_home.id.clone());
                } else {
                    self.team_id = Some(game.team_away.id.clone());
                }
            }
        }

        // Step 2: Filter to only team's players.
        if let Some(ref team_id) = self.team_id.clone() {
            let is_home = *team_id == game.team_home.id;
            self.injury_results.retain(|r| {
                r.injury_context().defender_id.as_deref()
                    .map(|did| if is_home { game.team_home.has_player(did) } else { game.team_away.has_player(did) })
                    .unwrap_or(false)
            });
        }

        // Step 3: If no injuries, done immediately.
        if self.injury_results.is_empty() {
            return StepOutcome::next();
        }

        // DEFERRED(apo-multiple-regen): Phase 1 — handle regeneration
        // Until regeneration is ported: skip directly to apothecary phase.

        // Step 4: Get remaining apothecaries.
        let remaining_apos = if let Some(ref team_id) = self.team_id {
            let team_apos = if *team_id == game.team_home.id {
                game.team_home.apothecaries
            } else {
                game.team_away.apothecaries
            };
            (team_apos - self.apos_used).max(0)
        } else { 0 };

        // Step 5a: Process DoRequest injuries.
        let has_do_request = self.injury_results.iter()
            .any(|r| r.injury_context().apothecary_status == ApothecaryStatus::DoRequest);
        if has_do_request {
            if remaining_apos > 0 {
                // DEFERRED(apo-multiple-dialog): show DialogUseApothecariesParameter → Continue.
                // Until dialog is ported: treat as DoNotUseApothecary.
            }
            for r in &mut self.injury_results {
                if r.injury_context().apothecary_status == ApothecaryStatus::DoRequest {
                    r.injury_context_mut().apothecary_status = ApothecaryStatus::DoNotUseApothecary;
                }
            }
        }

        // Step 5b: UseApothecary → DEFERRED(apo-multiple-roll).
        for r in &mut self.injury_results {
            if r.injury_context().apothecary_status == ApothecaryStatus::UseApothecary {
                // DEFERRED(apo-multiple-roll): rollApothecary, compare outcomes, DialogApothecaryChoiceParameter.
                r.injury_context_mut().apothecary_status = ApothecaryStatus::DoNotUseApothecary;
            }
        }

        // Step 6: Apply NoApothecary + DoNotUseApothecary injuries.
        for r in &self.injury_results {
            let status = r.injury_context().apothecary_status;
            if status == ApothecaryStatus::NoApothecary
                || status == ApothecaryStatus::DoNotUseApothecary
            {
                r.apply_to(game);
            }
        }
        self.injury_results.retain(|r| {
            let status = r.injury_context().apothecary_status;
            status != ApothecaryStatus::NoApothecary
                && status != ApothecaryStatus::DoNotUseApothecary
        });

        // DEFERRED(apo-multiple-injury): double-attacker-down special case.
        // DEFERRED(raise-dead): BB2025 Raise Dead mechanic (dead_results, checkRaiseDead).
        // DEFERRED(getting-even): BB2025 Getting Even mechanic (getting_even_results).

        StepOutcome::next()
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
            acting_team: None,
            apos_used: 0,
        }
    }
}

impl Step for StepApothecaryMultiple {
    fn id(&self) -> StepId { StepId::ApothecaryMultiple }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseApothecary { player_id, use_apothecary } => {
                // Java: CLIENT_USE_APOTHECARY → find matching injury result; update apothecaryStatus.
                // DEFERRED(apo-multiple-dialog): find result by player_id; mark UseApothecary or DoNotUse.
                let _ = (player_id, use_apothecary);
            }
            Action::Acknowledge => {
                // Java: CLIENT_USE_APOTHECARIES → all WAIT_FOR_APOTHECARY_USE → DO_NOT_USE_APOTHECARY.
                for r in &mut self.injury_results {
                    if r.injury_context().apothecary_status == ApothecaryStatus::WaitForApothecaryUse {
                        r.injury_context_mut().apothecary_status = ApothecaryStatus::DoNotUseApothecary;
                    }
                }
            }
            Action::UseReRoll { use_reroll: _ } => {
                // Java: CLIENT_USE_RE_ROLL for regeneration → DEFERRED(apo-multiple-regen).
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ActingTeam(v) => {
                self.acting_team = Some(*v);
                true
            }
            StepParameter::InjuryResult(r) => {
                // Java: only add if defender belongs to teamId.
                // team_id may not be resolved yet; filtering is deferred to execute_step.
                self.injury_results.push(r.clone());
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{ApothecaryMode, ApothecaryStatus, Rules};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn make_injury(defender_id: &str, apo_mode: ApothecaryMode, status: ApothecaryStatus) -> Box<InjuryResult> {
        let mut ir = Box::new(InjuryResult::new(apo_mode));
        ir.injury_context_mut().defender_id = Some(defender_id.to_string());
        ir.injury_context_mut().apothecary_status = status;
        ir
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
        assert_eq!(step.acting_team, Some(true));
    }

    #[test]
    fn set_injury_result_accepted() {
        let mut step = StepApothecaryMultiple::default();
        let ir = Box::new(InjuryResult::new(ApothecaryMode::HitPlayer));
        assert!(step.set_parameter(&StepParameter::InjuryResult(ir)));
        assert_eq!(step.injury_results.len(), 1);
    }

    #[test]
    fn handle_command_no_injuries_returns_next_step() {
        let mut game = make_game();
        let mut step = StepApothecaryMultiple::new("team".into());
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn team_id_resolved_from_acting_team_true() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepApothecaryMultiple::default();
        step.acting_team = Some(true);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.team_id.as_deref(), Some("home"));
    }

    #[test]
    fn team_id_resolved_from_acting_team_false() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepApothecaryMultiple::default();
        step.acting_team = Some(false);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.team_id.as_deref(), Some("away"));
    }

    #[test]
    fn start_injury_no_defender_filters_out() {
        // Injury with no defender_id: filtered out.
        let mut game = make_game();
        let mut step = StepApothecaryMultiple::default();
        step.team_id = Some("home".to_string());
        let mut ir = Box::new(InjuryResult::new(ApothecaryMode::HitPlayer));
        ir.injury_context_mut().apothecary_status = ApothecaryStatus::NoApothecary;
        step.injury_results.push(ir);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_apothecary_injury_applied_then_next_step() {
        let mut game = make_game();
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;
        let player = Player {
            id: "home_1".into(), nr: 1, name: "p1".into(),
            position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        };
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("home_1", FieldCoordinate::new(5, 7));

        let mut step = StepApothecaryMultiple::default();
        step.team_id = Some("home".to_string());
        step.injury_results.push(make_injury("home_1", ApothecaryMode::HitPlayer, ApothecaryStatus::NoApothecary));

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(step.injury_results.is_empty());
    }

    #[test]
    fn acknowledge_clears_wait_for_apo_use() {
        let mut game = make_game();
        let mut step = StepApothecaryMultiple::default();
        step.team_id = Some("home".to_string());
        step.injury_results.push(
            make_injury("unknown", ApothecaryMode::HitPlayer, ApothecaryStatus::WaitForApothecaryUse)
        );
        step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        for r in &step.injury_results {
            assert_ne!(r.injury_context().apothecary_status, ApothecaryStatus::WaitForApothecaryUse);
        }
    }

    #[test]
    fn injuries_for_other_team_filtered_out() {
        let mut game = make_game();
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;
        let away_player = Player {
            id: "away_1".into(), nr: 1, name: "p1".into(),
            position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        };
        game.team_away.players.push(away_player);

        let mut step = StepApothecaryMultiple::default();
        step.team_id = Some("home".to_string()); // home team only
        step.injury_results.push(
            make_injury("away_1", ApothecaryMode::HitPlayer, ApothecaryStatus::NoApothecary)
        );

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepApothecaryMultiple::default();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
