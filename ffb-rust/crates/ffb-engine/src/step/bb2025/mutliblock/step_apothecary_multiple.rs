use std::collections::HashMap;
use ffb_model::enums::{ApothecaryStatus, ApothecaryMode, Keyword, PS_BADLY_HURT, PS_KNOCKED_OUT, PS_STUNNED, PS_PRONE, PS_RESERVE, PS_RIP, PS_SERIOUS_INJURY, PlayerState};
use ffb_model::model::game::Game;
use ffb_model::model::player_state::PlayerState as PlayerStateModel;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_apothecary_roll::ReportApothecaryRoll;
use ffb_model::report::report_apothecary_choice::ReportApothecaryChoice;
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
/// Java: `rollApothecary(InjuryResult)` — rolls a new casualty for apothecary use.
///
/// If BH or KO: auto-cures (emits `ReportApothecaryChoice`).
/// If CAS: rolls new casualty, headless auto-picks the better outcome, emits both reports.
fn roll_apothecary_for_injury(result: &mut Box<InjuryResult>, rng: &mut GameRng, game: &mut Game) -> Option<ffb_model::prompts::AgentPrompt> {
    let ctx = result.injury_context();
    let defender_id = ctx.defender_id.clone().unwrap_or_default();
    let base = ctx.injury.map(|ps| ps.base()).unwrap_or(0);
    let mut apothecary_choice = base != PS_BADLY_HURT && base != PS_KNOCKED_OUT;

    if apothecary_choice {
        // Java: a new InjuryResult rolled and interpreted by the edition's RollMechanic.
        let mechanic = crate::mechanic::roll_mechanic_for(game.rules);
        let mut new_ctx = crate::injury::InjuryContext::new(ApothecaryMode::Defender);
        new_ctx.defender_id = Some(defender_id.clone());
        let casualty_roll = mechanic.roll_casualty(rng);
        new_ctx.casualty_roll = Some(casualty_roll);
        let new_state = game.player(&defender_id)
            .and_then(|d| mechanic.interpret_casualty_roll_and_add_modifiers(game, &mut new_ctx, d, false));
        let new_si = mechanic.interpret_serious_injury_roll(game, &mut new_ctx);
        apothecary_choice = new_state.map(|s| s.base()) != Some(PS_BADLY_HURT);
        let si_name = |k| crate::mechanic::state_mechanic::serious_injury_display_name(k, game.rules);
        let casualty_modifiers: Vec<String> = result.injury_context().casualty_modifiers.iter().map(|m| m.name.to_string()).collect();
        game.report_list.add(ReportApothecaryRoll::new(
            Some(defender_id.clone()), casualty_roll.to_vec(), new_state, new_si.map(si_name),
            new_ctx.original_serious_injury.map(si_name), casualty_modifiers,
        ));
        if apothecary_choice {
            let (old_state, old_si) = (result.injury_context().injury, result.injury_context().serious_injury);
            crate::util::util_server_dialog::UtilServerDialog::show_dialog(game,
                ffb_model::dialog::dialog_parameter_factory::DialogParameter::ApothecaryChoice(
                    ffb_model::dialog::dialog_apothecary_choice_parameter::DialogApothecaryChoiceParameter {
                        player_id: Some(defender_id.clone()),
                        player_state_old: old_state,
                        serious_injury_old: old_si,
                        player_state_new: new_state,
                        serious_injury_new: new_si,
                    }), true);
            return Some(ffb_model::prompts::AgentPrompt::ApothecaryChoice { player_id: defender_id, can_heal: true });
        }
    }
    // Java: no choice -> serious injury cleared, KO -> STUNNED (when the injury type allows), else
    // RESERVE; ReportApothecaryChoice(defender, state, null).
    let cured = if base == PS_KNOCKED_OUT
        && crate::injury::can_apo_ko_into_stun(result.injury_context().injury_type_name.as_deref())
    {
        PlayerState::new(PS_STUNNED)
    } else {
        PlayerState::new(PS_RESERVE)
    };
    let ctx_mut = result.injury_context_mut();
    ctx_mut.injury = Some(cured);
    ctx_mut.serious_injury = None;
    game.report_list.add(ReportApothecaryChoice::new(defender_id, cured, None));
    None
}

