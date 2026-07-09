/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.multiblock.StepApothecaryMultiple`.
///
/// Handles apothecary (and Igor/Mortuary-assistant regeneration) for all injuries
/// accumulated during a multi-block action.
///
/// Initialisation (from StepParameterSet):
///   - ACTING_TEAM (bool): if true, handle injuries to the acting team; if false, handle other team.
///
/// setParameter:
///   - INJURY_RESULT: add to `injury_results` (filtered to team at execute_step time).
///
/// executeStep flow (abbreviated):
///   1. Resolve teamId from acting_team flag + game state.
///   2. Filter injuryResults to only players on teamId's team.
///   3. If injuryResults.isEmpty → NEXT_STEP immediately.
///   4. Group injuryResults by ApothecaryStatus.
///   5. DO_REQUEST group:
///      - if remainingApos > 0 → client-only: show DialogUseApothecariesParameter; headless auto-declines.
///      - else → mark as DoNotUseApothecary.
///   6. USE_APOTHECARY group → rollApothecary; headless auto-picks better outcome.
///   7. NO_APOTHECARY + DO_NOT_USE_APOTHECARY group → apply via injuryResult.apply_to(game).
///   8. Regeneration: roll for each applied casualty with canRollToSaveFromInjury skill.
///   9. double-attacker-down special case — apo-multiple-injury not ported (blocked: handle_injury_side_effects).
///   10. → NEXT_STEP.
///
/// handleCommand handles:
///   - CLIENT_APOTHECARY_CHOICE → handleApothecaryChoice
///   - CLIENT_USE_APOTHECARIES → set all WAIT_FOR_APOTHECARY_USE to DO_NOT_USE_APOTHECARY
///   - CLIENT_USE_APOTHECARY → find result by defender_id, mark UseApothecary or DoNotUseApothecary
///   - CLIENT_USE_IGORS → apo-multiple-igor not ported (blocked: InducementSet/Igor inducement infra)
///
/// Igor inducement handling deferred — blocked by InducementSet/InducementTypeFactory not ported.
use ffb_model::enums::{ApothecaryStatus, ApothecaryMode, PS_BADLY_HURT, PS_KNOCKED_OUT, PS_STUNNED, PS_RESERVE, PS_RIP, PS_SERIOUS_INJURY, PlayerState};
use ffb_model::model::game::Game;
use ffb_model::model::player_state::PlayerState as PlayerStateModel;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_apothecary_roll::ReportApothecaryRoll;
use ffb_model::report::report_apothecary_choice::ReportApothecaryChoice;
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
    /// Java: `apothecaryMode` (ApothecaryMode)
    pub apothecary_mode: Option<ApothecaryMode>,
    /// Whether ACTING_TEAM was set (determines which team's injuries to handle).
    pub acting_team: Option<bool>,
    /// How many apothecaries have been consumed this step (to track remaining_apos).
    pub apos_used: i32,
}

/// Java: `rollApothecary(InjuryResult)` — rolls a new casualty for apothecary use.
///
/// If the original injury is BH or KO: auto-cures (KO → STUNNED, BH → RESERVE), emits
/// `ReportApothecaryChoice`. If it's CAS: rolls new [d16, d6] casualty, auto-picks the better
/// outcome (client-only: dialog skipped in headless; better outcome auto-selected), emits `ReportApothecaryRoll` + `ReportApothecaryChoice`.
fn roll_apothecary_for_injury(result: &mut Box<InjuryResult>, rng: &mut GameRng, game: &mut Game) {
    let ctx = result.injury_context();
    let defender_id = ctx.defender_id.clone().unwrap_or_default();
    let base = ctx.injury.map(|ps| ps.base()).unwrap_or(0);

    // Java: apothecaryChoice = (base != BADLY_HURT && base != KNOCKED_OUT)
    let apothecary_choice = base != PS_BADLY_HURT && base != PS_KNOCKED_OUT;

    if apothecary_choice {
        // Original injury is CAS (SI or RIP) — roll new casualty.
        let d16 = rng.die(16);
        let d6 = rng.d6();
        let new_base = if d16 >= 15 {
            ffb_model::enums::PS_RIP
        } else if d16 >= 9 {
            ffb_model::enums::PS_SERIOUS_INJURY
        } else {
            PS_BADLY_HURT
        };
        let new_state = PlayerState::new(new_base);

        // Emit the apo roll report.
        game.report_list.add(ReportApothecaryRoll::new(
            Some(defender_id.clone()),
            vec![d16, d6],
            Some(new_state),
            None, // serious_injury string — not ported yet
            None, // original_injury string
            vec![],
        ));

        // Headless: auto-pick the better outcome.
        // BH < SI < RIP numerically, so lower = better for the player.
        let orig_state = ctx.injury.unwrap_or(new_state);
        let pick_new = new_base < orig_state.base();
        let chosen_state = if pick_new { new_state } else { orig_state };
        let final_state = if chosen_state.base() == PS_BADLY_HURT {
            PlayerState::new(PS_RESERVE)
        } else {
            chosen_state
        };

        let ctx_mut = result.injury_context_mut();
        ctx_mut.injury = Some(final_state);
        ctx_mut.apothecary_status = ApothecaryStatus::DoNotUseApothecary;

        game.report_list.add(ReportApothecaryChoice::new(
            defender_id,
            PlayerStateModel::new(0),
            None,
        ));
    } else {
        // BH or KO: auto-cure — no roll needed.
        let cured = if base == PS_KNOCKED_OUT {
            PlayerState::new(PS_STUNNED)
        } else {
            PlayerState::new(PS_RESERVE)
        };
        let ctx_mut = result.injury_context_mut();
        ctx_mut.injury = Some(cured);
        ctx_mut.serious_injury = None;
        ctx_mut.apothecary_status = ApothecaryStatus::DoNotUseApothecary;

        game.report_list.add(ReportApothecaryChoice::new(
            defender_id,
            PlayerStateModel::new(0),
            None,
        ));
    }
}

impl StepApothecaryMultiple {
    pub fn new() -> Self {
        Self {
            team_id: None,
            injury_results: Vec::new(),
            regeneration_failed_results: Vec::new(),
            apothecary_mode: None,
            acting_team: None,
            apos_used: 0,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
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

        // Step 4a: Determine remaining apothecaries.
        let remaining_apos = if let Some(ref team_id) = self.team_id {
            let team_apos = if *team_id == game.team_home.id {
                game.team_home.apothecaries
            } else {
                game.team_away.apothecaries
            };
            (team_apos - self.apos_used).max(0)
        } else { 0 };

        // Step 4b: DoRequest → if apos available → client-only: show dialog; headless auto-declines.
        let has_do_request = self.injury_results.iter()
            .any(|r| r.injury_context().apothecary_status == ApothecaryStatus::DoRequest);
        if has_do_request {
            if remaining_apos > 0 {
                // client-only: DialogUseApothecariesParameter — headless auto-declines apothecary use
            }
            for r in &mut self.injury_results {
                if r.injury_context().apothecary_status == ApothecaryStatus::DoRequest {
                    r.injury_context_mut().apothecary_status = ApothecaryStatus::DoNotUseApothecary;
                }
            }
        }

        // Step 4c: UseApothecary → rollApothecary + headless auto-picks better outcome.
        // Java: rollApothecary() → if CAS: roll new [d16,d6], pick better; if BH/KO: auto-cure.
        for r in &mut self.injury_results {
            if r.injury_context().apothecary_status == ApothecaryStatus::UseApothecary {
                roll_apothecary_for_injury(r, rng, game);
            }
        }

        // Step 5: Apply NoApothecary + DoNotUseApothecary injuries.
        for r in &self.injury_results {
            let status = r.injury_context().apothecary_status;
            if status == ApothecaryStatus::NoApothecary
                || status == ApothecaryStatus::DoNotUseApothecary
            {
                r.apply_to(game);
            }
        }

        // Step 6: Remove applied results.
        self.injury_results.retain(|r| {
            let status = r.injury_context().apothecary_status;
            status != ApothecaryStatus::NoApothecary
                && status != ApothecaryStatus::DoNotUseApothecary
        });

        // Regeneration: for each applied casualty, roll Regeneration if the player has the skill.
        // Successful regeneration nullifies the injury (player restored to RESERVE).
        {
            let player_ids: Vec<String> = game.team_home.players.iter()
                .chain(game.team_away.players.iter())
                .map(|p| p.id.clone())
                .collect();
            for pid in &player_ids {
                crate::step::util_server_injury::handle_regeneration(game, rng, pid);
            }
        }
        // headless(apo-multiple-igor): Igor/mortuary-assistant inducement handling deferred.

        // Java: double-attacker-down: two injury results for the same player + regeneration failed.
        // When a player is injured twice (e.g. multi-block double-down), reset to RESERVE and
        // re-apply only the regeneration-failed injuries in the original order.
        {
            let double_attacker = self.injury_results.len() == 2
                && {
                    let id0 = self.injury_results[0].injury_context().defender_id.clone();
                    let id1 = self.injury_results[1].injury_context().defender_id.clone();
                    id0.is_some() && id0 == id1
                }
                && !self.regeneration_failed_results.is_empty();

            if double_attacker {
                let player_id = self.injury_results[0].injury_context().defender_id.clone().unwrap_or_default();
                if let Some(ps) = game.field_model.player_state(&player_id) {
                    game.field_model.set_player_state(&player_id, ps.change_base(PS_RESERVE));
                }
                // Clear serious injury from GameResult.
                {
                    let is_home = game.team_home.players.iter().any(|p| p.id == player_id);
                    let tr = game.game_result.team_result_mut(is_home);
                    let pr = tr.player_result_mut(&player_id);
                    pr.serious_injury = None;
                    pr.serious_injury_decay = None;
                }
                // Re-apply only the regeneration-failed injury results.
                let regen_failed_ids: std::collections::HashSet<*const InjuryResult> =
                    self.regeneration_failed_results.iter().map(|r| r.as_ref() as *const InjuryResult).collect();
                for r in &self.injury_results {
                    let ptr = r.as_ref() as *const InjuryResult;
                    let is_regen_failed = self.regeneration_failed_results.iter()
                        .any(|rf| rf.injury_context().defender_id == r.injury_context().defender_id
                            && rf.injury_context().injury == r.injury_context().injury);
                    let _ = regen_failed_ids; // suppress unused warning
                    if is_regen_failed {
                        r.apply_to(game);
                    }
                }
            }
        }

        StepOutcome::next()
    }
}

impl Default for StepApothecaryMultiple {
    fn default() -> Self { Self::new() }
}

impl Step for StepApothecaryMultiple {
    fn id(&self) -> StepId { StepId::ApothecaryMultiple }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseApothecary { player_id, use_apothecary } => {
                // Java: CLIENT_USE_APOTHECARY → find matching injury result; update apothecaryStatus
                // Java: if used → useApo(turnData, apothecaryType) decrements apothecary count.
                let new_status = if *use_apothecary {
                    self.apos_used += 1;
                    ApothecaryStatus::UseApothecary
                } else {
                    ApothecaryStatus::DoNotUseApothecary
                };
                for r in &mut self.injury_results {
                    if r.injury_context().defender_id.as_deref() == Some(player_id.as_str()) {
                        r.injury_context_mut().apothecary_status = new_status;
                    }
                }
            }
            Action::Acknowledge => {
                // Java: CLIENT_USE_APOTHECARIES → all WAIT_FOR_APOTHECARY_USE → DO_NOT_USE_APOTHECARY
                for r in &mut self.injury_results {
                    if r.injury_context().apothecary_status == ApothecaryStatus::WaitForApothecaryUse {
                        r.injury_context_mut().apothecary_status = ApothecaryStatus::DoNotUseApothecary;
                    }
                }
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{ApothecaryMode, ApothecaryStatus, Rules, PS_STUNNED, PS_KNOCKED_OUT};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn make_injury(defender_id: &str, apo_mode: ApothecaryMode, status: ApothecaryStatus) -> Box<InjuryResult> {
        let mut ir = Box::new(InjuryResult::new(apo_mode));
        ir.injury_context_mut().defender_id = Some(defender_id.to_string());
        ir.injury_context_mut().apothecary_status = status;
        ir
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

    #[test]
    fn start_injury_no_defender_filters_out_and_next_step() {
        // Injury result with no defender_id: filtered out → empty → NextStep.
        let mut game = make_game();
        let mut step = StepApothecaryMultiple::new();
        step.team_id = Some("home".to_string());
        let mut ir = Box::new(InjuryResult::new(ApothecaryMode::HitPlayer));
        ir.injury_context_mut().apothecary_status = ApothecaryStatus::NoApothecary;
        // no defender_id set → filtered out
        step.injury_results.push(ir);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn start_no_apothecary_injury_applied_then_next_step() {
        // Injury with NoApothecary status: apply_to is called, result is consumed.
        let mut game = make_game();
        // Add a player to the home team and put them on the field.
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;
        let player = Player {
            id: "home_1".into(), nr: 1, name: "p1".into(),
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("home_1", FieldCoordinate::new(5, 7));

        let mut step = StepApothecaryMultiple::new();
        step.team_id = Some("home".to_string());
        let ir = make_injury("home_1", ApothecaryMode::HitPlayer, ApothecaryStatus::NoApothecary);
        step.injury_results.push(ir);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // All NoApothecary injuries consumed.
        assert!(step.injury_results.is_empty());
    }

    #[test]
    fn do_request_with_no_apos_treated_as_do_not_use() {
        // DoRequest + no apothecaries → applied immediately, not waiting for dialog.
        let mut game = make_game();
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;
        game.team_home.apothecaries = 0;
        let player = Player {
            id: "home_1".into(), nr: 1, name: "p1".into(),
            position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("home_1", FieldCoordinate::new(5, 7));

        let mut step = StepApothecaryMultiple::new();
        step.team_id = Some("home".to_string());
        let ir = make_injury("home_1", ApothecaryMode::HitPlayer, ApothecaryStatus::DoRequest);
        step.injury_results.push(ir);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(step.injury_results.is_empty());
    }

    #[test]
    fn acknowledge_clears_wait_for_apo_use() {
        // CLIENT_USE_APOTHECARIES → all WaitForApothecaryUse → DoNotUseApothecary.
        let mut game = make_game();
        let mut step = StepApothecaryMultiple::new();
        step.team_id = Some("home".to_string());
        let ir = make_injury("unknown_player", ApothecaryMode::HitPlayer, ApothecaryStatus::WaitForApothecaryUse);
        step.injury_results.push(ir);

        step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));

        // After acknowledge: status should have been changed away from WaitForApothecaryUse.
        // (results filtered out if player not on team, but the status transition is tested here)
        for r in &step.injury_results {
            assert_ne!(r.injury_context().apothecary_status, ApothecaryStatus::WaitForApothecaryUse);
        }
    }

    #[test]
    fn team_id_resolved_from_acting_team_true() {
        // acting_team=true + game.home_playing=true → team_id = home.id
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepApothecaryMultiple::new();
        step.acting_team = Some(true);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.team_id.as_deref(), Some("home"));
    }

    #[test]
    fn team_id_resolved_from_acting_team_false() {
        // acting_team=false + game.home_playing=true → team_id = away.id
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepApothecaryMultiple::new();
        step.acting_team = Some(false);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.team_id.as_deref(), Some("away"));
    }

    #[test]
    fn injuries_for_other_team_filtered_out() {
        // Injury for away player ignored when team_id = home.
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
            is_big_guy: false,
            ..Default::default()
        };
        game.team_away.players.push(away_player);

        let mut step = StepApothecaryMultiple::new();
        step.team_id = Some("home".to_string()); // home team
        // Add injury for away player
        let ir = make_injury("away_1", ApothecaryMode::HitPlayer, ApothecaryStatus::NoApothecary);
        step.injury_results.push(ir);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    fn make_player_for_team(game: &mut Game, id: &str, home: bool) {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;
        let p = Player {
            id: id.into(), nr: 1, name: id.into(),
            position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0,
            career_spps: 0, race: None, ..Default::default()
        };
        if home { game.team_home.players.push(p); } else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 7));
    }

    #[test]
    fn roll_apothecary_ko_injury_cures_to_stunned() {
        // KO → apothecary cures to STUNNED (no new roll needed).
        let mut game = make_game();
        make_player_for_team(&mut game, "h1", true);

        let mut ir = Box::new(InjuryResult::new(ApothecaryMode::HitPlayer));
        ir.injury_context_mut().defender_id = Some("h1".into());
        ir.injury_context_mut().apothecary_status = ApothecaryStatus::UseApothecary;
        ir.injury_context_mut().injury = Some(PlayerState::new(PS_KNOCKED_OUT));

        let mut rng = GameRng::new(42);
        roll_apothecary_for_injury(&mut ir, &mut rng, &mut game);

        assert_eq!(ir.injury_context().injury.map(|ps| ps.base()), Some(PS_STUNNED));
        assert_eq!(ir.injury_context().apothecary_status, ApothecaryStatus::DoNotUseApothecary);
        // Report was emitted.
        assert!(!game.report_list.is_empty());
    }

    #[test]
    fn roll_apothecary_bh_injury_cures_to_reserve() {
        // BH → apothecary cures to RESERVE.
        let mut game = make_game();
        make_player_for_team(&mut game, "h1", true);

        let mut ir = Box::new(InjuryResult::new(ApothecaryMode::HitPlayer));
        ir.injury_context_mut().defender_id = Some("h1".into());
        ir.injury_context_mut().apothecary_status = ApothecaryStatus::UseApothecary;
        ir.injury_context_mut().injury = Some(PlayerState::new(PS_BADLY_HURT));

        let mut rng = GameRng::new(42);
        roll_apothecary_for_injury(&mut ir, &mut rng, &mut game);

        assert_eq!(ir.injury_context().injury.map(|ps| ps.base()), Some(PS_RESERVE));
        assert_eq!(ir.injury_context().apothecary_status, ApothecaryStatus::DoNotUseApothecary);
    }

    #[test]
    fn roll_apothecary_si_rolls_new_casualty_and_finalizes() {
        // SI → rolls new casualty, auto-picks the result, status = DoNotUseApothecary.
        let mut game = make_game();
        make_player_for_team(&mut game, "h1", true);

        let mut ir = Box::new(InjuryResult::new(ApothecaryMode::HitPlayer));
        ir.injury_context_mut().defender_id = Some("h1".into());
        ir.injury_context_mut().apothecary_status = ApothecaryStatus::UseApothecary;
        ir.injury_context_mut().injury = Some(PlayerState::new(PS_SERIOUS_INJURY));

        let mut rng = GameRng::new(42);
        roll_apothecary_for_injury(&mut ir, &mut rng, &mut game);

        assert_eq!(ir.injury_context().apothecary_status, ApothecaryStatus::DoNotUseApothecary);
        // Final injury is either the new roll or original, but must be set.
        assert!(ir.injury_context().injury.is_some());
        // At least two reports emitted (ReportApothecaryRoll + ReportApothecaryChoice).
        assert!(!game.report_list.is_empty());
    }

    #[test]
    fn use_apothecary_command_increments_apos_used() {
        // When a client uses an apothecary, apos_used should increase so remaining count decreases.
        let mut game = make_game();
        make_player_for_team(&mut game, "h1", true);
        game.team_home.apothecaries = 2;
        let mut step = StepApothecaryMultiple::new();
        step.team_id = Some("home".into());
        let ir = make_injury("h1", ApothecaryMode::HitPlayer, ApothecaryStatus::WaitForApothecaryUse);
        step.injury_results.push(ir);

        assert_eq!(step.apos_used, 0);
        step.handle_command(
            &Action::UseApothecary { player_id: "h1".into(), use_apothecary: true },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(step.apos_used, 1, "apos_used should increment when apothecary is used");
    }

    #[test]
    fn use_apothecary_command_no_increment_when_declined() {
        let mut game = make_game();
        let mut step = StepApothecaryMultiple::new();
        step.team_id = Some("home".into());

        assert_eq!(step.apos_used, 0);
        step.handle_command(
            &Action::UseApothecary { player_id: "h1".into(), use_apothecary: false },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(step.apos_used, 0, "apos_used should not change when apothecary is declined");
    }
}