/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.mutliblock.StepApothecaryMultiple`.
pub struct StepApothecaryMultiple {
    /// Java: teamId (resolved from init param ACTING_TEAM at start())
    pub team_id: Option<String>,
    /// Java: injuryResults (List<InjuryResult>)
    pub injury_results: Vec<Box<InjuryResult>>,
    /// Java: regenerationFailedResults — populated after regeneration phase.
    pub regeneration_failed_results: Vec<Box<InjuryResult>>,
    /// Java: gettingEvenResults — BB2025: SI players (defender ids) still to get even.
    pub getting_even_results: Vec<String>,
    /// Java: deadResults — BB2025: RIP players (defender ids) whose raise position is still open.
    pub dead_results: Vec<String>,
    /// Java: availableKeyWordsMap -- keyed by the defender id here (Java: the result's index).
    pub available_keywords_map: HashMap<String, Vec<Keyword>>,
    /// Whether ACTING_TEAM init param was set.
    pub acting_team: Option<bool>,
    /// How many apothecaries have been consumed this step.
    pub apos_used: i32,
    /// Java `regenerationFailedResults != null`: the regeneration phase has run; re-executions
    /// after a coach's answer go straight to the apothecary phase.
    regeneration_done: bool,
    /// The casualty whose failed Regeneration is offered a team re-roll (Java
    /// `askForReRollIfAvailable(player, REGENERATION, 4)`), answered by CLIENT_USE_RE_ROLL.
    regen_reroll_pending: Option<String>,
    /// Java bb2025 executeStep reports the regeneration-failed results ("only report cas rolls")
    /// once the regeneration phase is over; later executions find them already reported.
    post_regeneration_reported: bool,
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
            regeneration_done: false,
            regen_reroll_pending: None,
            post_regeneration_reported: false,
        }
    }

    /// Java `remainingApos()`: the team's TurnData apothecaries plus plague doctors.
    fn remaining_apos(&self, game: &Game) -> i32 {
        let td = if self.team_id.as_deref() == Some(game.team_home.id.as_str()) { &game.turn_data_home } else { &game.turn_data_away };
        td.apothecaries + td.plague_doctors
    }

    /// Java `ApothecaryType.forPlayer(game, player, playerState)`.
    fn apothecary_types(game: &Game, player_id: &str, state: Option<PlayerState>) -> Vec<ffb_model::enums::ApothecaryType> {
        use ffb_mechanics::apothecary_mechanic::ApothecaryMechanic as _;
        match (game.player(player_id), state) {
            (Some(defender), Some(ps)) => match game.rules {
                ffb_model::enums::Rules::Bb2016 => ffb_mechanics::bb2016::apothecary_mechanic::ApothecaryMechanic::new()
                    .apothecary_types(game, defender, ps),
                ffb_model::enums::Rules::Bb2020 => ffb_mechanics::bb2020::apothecary_mechanic::ApothecaryMechanic::new()
                    .apothecary_types(game, defender, ps),
                _ => ffb_mechanics::bb2025::apothecary_mechanic::ApothecaryMechanic::new()
                    .apothecary_types(game, defender, ps),
            },
            _ => Vec::new(),
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
        // Java: UtilServerDialog.hideDialog(getGameState())
        // DEVIATION: the step's showDialog calls (DialogUseApothecariesParameter,
        // DialogApothecaryChoiceParameter, DialogReRollRegenerationMultipleParameter,
        // DialogUseInducementParameter, DialogSelectKeywordParameter, DialogSelectPositionParameter)
        // and the hideDialogs on CLIENT_KEYWORD_SELECTION / CLIENT_POSITION_SELECTION are not
        // ported: this Rust step answers every one of those decisions inline (headless) and never
        // waits, so there is no point at which the dialog would be open.
        crate::util::util_server_dialog::UtilServerDialog::hide_dialog(game);

        // Phase 1: Regeneration — for each injury, roll Regeneration if player has the skill.
        // Successful regeneration: remove from injury_results (injury nullified).
        // Failed: move to regeneration_failed_results for apothecary processing.
        let bb2025 = game.rules == ffb_model::enums::Rules::Bb2025;
        let first_pass = !self.regeneration_done;
        self.regeneration_done = true;
        if first_pass {
            // Java bb2025 regenerationHandled (first call): "report injuries without cas rolls" —
            // result.report(this) on every result while still preRegeneration (skip CAS), then
            // setAlreadyReported(false) so the post-regeneration pass reports the cas part.
            // Missing, a multi-block's knockdowns wrote no injury reports at all (Grim Ironjaw,
            // dwarf vs slann_fumbbl bb2025 seed 1).
            if bb2025 {
                for r in self.injury_results.iter_mut() {
                    r.report(game);
                    r.set_already_reported(false);
                }
            } else {
                // Java bb2020/multiblock/StepApothecaryMultiple.executeStep groups by status and
                // reports the DO_REQUEST group (`injuryResult.report(this)` before the apothecary
                // question), then the NO_APOTHECARY group — both ahead of the regeneration roll
                // (halfling vs necromantic bb2020 seed 2: Grim Ironjaw's two POWs).
                for r in self.injury_results.iter_mut() {
                    if r.injury_context().apothecary_status == ApothecaryStatus::DoRequest {
                        r.report(game);
                    }
                }
                // With an apothecary left Java shows DialogUseApothecariesParameter and returns;
                // the declined answer re-runs executeStep, whose DO_NOT_USE_APOTHECARY group adds
                // `ReportApothecaryRoll(defender, null, null, null, null, casualtyModifiers)` before
                // the NO_APOTHECARY reports (Grim Ironjaw vs a halfling team with an apothecary,
                // old_world_alliance_ogre vs halfling bb2020 seed 2). Headless declines.
                let apos_left = self.team_id.as_ref().map(|t| {
                    let total = if *t == game.team_home.id { game.team_home.apothecaries } else { game.team_away.apothecaries };
                    total - self.apos_used > 0
                }).unwrap_or(false);
                if apos_left {
                    let declined: Vec<(Option<String>, Vec<String>)> = self.injury_results.iter()
                        .filter(|r| r.injury_context().apothecary_status == ApothecaryStatus::DoRequest)
                        .map(|r| (r.injury_context().defender_id.clone(),
                                  r.injury_context().casualty_modifiers.iter().map(|m| m.name.to_string()).collect()))
                        .collect();
                    for (defender, mods) in declined {
                        game.report_list.add(ReportApothecaryRoll::new(defender, vec![], None, None, None, mods));
                    }
                }
                for r in self.injury_results.iter_mut() {
                    if r.injury_context().apothecary_status == ApothecaryStatus::NoApothecary {
                        r.report(game);
                    }
                }
            }
            let mut results_to_keep = Vec::new();
            let drained: Vec<Box<InjuryResult>> = self.injury_results.drain(..).collect();
            for r in drained {
                let player_id = r.injury_context().defender_id.clone();
                if bb2025 {
                    // Java bb2025 regenerationHandled: results are grouped by
                    // `canRollToSaveFromInjury && injuryType.canUseApo()`; the others pass
                    // regeneration untouched. A casualty that regenerates is cured; one that fails
                    // with a re-roll (Regeneration inducement or team re-roll) left stays
                    // PRE-regeneration for the re-roll question below; any other passes.
                    let mut r = r;
                    let can_regen = player_id.as_deref().and_then(|id| game.player(id))
                        .map(|p| p.has_skill_property(ffb_model::model::property::named_properties::NamedProperties::CAN_ROLL_TO_SAVE_FROM_INJURY))
                        .unwrap_or(false) && r.injury_context().can_use_apo;
                    let ctx_state = r.injury_context().injury;
                    let is_casualty = ctx_state.map(|s| s.is_casualty()).unwrap_or(false);
                    match (player_id.as_deref(), ctx_state) {
                        (Some(pid), Some(state)) if can_regen && is_casualty => {
                            if crate::step::util_server_injury::handle_regeneration_with_state(game, rng, pid, state, false).0 {
                                continue; // regenerated: the injury is nullified
                            }
                            if !self.regeneration_re_rolls_available(game, pid) {
                                r.passed_regeneration();
                            }
                        }
                        _ => r.passed_regeneration(),
                    }
                    results_to_keep.push(r);
                    continue;
                }
                // Java `regenerationHandled`: `injuryContext.isCasualty() && handleRegeneration(this,
                // player, injuryContext.getPlayerState())` -- the CONTEXT's state; the injury has
                // not been applied to the field yet. Rust read the field state (not a casualty
                // yet), so a multi-block casualty never rolled Regeneration (star tier: halfling
                // vs necromantic bb2020, Grim Ironjaw's Multiple Block).
                let ctx_state = r.injury_context().injury;
                let regen_rolled = match (player_id.as_ref(), ctx_state) {
                    (Some(pid), Some(state)) => crate::step::util_server_injury::handle_regeneration_with_state(
                        game, rng, pid, state, false).0,
                    _ => false,
                };
                if !regen_rolled {
                    // Java: every result that is not regenerated calls passedRegeneration().
                    let mut r = r;
                    r.passed_regeneration();
                    results_to_keep.push(r);
                }
                // regen succeeded → injury nullified, not added back
            }
            self.regeneration_failed_results = results_to_keep;
        }

        // Work from regeneration_failed_results for apothecary processing.
        // (injury_results is now empty; swap to regeneration_failed_results for phase 2+)
        if self.injury_results.is_empty() {
            std::mem::swap(&mut self.injury_results, &mut self.regeneration_failed_results);
        }

        // Java bb2025 `if (!regenerationHandled()) { CONTINUE; return; }`: a failed Regeneration
        // that can still be re-rolled is put to the coach first.
        if bb2025 {
            if let Some(out) = self.ask_for_regeneration_re_rolls(game) {
                return out;
            }
        }

        // Java bb2025 executeStep: `regenerationFailedResults.forEach(result -> result.report(this))`
        // — "only report cas rolls" (preRegeneration is now false: EVERYTHING_BUT_CAS).
        if bb2025 && !self.post_regeneration_reported {
            self.post_regeneration_reported = true;
            for r in self.injury_results.iter_mut() {
                r.report(game);
            }
        }

        // Java executeStep, DO_REQUEST group: every casualty the team could still patch up is put
        // to the coach in ONE DialogUseApothecariesParameter (WAIT_FOR_APOTHECARY_USE); the others
        // get no apothecary. Remaining apothecaries are the team's TurnData ones (+ plague doctors).
        let do_request: Vec<usize> = (0..self.injury_results.len())
            .filter(|&i| self.injury_results[i].injury_context().apothecary_status == ApothecaryStatus::DoRequest)
            .collect();
        if !do_request.is_empty() {
            let remaining_apos = self.remaining_apos(game);
            let mut descriptions: Vec<serde_json::Value> = Vec::new();
            let mut player_ids = Vec::new();
            for i in do_request {
                let (defender_id, state, si) = {
                    let ctx = self.injury_results[i].injury_context();
                    (ctx.defender_id.clone().unwrap_or_default(), ctx.injury, ctx.serious_injury)
                };
                let types = Self::apothecary_types(game, &defender_id, state);
                let ctx = self.injury_results[i].injury_context_mut();
                if remaining_apos > 0 && !types.is_empty() {
                    ctx.apothecary_status = ApothecaryStatus::WaitForApothecaryUse;
                    descriptions.push(serde_json::json!({
                        "apothecaryTypes": types.iter().map(|t| t.name()).collect::<Vec<_>>(),
                        "playerId": defender_id,
                        "playerState": state.map(|s| s.0 as i64),
                        "seriousInjury": ffb_model::factory::serious_injury_factory::kind_to_json(game.rules, si)
                            .unwrap_or(serde_json::Value::Null),
                    }));
                    player_ids.push(defender_id);
                } else {
                    ctx.apothecary_status = ApothecaryStatus::DoNotUseApothecary;
                }
            }
            if !descriptions.is_empty() {
                let team_id = self.team_id.clone().unwrap_or_default();
                crate::util::util_server_dialog::UtilServerDialog::show_dialog(game,
                    ffb_model::dialog::dialog_parameter_factory::DialogParameter::UseApothecaries(
                        ffb_model::dialog::dialog_use_apothecaries_parameter::DialogUseApothecariesParameter {
                            team_id: Some(team_id.clone()),
                            injury_descriptions: descriptions,
                        }), true);
                return StepOutcome::cont().with_prompt(ffb_model::prompts::AgentPrompt::UseApothecaries { team_id, player_ids });
            }
        }

        // Java USE_APOTHECARY group: rollApothecary each; a new casualty the coach may prefer waits
        // for DialogApothecaryChoiceParameter (WAIT_FOR_APOTHECARY_CHOICE), otherwise RESULT_CHOICE.
        for i in 0..self.injury_results.len() {
            if self.injury_results[i].injury_context().apothecary_status != ApothecaryStatus::UseApothecary {
                continue;
            }
            if let Some(prompt) = roll_apothecary_for_injury(&mut self.injury_results[i], rng, game) {
                self.injury_results[i].injury_context_mut().apothecary_status = ApothecaryStatus::WaitForApothecaryChoice;
                return StepOutcome::cont().with_prompt(prompt);
            }
            self.injury_results[i].injury_context_mut().apothecary_status = ApothecaryStatus::ResultChoice;
        }
        if self.injury_results.iter().any(|r| r.injury_context().apothecary_status == ApothecaryStatus::WaitForApothecaryChoice) {
            return StepOutcome::cont();
        }

        // Step 6: apply the injuries (Java applies every result; an apothecary's RESULT_CHOICE
        // carries the chosen state).
        let applied = |status: ApothecaryStatus| status == ApothecaryStatus::NoApothecary
            || status == ApothecaryStatus::DoNotUseApothecary
            || status == ApothecaryStatus::ResultChoice;
        for r in &self.injury_results {
            if applied(r.injury_context().apothecary_status) {
                r.apply_to_with_stats(game, false);
            }
        }
        // Java walks `regenerationFailedResults` for Getting Even and Raise Dead after applying.
        let final_contexts: Vec<crate::injury::InjuryContext> = self.injury_results.iter()
            .map(|r| r.injury_context().clone()).collect();
        self.injury_results.retain(|r| !applied(r.injury_context().apothecary_status));

        // Java: double-attacker-down: two injury results for same player + regeneration failed.
        {
            let double_attacker = self.injury_results.len() == 2
                && {
                    let id0 = self.injury_results[0].injury_context().defender_id.clone();
                    let id1 = self.injury_results[1].injury_context().defender_id.clone();
                    id0.is_some() && id0 == id1
                }
                && !self.regeneration_failed_results.is_empty();

            if double_attacker {
                // Java (bb2025): playerState.changeBase(PlayerState.PRONE) — note this differs
                // from the bb2020 sibling which uses PlayerState.RESERVE here.
                let player_id = self.injury_results[0].injury_context().defender_id.clone().unwrap_or_default();
                if let Some(ps) = game.field_model.player_state(&player_id) {
                    game.field_model.set_player_state(&player_id, ps.change_base(PS_PRONE));
                }
                {
                    let is_home = game.team_home.players.iter().any(|p| p.id == player_id);
                    let tr = game.game_result.team_result_mut(is_home);
                    let pr = tr.player_result_mut(&player_id);
                    pr.serious_injury = None;
                    pr.serious_injury_decay = None;
                }
                // Re-apply only regeneration-failed injury results.
                for r in &self.injury_results {
                    let is_regen_failed = self.regeneration_failed_results.iter()
                        .any(|rf| rf.injury_context().defender_id == r.injury_context().defender_id
                            && rf.injury_context().injury == r.injury_context().injury);
                    if is_regen_failed {
                        r.apply_to_with_stats(game, false);
                    }
                }
            }
        }

        // Java: for each regeneration-failed result, check handlePumpUp (BB2025 Pump Up the Crowd).
        {
            let mechanic = crate::mechanic::state_mechanic_for(game.rules);
            let regen_ids: Vec<_> = self.regeneration_failed_results.iter()
                .map(|r| r.injury_context().clone())
                .collect();
            for ctx in &regen_ids {
                if mechanic.handle_pump_up(game, ctx) {
                    let _team_id = ctx.defender_id.as_deref().unwrap_or("");
                }
            }
        }

        if !bb2025 {
            return StepOutcome::next();
        }

        // Java: an SI caused by an attacker whose position carries keywords the defender can get
        // even with joins gettingEvenResults (keywords per result in availableKeyWordsMap).
        use crate::step::bb2025::shared::step_apothecary as apo;
        for ctx in &final_contexts {
            let (Some(defender_id), Some(attacker_id)) = (ctx.defender_id.as_deref(), ctx.attacker_id.as_deref()) else { continue };
            if !ctx.injury.map(|s| s.is_si()).unwrap_or(false) {
                continue;
            }
            let keywords = apo::getting_even_keywords(game, attacker_id, defender_id);
            if !keywords.is_empty() {
                self.getting_even_results.push(defender_id.to_string());
                self.available_keywords_map.insert(defender_id.to_string(), keywords);
            }
        }
        // Java: a dead player gets no Getting Even; the first one per raising team that the team
        // can raise joins deadResults.
        let mut raising_teams: Vec<String> = Vec::new();
        for ctx in &final_contexts {
            let Some(defender_id) = ctx.defender_id.as_deref() else { continue };
            if ctx.injury.map(|s| s.base()) != Some(PS_RIP) {
                continue;
            }
            self.getting_even_results.retain(|id| !id.eq_ignore_ascii_case(defender_id));
            if let Some((raising_team_id, _)) = apo::raise_dead_positions(game, defender_id, ctx.attacker_id.as_deref()) {
                if !raising_teams.contains(&raising_team_id) {
                    self.dead_results.push(defender_id.to_string());
                    raising_teams.push(raising_team_id);
                }
            }
        }
        // Java: a single keyword needs no question -- push GETTING_EVEN for it right away.
        let mut out = StepOutcome::next();
        for defender_id in self.getting_even_results.clone() {
            let keywords = self.available_keywords_map.get(&defender_id).cloned().unwrap_or_default();
            if keywords.len() == 1 {
                out = out.push_seq(apo::getting_even_sequence(&defender_id, keywords[0]));
                self.getting_even_results.retain(|id| *id != defender_id);
            }
        }
        if self.dead_results.is_empty() {
            return self.check_getting_even(game, out);
        }
        // Java: a raising team with fewer than two positions needs no question (one: raised now).
        for defender_id in self.dead_results.clone() {
            let positions = Self::raise_positions_against(game, &defender_id);
            if positions.len() < 2 {
                self.dead_results.retain(|id| *id != defender_id);
                if positions.len() == 1 {
                    for ev in apo::raise_dead_player(game, &defender_id, &positions[0]) { out = out.with_event(ev); }
                }
            }
        }
        self.check_raise_dead(game, out)
    }

    /// Java `injuryMechanic.raisePositions(game.getOtherTeam(defender.getTeam()))`.
    fn raise_positions_against(game: &Game, defender_id: &str) -> Vec<ffb_model::model::roster_position::RosterPosition> {
        let raising_is_home = !game.team_home.has_player(defender_id);
        let raising_team = if raising_is_home { &game.team_home } else { &game.team_away };
        crate::step::util_server_injury::injury_mechanic_for(game.rules).raise_positions(raising_team, game.rules)
    }

    /// Java `checkGettingEven()`: NEXT_STEP when no keyword question is left, else the
    /// SELECT_KEYWORD dialog for the first remaining player (CONTINUE).
    fn check_getting_even(&mut self, game: &mut Game, out: StepOutcome) -> StepOutcome {
        let Some(defender_id) = self.getting_even_results.first().cloned() else { return out };
        let keywords = self.available_keywords_map.get(&defender_id).cloned().unwrap_or_default();
        let prompt = crate::step::bb2025::shared::step_apothecary::show_getting_even_dialog(game, &defender_id, &keywords);
        Self::waiting(out, prompt)
    }

    /// Java `checkRaiseDead()`: Getting Even when no position question is left, else the
    /// RAISE_DEAD SELECT_POSITION dialog for the first dead player's raising team (CONTINUE).
    fn check_raise_dead(&mut self, game: &mut Game, out: StepOutcome) -> StepOutcome {
        let Some(defender_id) = self.dead_results.first().cloned() else { return self.check_getting_even(game, out) };
        let raising_is_home = !game.team_home.has_player(&defender_id);
        let raising_team_id = if raising_is_home { game.team_home.id.clone() } else { game.team_away.id.clone() };
        let positions = Self::raise_positions_against(game, &defender_id);
        let prompt = crate::step::bb2025::shared::step_apothecary::show_raise_dead_dialog(game, raising_team_id, &positions);
        Self::waiting(out, prompt)
    }

    /// `out` (its pushes and events kept) turned into a CONTINUE waiting on `prompt`.
    fn waiting(out: StepOutcome, prompt: ffb_model::prompts::AgentPrompt) -> StepOutcome {
        let mut waiting = StepOutcome::cont().with_prompt(prompt);
        waiting.pushes = out.pushes;
        waiting.events = out.events;
        waiting
    }

    /// The TurnData of the step's team (Java `getTurnData()`).
    fn turn_data<'a>(&self, game: &'a mut Game) -> &'a mut ffb_model::model::turn_data::TurnData {
        if self.team_id.as_deref() == Some(game.team_home.id.as_str()) { &mut game.turn_data_home } else { &mut game.turn_data_away }
    }

    /// Java `regenerationTypes(inducementSet)`: the Regeneration inducements with uses left, by
    /// InducementType priority (Mortuary Assistants 1, Plague Doctors -- which also count as an
    /// apothecary -- 2).
    fn regeneration_inducement_types(&self, game: &Game) -> Vec<String> {
        let td = if self.team_id.as_deref() == Some(game.team_home.id.as_str()) { &game.turn_data_home } else { &game.turn_data_away };
        let mut types: Vec<(i32, String)> = td.inducement_set.get_inducements().into_iter()
            .filter(|i| i.has_usage(ffb_model::inducement::usage::Usage::REGENERATION) && i.get_uses_left() > 0)
            .map(|i| (if i.has_usage(ffb_model::inducement::usage::Usage::APOTHECARY_JOURNEYMEN) { 2 } else { 1 }, i.type_id.clone()))
            .collect();
        types.sort();
        types.into_iter().map(|(_, t)| t).collect()
    }

    /// Java `regenerationReRollsAvailable(player)`: a Regeneration inducement use left, or a
    /// team re-roll the player may use.
    fn regeneration_re_rolls_available(&self, game: &Game, player_id: &str) -> bool {
        !self.regeneration_inducement_types(game).is_empty()
            || game.player(player_id)
                .map(|p| crate::util::util_server_re_roll::UtilServerReRoll::is_team_re_roll_available(game, p))
                .unwrap_or(false)
    }

    /// Java bb2025 `regenerationHandled()` after the first regeneration pass: the failed
    /// regenerations that can still be re-rolled are asked about -- one player with an inducement
    /// gets DialogUseInducementParameter, one without gets the team re-roll offer
    /// (`askForReRollIfAvailable(player, REGENERATION, 4)`), several get
    /// DialogReRollRegenerationMultipleParameter. `None` when nothing is left to ask.
    fn ask_for_regeneration_re_rolls(&mut self, game: &mut Game) -> Option<StepOutcome> {
        use ffb_model::dialog::dialog_parameter_factory::DialogParameter;
        let fails: Vec<String> = self.injury_results.iter()
            .filter(|r| r.is_pre_regeneration())
            .filter_map(|r| r.injury_context().defender_id.clone())
            .filter(|pid| self.regeneration_re_rolls_available(game, pid))
            .collect();
        let team_id = self.team_id.clone().unwrap_or_default();
        let team_is_home = team_id == game.team_home.id;
        let opponent_asked = team_is_home != game.home_playing;
        let inducement = self.regeneration_inducement_types(game).into_iter().next();
        match fails.len() {
            0 => None,
            1 => {
                let pid = fails[0].clone();
                if let Some(inducement_type) = inducement {
                    crate::util::util_server_dialog::UtilServerDialog::show_dialog(game, DialogParameter::UseInducement(
                        ffb_model::dialog::dialog_use_inducement_parameter::DialogUseInducementParameter {
                            team_id: Some(team_id.clone()),
                            inducement_types: vec![Some(inducement_type.clone())],
                            cards: vec![],
                            player_id: Some(pid.clone()),
                        }), opponent_asked);
                    return Some(StepOutcome::cont().with_prompt(ffb_model::prompts::AgentPrompt::RegenerationReRolls {
                        team_id, player_ids: vec![pid], inducement_type: Some(inducement_type),
                    }));
                }
                match crate::step::util_server_re_roll::ask_for_reroll_if_available_for(game, Some(&pid), "REGENERATION", 4, false) {
                    Some(mut prompt) => {
                        // Java StepApothecaryMultiple extends AbstractStep: no CLIENT_USE_SKILL arm,
                        // so only the team re-roll can answer (as in StepApothecary).
                        if let ffb_model::prompts::AgentPrompt::ReRollOffer { re_roll_skill, .. } = &mut prompt {
                            *re_roll_skill = None;
                        }
                        self.regen_reroll_pending = Some(pid);
                        Some(StepOutcome::cont().with_prompt(prompt))
                    }
                    None => {
                        for r in self.injury_results.iter_mut() {
                            if r.injury_context().defender_id.as_deref() == Some(pid.as_str()) {
                                r.passed_regeneration();
                            }
                        }
                        None
                    }
                }
            }
            _ => {
                crate::util::util_server_dialog::UtilServerDialog::show_dialog(game, DialogParameter::ReRollRegenerationMultiple(
                    ffb_model::dialog::dialog_re_roll_regeneration_multiple_parameter::DialogReRollRegenerationMultipleParameter {
                        player_ids: fails.clone(),
                        inducement_type: inducement.clone(),
                        re_roll_options: vec![],
                    }), opponent_asked);
                Some(StepOutcome::cont().with_prompt(ffb_model::prompts::AgentPrompt::RegenerationReRolls {
                    team_id, player_ids: fails, inducement_type: inducement,
                }))
            }
        }
    }

    /// Java: the re-rolled Regeneration of `player_id` (CLIENT_USE_INDUCEMENT / CLIENT_USE_RE_ROLL):
    /// a success cures the casualty (the result leaves the list); either way it has passed
    /// regeneration.
    fn re_roll_regeneration(&mut self, game: &mut Game, rng: &mut GameRng, player_id: &str, do_roll: bool) {
        let Some(i) = self.injury_results.iter().position(|r| r.injury_context().defender_id.as_deref() == Some(player_id)) else { return };
        if do_roll {
            let state = self.injury_results[i].injury_context().injury.unwrap_or_default();
            if crate::step::util_server_injury::handle_regeneration_with_state(game, rng, player_id, state, true).0 {
                self.injury_results.remove(i);
                return;
            }
        }
        self.injury_results[i].passed_regeneration();
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
            regeneration_done: false,
            regen_reroll_pending: None,
            post_regeneration_reported: false,
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
            // A declined apothecary-choice dialog (the agents' answer) keeps the original injury.
            Action::UseApothecary { player_id, use_apothecary: false }
                if self.injury_results.iter().any(|r| r.injury_context().apothecary_status == ApothecaryStatus::WaitForApothecaryChoice
                    && r.injury_context().defender_id.as_deref() == Some(player_id.as_str())) =>
            {
                for r in &mut self.injury_results {
                    if r.injury_context().apothecary_status == ApothecaryStatus::WaitForApothecaryChoice
                        && r.injury_context().defender_id.as_deref() == Some(player_id.as_str())
                    {
                        r.injury_context_mut().apothecary_status = ApothecaryStatus::ResultChoice;
                    }
                }
            }
            Action::UseApothecary { player_id, use_apothecary } => {
                // Java CLIENT_USE_APOTHECARY: the named casualty uses an apothecary (when one is left:
                // useApo) or not; every other casualty still waiting is asked again (DO_REQUEST).
                let remaining = self.remaining_apos(game);
                let team_home = self.team_id.as_deref() == Some(game.team_home.id.as_str());
                let mut used = false;
                for r in &mut self.injury_results {
                    if r.injury_context().apothecary_status == ApothecaryStatus::WaitForApothecaryUse
                        && r.injury_context().defender_id.as_deref() == Some(player_id.as_str())
                        && !used
                    {
                        let new_status = if *use_apothecary && remaining > 0 {
                            used = true;
                            ApothecaryStatus::UseApothecary
                        } else {
                            ApothecaryStatus::DoNotUseApothecary
                        };
                        r.injury_context_mut().apothecary_status = new_status;
                    }
                }
                if used {
                    let td = if team_home { &mut game.turn_data_home } else { &mut game.turn_data_away };
                    td.use_apothecary(ffb_model::enums::ApothecaryType::Team);
                    self.apos_used += 1;
                }
                for r in &mut self.injury_results {
                    if r.injury_context().apothecary_status == ApothecaryStatus::WaitForApothecaryUse {
                        r.injury_context_mut().apothecary_status = ApothecaryStatus::DoRequest;
                    }
                }
            }
            // Java CLIENT_APOTHECARY_CHOICE: the chosen state and serious injury (RESULT_CHOICE).
            Action::ApothecaryChoice { player_state, serious_injury } => {
                let si: Option<ffb_model::enums::SeriousInjuryKind> = serious_injury.as_deref()
                    .and_then(|s| serde_json::from_str(&format!("\"{s}\"")).ok());
                if let Some(r) = self.injury_results.iter_mut()
                    .find(|r| r.injury_context().apothecary_status == ApothecaryStatus::WaitForApothecaryChoice)
                {
                    let ctx = r.injury_context_mut();
                    ctx.injury = Some(PlayerState::new(*player_state));
                    ctx.serious_injury = si;
                    ctx.apothecary_status = ApothecaryStatus::ResultChoice;
                }
            }
            Action::Acknowledge => {
                // Java: CLIENT_USE_APOTHECARIES → all WAIT_FOR_APOTHECARY_USE → DO_NOT_USE_APOTHECARY.
                for r in &mut self.injury_results {
                    if r.injury_context().apothecary_status == ApothecaryStatus::WaitForApothecaryUse {
                        r.injury_context_mut().apothecary_status = ApothecaryStatus::DoNotUseApothecary;
                    }
                }
            }
            // Java CLIENT_USE_RE_ROLL (REGENERATION): an accepted team re-roll is spent and the
            // Regeneration rolled again; either way the player has passed regeneration.
            Action::UseReRoll { use_reroll } => {
                if let Some(pid) = self.regen_reroll_pending.take() {
                    let do_roll = *use_reroll
                        && crate::step::util_server_re_roll::use_reroll(game, &ffb_model::enums::ReRollSource::new("TRR"), &pid, rng);
                    self.re_roll_regeneration(game, rng, &pid, do_roll);
                }
            }
            // Java CLIENT_USE_INDUCEMENT: the chosen player's Regeneration is re-rolled with the
            // inducement (a Plague Doctor also counts as an apothecary used) or, without one, a
            // team re-roll; no player declines every open re-roll.
            Action::UseInducement { inducement_type, player_ids, .. } => {
                match player_ids.first() {
                    Some(pid) if self.injury_results.iter().any(|r| r.injury_context().defender_id.as_deref() == Some(pid.as_str())) => {
                        let pid = pid.clone();
                        let do_roll = match inducement_type {
                            Some(t) => {
                                let used = self.turn_data(game).inducement_set.use_one_of(t);
                                if used {
                                    let journeymen = self.turn_data(game).inducement_set.get(t)
                                        .map(|i| i.has_usage(ffb_model::inducement::usage::Usage::APOTHECARY_JOURNEYMEN))
                                        .unwrap_or(false);
                                    let player_home = game.team_home.has_player(&pid);
                                    if journeymen {
                                        let td = if player_home { &mut game.turn_data_home } else { &mut game.turn_data_away };
                                        td.use_apothecary(ffb_model::enums::ApothecaryType::Plague);
                                    }
                                    let team_id = if player_home { game.team_home.id.clone() } else { game.team_away.id.clone() };
                                    game.report_list.add(ffb_model::report::report_inducement::ReportInducement::new(team_id, t.clone(), 1));
                                }
                                used
                            }
                            None => crate::step::util_server_re_roll::use_reroll(game, &ffb_model::enums::ReRollSource::new("TRR"), &pid, rng),
                        };
                        self.re_roll_regeneration(game, rng, &pid, do_roll);
                    }
                    _ => {
                        for r in self.injury_results.iter_mut() {
                            r.passed_regeneration();
                        }
                    }
                }
            }
            // Java CLIENT_KEYWORD_SELECTION: hideDialog, one GETTING_EVEN per chosen keyword
            // (pushed in reverse), then the next Getting Even question or NEXT_STEP.
            Action::KeywordSelection { player_id, keywords }
                if self.getting_even_results.first() == Some(player_id) =>
            {
                crate::util::util_server_dialog::UtilServerDialog::hide_dialog(game);
                let mut out = StepOutcome::next();
                for keyword in keywords.iter().rev() {
                    out = out.push_seq(crate::step::bb2025::shared::step_apothecary::getting_even_sequence(player_id, Keyword::for_name(keyword)));
                }
                self.getting_even_results.remove(0);
                return self.check_getting_even(game, out);
            }
            // Java CLIENT_POSITION_SELECTION: hideDialog, raise the first dead player as the chosen
            // position, then the next Raise Dead question (or Getting Even).
            Action::PositionSelection { positions, .. } if !self.dead_results.is_empty() => {
                crate::util::util_server_dialog::UtilServerDialog::hide_dialog(game);
                let defender_id = self.dead_results.remove(0);
                let mut out = StepOutcome::next();
                if let Some(position_id) = positions.first() {
                    for ev in crate::step::bb2025::shared::step_apothecary::raise_dead_player_as(game, &defender_id, position_id) {
                        out = out.with_event(ev);
                    }
                }
                return self.check_raise_dead(game, out);
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
    fn acting_team_resolution_applies_attacker_ko() {
        // dark_elf bb2020 seed 53 i=8: the driver constructed this step with
        // new(String::new()), pre-setting team_id to Some("") — the acting-team → team-id
        // resolution (Java: init from ACTING_TEAM) is guarded on team_id.is_none() and never
        // ran, so the retain filter compared "" against real team ids and dropped the
        // multiple-block attacker's KO. Default-constructed (team_id None) + ActingTeam(true)
        // must resolve to the acting team and apply the KO.
        use ffb_model::enums::{PlayerState, PS_KNOCKED_OUT, PS_PRONE, PS_STANDING};
        let mut game = make_game();
        game.team_home.players.push(ffb_model::model::player::Player {
            id: "home_02".into(), name: "home_02".into(), nr: 2,
            position_id: "lineman".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            ..Default::default()
        });
        game.home_playing = true;
        game.field_model.set_player_coordinate("home_02", FieldCoordinate::new(12, 6));
        game.field_model.set_player_state("home_02", PlayerState::new(PS_PRONE));
        let _ = PS_STANDING;
        let mut step = StepApothecaryMultiple::default();
        step.set_parameter(&StepParameter::ActingTeam(true));
        let mut ir = make_injury("home_02", ApothecaryMode::Attacker, ApothecaryStatus::NoApothecary);
        ir.injury_context_mut().injury = Some(PlayerState::new(PS_KNOCKED_OUT));
        step.set_parameter(&StepParameter::InjuryResult(ir));
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.team_id.as_deref(), Some("home"),
            "ActingTeam(true) with home acting must resolve team_id to the home team");
        let state = game.field_model.player_state("home_02").expect("state");
        assert_eq!(state.base(), PS_KNOCKED_OUT,
            "the acting-team apothecary step must apply the attacker's KO");
    }

    /// Java bb2025 StepApothecaryMultiple: regenerationHandled reports every result pre-regen
    /// (skipInjuryParts CAS), then executeStep reports the regeneration-failed ones again with
    /// EVERYTHING_BUT_CAS — for two targets: CAS, CAS, EVERYTHING_BUT_CAS, EVERYTHING_BUT_CAS
    /// (Grim Ironjaw's Multiple Block, dwarf vs slann_fumbbl bb2025 seed 1).
    #[test]
    fn bb2025_reports_each_injury_pre_and_post_regeneration() {
        use ffb_model::enums::{PlayerState, PS_PRONE, PS_STANDING};
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        for id in ["away_04", "away_05"] {
            game.team_away.players.push(ffb_model::model::player::Player {
                id: id.into(), name: id.into(), nr: 4, position_id: "lineman".into(),
                movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
                ..Default::default()
            });
            game.field_model.set_player_coordinate(id, FieldCoordinate::new(12, 6));
            game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
        }
        game.home_playing = true;
        let mut step = StepApothecaryMultiple::default();
        step.set_parameter(&StepParameter::ActingTeam(false));
        for id in ["away_05", "away_04"] {
            let mut ir = make_injury(id, ApothecaryMode::Defender, ApothecaryStatus::NoApothecary);
            ir.injury_context_mut().injury = Some(PlayerState::new(PS_PRONE));
            step.set_parameter(&StepParameter::InjuryResult(ir));
        }
        step.start(&mut game, &mut GameRng::new(0));
        let skips: Vec<String> = game.report_list.get_reports().iter()
            .filter(|r| r.get_id() == ReportId::INJURY)
            .filter_map(|r| r.to_json())
            .map(|j| j["skipInjuryParts"].as_str().unwrap_or("").to_string())
            .collect();
        assert_eq!(skips, vec!["CAS", "CAS", "EVERYTHING_BUT_CAS", "EVERYTHING_BUT_CAS"]);
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
            is_big_guy: false,
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
            is_big_guy: false,
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

    #[test]
    fn double_attacker_down_resets_state_to_prone_not_reserve() {
        // Java StepApothecaryMultiple (bb2025), doubleAttackerDown branch:
        //   game.getFieldModel().setPlayerState(player, playerState.changeBase(PlayerState.PRONE));
        // (the bb2020 sibling uses PlayerState.RESERVE here instead — bb2025 differs).
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        use std::collections::HashSet;

        let mut game = make_game();
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
        game.field_model.set_player_state("home_1", PlayerState::new(PS_STUNNED));

        let mut step = StepApothecaryMultiple::default();
        step.team_id = Some("home".to_string());
        // Two injury results for the same defender, with a status untouched by the
        // DoRequest/UseApothecary phases so both survive the apply/retain step.
        step.injury_results.push(make_injury("home_1", ApothecaryMode::HitPlayer, ApothecaryStatus::WaitForApothecaryUse));
        step.injury_results.push(make_injury("home_1", ApothecaryMode::HitPlayer, ApothecaryStatus::WaitForApothecaryUse));
        // The regeneration phase has run (re-execution after a coach's answer).
        step.regeneration_done = true;
        // Non-empty regeneration_failed_results to enable the doubleAttackerDown branch
        // and skip the phase-1 regeneration filtering.
        step.regeneration_failed_results.push(
            make_injury("home_1", ApothecaryMode::HitPlayer, ApothecaryStatus::ResultChoice)
        );

        step.start(&mut game, &mut GameRng::new(0));

        let final_state = game.field_model.player_state("home_1").expect("player state set");
        assert_eq!(final_state.base(), PS_PRONE, "bb2025 double-attacker-down must reset to PRONE, not RESERVE");
    }

    /// Java DO_REQUEST group: a casualty the team could patch up is put to the coach in one
    /// DialogUseApothecariesParameter; declining them all (CLIENT_USE_APOTHECARIES) applies it.
    #[test]
    fn multi_block_casualties_ask_for_the_apothecary() {
        use ffb_model::enums::{PlayerState, PS_STANDING, PS_SERIOUS_INJURY};
        let mut game = make_game();
        game.team_home.players.push(ffb_model::model::player::Player {
            id: "home_03".into(), name: "home_03".into(), nr: 3, position_id: "lineman".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            ..Default::default()
        });
        game.team_home.apothecaries = 1;
        game.turn_data_home.apothecaries = 1;
        game.field_model.set_player_coordinate("home_03", FieldCoordinate::new(5, 7));
        game.field_model.set_player_state("home_03", PlayerState::new(PS_STANDING));
        let mut step = StepApothecaryMultiple::default();
        step.team_id = Some("home".to_string());
        let mut ir = make_injury("home_03", ApothecaryMode::Defender, ApothecaryStatus::DoRequest);
        ir.injury_context_mut().injury = Some(PlayerState::new(PS_SERIOUS_INJURY));
        step.injury_results.push(ir);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::UseApothecaries { ref player_ids, .. })
            if player_ids == &vec!["home_03".to_string()]), "{:?}", out.prompt);
        assert!(matches!(game.dialog_parameter,
            Some(ffb_model::dialog::dialog_parameter_factory::DialogParameter::UseApothecaries(_))));
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.turn_data_home.apothecaries, 1, "declined: none used");
        assert_eq!(game.field_model.player_state("home_03").unwrap().base(), PS_SERIOUS_INJURY);
    }

    fn regenerating_home_player(game: &mut Game, id: &str, x: i32) {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        game.team_home.players.push(ffb_model::model::player::Player {
            id: id.into(), name: id.into(), nr: x, position_id: "lineman".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![ffb_model::model::skill_def::SkillWithValue::new(ffb_model::enums::SkillId::Regeneration)],
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(x, 7));
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    fn two_regenerating_casualties(seed: u64) -> (Game, StepApothecaryMultiple, StepOutcome, GameRng) {
        use ffb_model::enums::{PlayerState, PS_SERIOUS_INJURY};
        use ffb_model::inducement::{inducement::Inducement, usage::Usage};
        let mut game = make_game();
        regenerating_home_player(&mut game, "home_03", 3);
        regenerating_home_player(&mut game, "home_04", 4);
        game.turn_data_home.inducement_set.add_inducement(Inducement::new("mortuaryAssistant", 1, vec![Usage::REGENERATION]));
        let mut step = StepApothecaryMultiple::default();
        step.team_id = Some("home".to_string());
        for id in ["home_03", "home_04"] {
            let mut ir = make_injury(id, ApothecaryMode::Defender, ApothecaryStatus::NoApothecary);
            ir.injury_context_mut().injury = Some(PlayerState::new(PS_SERIOUS_INJURY));
            step.injury_results.push(ir);
        }
        let mut rng = GameRng::new(seed);
        let out = step.start(&mut game, &mut rng);
        (game, step, out, rng)
    }

    /// Java bb2025 StepApothecaryMultiple.regenerationHandled: SEVERAL failed Regenerations that a
    /// Regeneration inducement could re-roll -> one DialogReRollRegenerationMultipleParameter; no
    /// player chosen (CLIENT_USE_INDUCEMENT without players) lets every failure stand.
    #[test]
    fn several_failed_regenerations_are_asked_together() {
        let (mut game, mut step, out, mut rng) = (0..200u64)
            .map(two_regenerating_casualties)
            .find(|(_, _, out, _)| matches!(out.prompt,
                Some(ffb_model::prompts::AgentPrompt::RegenerationReRolls { ref player_ids, .. }) if player_ids.len() == 2))
            .expect("a seed where both Regenerations fail");
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::RegenerationReRolls { ref inducement_type, .. })
            if inducement_type.as_deref() == Some("mortuaryAssistant")));
        assert!(matches!(game.dialog_parameter,
            Some(ffb_model::dialog::dialog_parameter_factory::DialogParameter::ReRollRegenerationMultiple(_))));

        let calls = rng.call_count;
        let decline = Action::UseInducement { inducement_type: Some("mortuaryAssistant".into()), card_id: None, player_ids: vec![] };
        let out = step.handle_command(&decline, &mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(rng.call_count, calls, "declined: no Regeneration re-rolled");
        assert!(game.turn_data_home.inducement_set.has_uses_left("mortuaryAssistant"));
        assert_eq!(game.field_model.player_state("home_03").unwrap().base(), ffb_model::enums::PS_SERIOUS_INJURY);
    }

    /// Java CLIENT_USE_INDUCEMENT with a player: the inducement is spent (ReportInducement) and
    /// that player's Regeneration rolled again; the other failure stands.
    #[test]
    fn a_chosen_regeneration_is_re_rolled_with_the_inducement() {
        let (mut game, mut step, _, mut rng) = (0..200u64)
            .map(two_regenerating_casualties)
            .find(|(_, _, out, _)| matches!(out.prompt,
                Some(ffb_model::prompts::AgentPrompt::RegenerationReRolls { ref player_ids, .. }) if player_ids.len() == 2))
            .expect("a seed where both Regenerations fail");
        let calls = rng.call_count;
        let accept = Action::UseInducement { inducement_type: Some("mortuaryAssistant".into()), card_id: None, player_ids: vec!["home_03".into()] };
        let out = step.handle_command(&accept, &mut game, &mut rng);
        assert_eq!(rng.call_count, calls + 1, "one fresh Regeneration die");
        assert!(!game.turn_data_home.inducement_set.has_uses_left("mortuaryAssistant"), "the inducement is spent");
        assert!(game.report_list.get_reports().iter().any(|r| r.get_id() == ffb_model::report::report_id::ReportId::INDUCEMENT));
        // No inducement left: the remaining failure is not asked again.
        assert_eq!(out.action, StepAction::NextStep, "{:?}", out.prompt);
    }

    /// Java bb2025 StepApothecaryMultiple: an SI from an attacker with several get-even keywords
    /// -> checkGettingEven's DialogSelectKeywordParameter; the answer pushes GETTING_EVEN.
    #[test]
    fn a_multi_block_serious_injury_asks_which_keyword_to_get_even_with() {
        use ffb_model::enums::{PlayerState, PS_SERIOUS_INJURY, PS_STANDING};
        let mut game = make_game();
        game.team_home.players.push(ffb_model::model::player::Player { id: "home_05".into(), name: "home_05".into(), ..Default::default() });
        game.team_away.players.push(ffb_model::model::player::Player {
            id: "away_01".into(), keywords: vec!["Troll".into(), "Goblin".into()], ..Default::default()
        });
        game.field_model.set_player_coordinate("home_05", FieldCoordinate::new(5, 7));
        game.field_model.set_player_state("home_05", PlayerState::new(PS_STANDING));
        let mut step = StepApothecaryMultiple::default();
        step.team_id = Some("home".to_string());
        let mut ir = make_injury("home_05", ApothecaryMode::Defender, ApothecaryStatus::NoApothecary);
        ir.injury_context_mut().injury = Some(PlayerState::new(PS_SERIOUS_INJURY));
        ir.injury_context_mut().attacker_id = Some("away_01".into());
        step.injury_results.push(ir);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(ffb_model::prompts::AgentPrompt::SelectKeyword { ref keywords, .. })
            if keywords == &vec!["Goblin".to_string(), "Troll".to_string()]), "{:?}", out.prompt);
        let answer = Action::KeywordSelection { player_id: "home_05".into(), keywords: vec!["Goblin".into()] };
        let out = step.handle_command(&answer, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.iter().flatten().any(|s| s.step_id == StepId::GettingEven));
        assert!(game.dialog_parameter.is_none());
    }
}
