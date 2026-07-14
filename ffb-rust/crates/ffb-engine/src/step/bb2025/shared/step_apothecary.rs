/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.StepApothecary.
///
/// Handles apothecary use after an injury. Driven by ApothecaryStatus state machine:
///   DO_REQUEST → show UseApothecary prompt → WAIT_FOR_APOTHECARY_USE (Continue)
///   CLIENT_USE_APOTHECARY(true)  → USE_APOTHECARY → rollApothecary()
///     → if dialog needed: show ApothecaryChoice prompt (Continue)
///     → else: RESULT_CHOICE → applyTo + sideEffects → NextStep
///   CLIENT_USE_APOTHECARY(false) → DO_NOT_USE_APOTHECARY → applyTo + sideEffects → NextStep
///   NO_APOTHECARY → applyTo + sideEffects → NextStep
use ffb_model::enums::{
    ApothecaryMode, ApothecaryStatus, ApothecaryType,
    PlayerState, PS_BADLY_HURT, PS_KNOCKED_OUT, PS_RESERVE, PS_SERIOUS_INJURY, PS_STUNNED,
    SeriousInjuryKind,
};
use ffb_model::events::GameEvent;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_apothecary_roll::ReportApothecaryRoll;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::rng::GameRng;
use ffb_model::prompts::AgentPrompt;
use crate::action::Action;
use crate::injury::InjuryResult;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

pub struct StepApothecary {
    /// Java: fApothecaryMode (mandatory init param)
    pub apothecary_mode: Option<ApothecaryMode>,
    /// Java: fInjuryResult
    pub injury_result: Option<InjuryResult>,
    /// Java: fShowReport (default true in Java constructor)
    pub show_report: bool,
    /// Java: fDefenderPoisoned
    pub defender_poisoned: bool,
    /// Java: fAttackerPoisoned
    pub attacker_poisoned: bool,
    /// Java: apothecaryType
    pub apothecary_type: Option<ApothecaryType>,
}

impl StepApothecary {
    pub fn new() -> Self {
        Self {
            apothecary_mode: None,
            injury_result: None,
            show_report: true,
            defender_poisoned: false,
            attacker_poisoned: false,
            apothecary_type: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (fInjuryResult == null) { getResult().setNextAction(NEXT_STEP); return; }
        if self.injury_result.is_none() {
            return StepOutcome::next();
        }

        // Java: UtilServerDialog.hideDialog(getGameState()) — no-op in Rust

        let mut do_next_step = true;
        let mut pending_events: Vec<GameEvent> = vec![];

        let apo_status = self.injury_result.as_ref().unwrap().injury_context.apothecary_status;

        // Java: if (fInjuryResult.injuryContext().getApothecaryStatus() != null) { switch ... }
        match apo_status {
            ApothecaryStatus::DoRequest => {
                if self.show_report {
                    if let Some(ref mut ir) = self.injury_result {
                        ir.report(game);
                    }
                }
                // Java: apothecaryTypes = ApothecaryType.forPlayer(game, defender, playerState)
                // Java: showDialog(new DialogUseApothecaryParameter(defenderId, playerState, seriousInjury, apothecaryTypes))
                // Java: setApothecaryStatus(WAIT_FOR_APOTHECARY_USE)
                let defender_id = self.injury_result.as_ref()
                    .and_then(|ir| ir.injury_context.defender_id.clone())
                    .unwrap_or_default();
                self.injury_result.as_mut().unwrap().injury_context.apothecary_status =
                    ApothecaryStatus::WaitForApothecaryUse;
                do_next_step = false;
                return StepOutcome::cont().with_prompt(AgentPrompt::UseApothecary {
                    player_id: defender_id,
                    apothecary_type: "team".to_string(),
                });
            }
            ApothecaryStatus::UseApothecary => {
                // Java: if (rollApothecary()) { setStatus(WAIT_FOR_APOTHECARY_USE); doNextStep=false }
                //       else { setStatus(RESULT_CHOICE) }
                let (showed_dialog, apo_events) = self.roll_apothecary(game, rng);
                if showed_dialog {
                    let defender_id = self.injury_result.as_ref()
                        .and_then(|ir| ir.injury_context.defender_id.clone())
                        .unwrap_or_default();
                    self.injury_result.as_mut().unwrap().injury_context.apothecary_status =
                        ApothecaryStatus::WaitForApothecaryUse;
                    do_next_step = false;
                    let mut out = StepOutcome::cont().with_prompt(AgentPrompt::ApothecaryChoice {
                        player_id: defender_id,
                        can_heal: true,
                    });
                    for ev in apo_events { out = out.with_event(ev); }
                    return out;
                } else {
                    self.injury_result.as_mut().unwrap().injury_context.apothecary_status =
                        ApothecaryStatus::ResultChoice;
                    // apo_events = [ApothecaryRoll, ApothecaryChoice] when healed from SI/BH, or [ApothecaryChoice] for KO
                    pending_events.extend(apo_events);
                }
            }
            ApothecaryStatus::DoNotUseApothecary => {
                // Java: getResult().addReport(new ReportApothecaryRoll(defenderId, null, null, null, null, modifiers))
                // null roll/state = "no apo roll taken" (team declined)
                let defender_id = self.injury_result.as_ref()
                    .and_then(|ir| ir.injury_context.defender_id.clone())
                    .unwrap_or_default();
                game.report_list.add(ReportApothecaryRoll::new(
                    Some(defender_id.clone()), vec![], None, None, None, vec![],
                ));
                pending_events.push(GameEvent::ApothecaryRoll {
                    player_id: defender_id,
                    roll: None,
                    new_state: None,
                    new_serious_injury: None,
                });
            }
            ApothecaryStatus::NoApothecary => {
                if self.show_report {
                    if let Some(ref mut ir) = self.injury_result {
                        ir.report(game);
                    }
                }
            }
            // WAIT_FOR_APOTHECARY_USE, WAIT_FOR_APOTHECARY_CHOICE, RESULT_CHOICE, etc. — fall through
            _ => {}
        }

        if do_next_step {
            // Java: second switch on apothecaryStatus after first pass
            let apo_status2 = self.injury_result.as_ref().unwrap().injury_context.apothecary_status;
            match apo_status2 {
                ApothecaryStatus::DoNotUseIgor => {
                    // Java: nothing — fall through to handleInjurySideEffects
                }
                ApothecaryStatus::UseIgor => {
                    // Java: find REGENERATION inducement with uses left, useInducement, handleRegeneration
                    let defender_id2 = self.injury_result.as_ref()
                        .and_then(|ir| ir.injury_context.defender_id.clone());
                    if let Some(ref id) = defender_id2 {
                        let is_home = game.team_home.has_player(id);
                        if is_home {
                            game.turn_data_home.inducement_set.use_one_for_usage(Usage::REGENERATION);
                        } else {
                            game.turn_data_away.inducement_set.use_one_for_usage(Usage::REGENERATION);
                        }
                        crate::step::util_server_injury::handle_regeneration(game, rng, id);
                    }
                }
                _ => {
                    // Java: fInjuryResult.applyTo(this)
                    if let Some(ir) = self.injury_result.as_ref() {
                        ir.apply_to(game);
                    }
                    // Java: if (playerState.isCasualty() && canRollToSaveFromInjury && canUseApo())
                    if let Some(defender_id) = self.injury_result.as_ref()
                        .and_then(|ir| ir.injury_context.defender_id.clone())
                    {
                        let is_casualty = game.field_model.player_state(&defender_id)
                            .map(|s| s.is_casualty())
                            .unwrap_or(false);
                        if is_casualty {
                            let regenerated = crate::step::util_server_injury::handle_regeneration(
                                game, rng, &defender_id,
                            );
                            if regenerated {
                                self.cure_poison(game);
                            } else {
                                // client-only: Igor/WAIT_FOR_IGOR_USE dialog — headless auto-declines
                            }
                        }
                    }
                }
            }
        }

        if do_next_step {
            // Java: UtilServerInjury.handleInjurySideEffects(this, fInjuryResult)
            let side_events = if let Some(ir) = &self.injury_result {
                crate::step::util_server_injury::handle_injury_side_effects(game, ir)
            } else {
                vec![]
            };
            let mut out = StepOutcome::next();
            for ev in pending_events { out = out.with_event(ev); }
            for ev in side_events { out = out.with_event(ev); }
            out
        } else {
            StepOutcome::cont()
        }
    }

    /// Java: private boolean rollApothecary()
    ///
    /// Marks the apothecary used, rolls a new casualty die.
    /// Returns (showed_dialog, events) — events are all reports emitted during this call.
    fn roll_apothecary(&mut self, game: &mut Game, rng: &mut GameRng) -> (bool, Vec<GameEvent>) {
        // Java: use_apo(turnData, apothecaryType)
        let apo_type = self.apothecary_type.unwrap_or(ffb_model::enums::ApothecaryType::Team);
        let defender_on_home = self.injury_result.as_ref()
            .and_then(|ir| ir.injury_context.defender_id.as_deref())
            .map(|id| game.team_home.has_player(id))
            .unwrap_or(false);
        if defender_on_home {
            game.turn_data_home.use_apothecary(apo_type);
        } else {
            game.turn_data_away.use_apothecary(apo_type);
        }

        let ir = match self.injury_result.as_ref() {
            Some(ir) => ir,
            None => return (false, vec![]),
        };
        let player_state_base = ir.injury_context.injury.map(|s| s.base()).unwrap_or(PS_BADLY_HURT);
        let player_id = ir.injury_context.defender_id.clone().unwrap_or_default();

        // Java: apothecaryChoice = !(BADLY_HURT || KNOCKED_OUT)
        let mut apothecary_choice =
            player_state_base != PS_BADLY_HURT && player_state_base != PS_KNOCKED_OUT;

        let mut events: Vec<GameEvent> = vec![];
        if apothecary_choice {
            // Java: newInjuryResult = roll new casualty die + interpret
            // Java: apothecaryChoice = (newState != BADLY_HURT)
            let roll = rng.die(16);
            let new_state_base = if roll >= 15 { PS_RESERVE } // RIP re-rolled
                else if roll >= 9 { PS_SERIOUS_INJURY }
                else { PS_BADLY_HURT };
            apothecary_choice = new_state_base != PS_BADLY_HURT;
            // Java: addReport(new ReportApothecaryRoll(defender, casualtyRoll, newState, newSI, origSI, mods))
            game.report_list.add(ReportApothecaryRoll::new(
                Some(player_id.clone()), vec![roll], None, None, None, vec![],
            ));
            events.push(GameEvent::ApothecaryRoll {
                player_id: player_id.clone(),
                roll: Some(roll),
                new_state: Some(new_state_base as u16),
                new_serious_injury: None,
            });
        }

        if !apothecary_choice {
            // Java: curePoison() before modifying injury_result (cure_poison borrows self)
            if player_state_base != PS_KNOCKED_OUT {
                self.cure_poison(game);
            }
            // Java: fInjuryResult.injuryContext().setSeriousInjury(null)
            // Java: if (KO && canApoKoIntoStun) → STUNNED else RESERVE
            let ir = self.injury_result.as_mut().unwrap();
            ir.injury_context.serious_injury = None;
            if player_state_base == PS_KNOCKED_OUT {
                // Java: injuryType.canApoKoIntoStun() — true for most types, false for CrowdPush/TrapDoor
                if crate::injury::can_apo_ko_into_stun(ir.injury_context.injury_type_name.as_deref()) {
                    ir.injury_context.set_injury(PlayerState::new(PS_STUNNED));
                } else {
                    ir.injury_context.set_injury(PlayerState::new(PS_RESERVE));
                }
            } else {
                ir.injury_context.set_injury(PlayerState::new(PS_RESERVE));
            }
            // Java: addReport(new ReportApothecaryChoice(defenderId, playerState, null))
            // healed=true: apo cured the player (BH→RESERVE or KO→STUNNED)
            events.push(GameEvent::ApothecaryChoice {
                player_id,
                healed: true,
            });
        }

        (apothecary_choice, events)
    }

    /// Java: private void curePoison() — removes POISONED card effect from the injured player.
    fn cure_poison(&self, game: &mut Game) {
        use ffb_model::enums::CardEffect;
        let player_id = self.injury_result.as_ref()
            .and_then(|ir| ir.injury_context.defender_id.clone());
        let player_id = match player_id { Some(id) => id, None => return };
        let mode = match &self.apothecary_mode { Some(m) => *m, None => return };
        let should_cure = matches!(mode, ApothecaryMode::Defender) && self.defender_poisoned
            || matches!(mode, ApothecaryMode::Attacker) && self.attacker_poisoned;
        if should_cure {
            game.field_model.remove_card_effect(&player_id, CardEffect::Poisoned);
        }
    }

    /// Java: private void handleApothecaryChoice(PlayerState pPlayerState, SeriousInjury pSeriousInjury)
    fn handle_apothecary_choice(&mut self, player_state: PlayerState, serious_injury: Option<SeriousInjuryKind>) {
        if let Some(ir) = self.injury_result.as_mut() {
            if player_state.base() == PS_BADLY_HURT {
                // Java: BADLY_HURT choice → set RESERVE (healed)
                ir.injury_context.set_injury(PlayerState::new(PS_RESERVE));
                ir.injury_context.serious_injury = None;
            } else {
                ir.injury_context.set_injury(player_state);
                ir.injury_context.serious_injury = serious_injury;
            }
            ir.injury_context.apothecary_status = ApothecaryStatus::ResultChoice;
        }
    }
}

impl Default for StepApothecary {
    fn default() -> Self { Self::new() }
}

impl Step for StepApothecary {
    fn id(&self) -> StepId { StepId::Apothecary }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_USE_APOTHECARY → set status USE/DO_NOT_USE, re-run executeStep
            Action::UseApothecary { player_id, use_apothecary } => {
                let defender_id = self.injury_result.as_ref()
                    .and_then(|ir| ir.injury_context.defender_id.as_deref())
                    .unwrap_or("");
                if self.injury_result.is_some() && player_id == defender_id {
                    let new_status = if *use_apothecary {
                        ApothecaryStatus::UseApothecary
                    } else {
                        ApothecaryStatus::DoNotUseApothecary
                    };
                    // Java: apothecaryType = useApothecaryCommand.getApothecaryType()
                    // Action::UseApothecary doesn't carry type yet — default to Team
                    self.apothecary_type = Some(ApothecaryType::Team);
                    self.injury_result.as_mut().unwrap().injury_context.apothecary_status = new_status;
                    return self.execute_step(game, rng);
                }
            }
            // Java: CLIENT_APOTHECARY_CHOICE → handleApothecaryChoice(state, seriousInjury)
            Action::ApothecaryChoice { player_state, serious_injury } => {
                if self.injury_result.is_some() {
                    let ps = PlayerState::new(*player_state);
                    let si: Option<SeriousInjuryKind> = serious_injury.as_deref().and_then(|s| {
                        serde_json::from_str(&format!("\"{}\"", s)).ok()
                    });
                    self.handle_apothecary_choice(ps, si);
                }
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: StepParameterKey.APOTHECARY_MODE (mandatory init param)
            StepParameter::ApothecaryMode(v) => {
                self.apothecary_mode = Some(*v);
                true
            }
            // Java: INJURY_RESULT — accepted only when modes match
            StepParameter::InjuryResult(ir) => {
                if self.apothecary_mode == Some(ir.injury_context().apothecary_mode) {
                    self.injury_result = Some((**ir).clone());
                    true
                } else {
                    false
                }
            }
            // Java: USING_PILING_ON — if (DEFENDER && !pilingOn) fShowReport = false
            StepParameter::UsingPilingOn(v) => {
                if self.apothecary_mode == Some(ApothecaryMode::Defender) && !v {
                    self.show_report = false;
                    true
                } else {
                    false
                }
            }
            // Java: DEFENDER_POISONED
            StepParameter::DefenderPoisoned(v) => {
                self.defender_poisoned = *v;
                self.apothecary_mode == Some(ApothecaryMode::Defender)
            }
            // Java: ATTACKER_POISONED
            StepParameter::AttackerPoisoned(v) => {
                self.attacker_poisoned = *v;
                self.apothecary_mode == Some(ApothecaryMode::Attacker)
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn make_ir(mode: ApothecaryMode, status: ApothecaryStatus) -> InjuryResult {
        let mut ir = InjuryResult::new(mode);
        ir.injury_context.apothecary_status = status;
        ir.injury_context.defender_id = Some("d1".to_string());
        ir.injury_context.set_injury(PlayerState::new(PS_SERIOUS_INJURY));
        ir
    }

    #[test]
    fn new_sets_show_report_true() {
        let step = StepApothecary::new();
        assert!(step.show_report);
    }

    #[test]
    fn start_without_injury_result_returns_next() {
        let mut game = make_game();
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn apothecary_mode_parameter_accepted() {
        let mut step = StepApothecary::default();
        let accepted = step.set_parameter(&StepParameter::ApothecaryMode(ApothecaryMode::Attacker));
        assert!(accepted);
        assert_eq!(step.apothecary_mode, Some(ApothecaryMode::Attacker));
    }

    #[test]
    fn injury_result_accepted_when_modes_match() {
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let ir = InjuryResult::new(ApothecaryMode::Defender);
        let accepted = step.set_parameter(&StepParameter::InjuryResult(Box::new(ir)));
        assert!(accepted);
        assert!(step.injury_result.is_some());
    }

    #[test]
    fn injury_result_rejected_when_modes_differ() {
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let ir = InjuryResult::new(ApothecaryMode::Attacker);
        let accepted = step.set_parameter(&StepParameter::InjuryResult(Box::new(ir)));
        assert!(!accepted);
        assert!(step.injury_result.is_none());
    }

    #[test]
    fn using_piling_on_false_clears_show_report_in_defender_mode() {
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        step.set_parameter(&StepParameter::UsingPilingOn(false));
        assert!(!step.show_report);
    }

    #[test]
    fn using_piling_on_true_does_not_clear_show_report() {
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        step.set_parameter(&StepParameter::UsingPilingOn(true));
        assert!(step.show_report);
    }

    #[test]
    fn defender_poisoned_parameter_accepted_in_defender_mode() {
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let accepted = step.set_parameter(&StepParameter::DefenderPoisoned(true));
        assert!(accepted);
        assert!(step.defender_poisoned);
    }

    #[test]
    fn attacker_poisoned_parameter_accepted_in_attacker_mode() {
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Attacker);
        let accepted = step.set_parameter(&StepParameter::AttackerPoisoned(true));
        assert!(accepted);
        assert!(step.attacker_poisoned);
    }

    #[test]
    fn no_apothecary_status_returns_next() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::NoApothecary);
        ir.injury_context.set_injury(PlayerState::new(PS_STUNNED));
        step.injury_result = Some(ir);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn do_not_use_apothecary_status_returns_next() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::DoNotUseApothecary);
        step.injury_result = Some(ir);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn do_request_status_shows_prompt_and_continues() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::DoRequest);
        step.injury_result = Some(ir);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
        assert_eq!(
            step.injury_result.as_ref().unwrap().injury_context.apothecary_status,
            ApothecaryStatus::WaitForApothecaryUse
        );
    }

    #[test]
    fn use_apothecary_command_false_sets_do_not_use() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::WaitForApothecaryUse);
        step.injury_result = Some(ir);
        let action = Action::UseApothecary { player_id: "d1".to_string(), use_apothecary: false };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        // DO_NOT_USE_APOTHECARY → falls through to applyTo/sideEffects → NextStep
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn use_apothecary_command_true_triggers_roll() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::WaitForApothecaryUse);
        step.injury_result = Some(ir);
        // Seed the RNG so rollApothecary's die(16) produces a known result
        let action = Action::UseApothecary { player_id: "d1".to_string(), use_apothecary: true };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(42));
        // Either showed choice dialog (Continue) or resolved directly (NextStep)
        assert!(matches!(out.action, StepAction::Continue | StepAction::NextStep));
    }

    #[test]
    fn use_apothecary_command_wrong_player_ignored() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::WaitForApothecaryUse);
        step.injury_result = Some(ir);
        let action = Action::UseApothecary { player_id: "wrong".to_string(), use_apothecary: false };
        // wrong player_id → not matched → executeStep with unchanged WaitForApothecaryUse
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        // WaitForApothecaryUse falls through the main switch (no match) → applyTo todo → NextStep
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_apothecary_choice_badly_hurt_sets_reserve() {
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        step.injury_result = Some(make_ir(ApothecaryMode::Defender, ApothecaryStatus::WaitForApothecaryUse));
        step.handle_apothecary_choice(PlayerState::new(PS_BADLY_HURT), None);
        let ir = step.injury_result.as_ref().unwrap();
        assert_eq!(ir.injury_context.injury.unwrap().base(), PS_RESERVE);
        assert!(ir.injury_context.serious_injury.is_none());
        assert_eq!(ir.injury_context.apothecary_status, ApothecaryStatus::ResultChoice);
    }

    #[test]
    fn handle_apothecary_choice_serious_injury_preserved() {
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        step.injury_result = Some(make_ir(ApothecaryMode::Defender, ApothecaryStatus::WaitForApothecaryUse));
        let si = PlayerState::new(PS_SERIOUS_INJURY);
        step.handle_apothecary_choice(si, Some(SeriousInjuryKind::HeadInjuryAv));
        let ir = step.injury_result.as_ref().unwrap();
        assert_eq!(ir.injury_context.injury.unwrap().base(), PS_SERIOUS_INJURY);
        assert_eq!(ir.injury_context.serious_injury, Some(SeriousInjuryKind::HeadInjuryAv));
    }

    #[test]
    fn roll_apothecary_ko_sets_stunned() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        ir.injury_context.defender_id = Some("d1".to_string());
        ir.injury_context.set_injury(PlayerState::new(PS_KNOCKED_OUT));
        step.injury_result = Some(ir);
        step.roll_apothecary(&mut game, &mut GameRng::new(0));
        let result = step.injury_result.as_ref().unwrap().injury_context.injury.unwrap().base();
        assert_eq!(result, PS_STUNNED);
    }

    #[test]
    fn roll_apothecary_badly_hurt_sets_reserve() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        ir.injury_context.defender_id = Some("d1".to_string());
        ir.injury_context.set_injury(PlayerState::new(PS_BADLY_HURT));
        step.injury_result = Some(ir);
        step.roll_apothecary(&mut game, &mut GameRng::new(0));
        let result = step.injury_result.as_ref().unwrap().injury_context.injury.unwrap().base();
        assert_eq!(result, PS_RESERVE);
    }

    #[test]
    fn apothecary_choice_action_badly_hurt_cures_to_reserve() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::WaitForApothecaryUse);
        ir.injury_context.set_injury(PlayerState::new(PS_BADLY_HURT));
        step.injury_result = Some(ir);
        let action = Action::ApothecaryChoice { player_state: PS_BADLY_HURT, serious_injury: None };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        let result = step.injury_result.as_ref().unwrap();
        assert_eq!(result.injury_context.injury.unwrap().base(), PS_RESERVE);
        assert_eq!(result.injury_context.apothecary_status, ApothecaryStatus::ResultChoice);
    }

    #[test]
    fn apothecary_choice_action_serious_injury_name_parsed() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        step.injury_result = Some(make_ir(ApothecaryMode::Defender, ApothecaryStatus::WaitForApothecaryUse));
        let action = Action::ApothecaryChoice {
            player_state: PS_SERIOUS_INJURY,
            serious_injury: Some("HeadInjuryAv".into()),
        };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        let result = step.injury_result.as_ref().unwrap();
        assert_eq!(result.injury_context.serious_injury, Some(SeriousInjuryKind::HeadInjuryAv));
        assert_eq!(result.injury_context.apothecary_status, ApothecaryStatus::ResultChoice);
    }

    #[test]
    fn apothecary_choice_action_no_injury_result_noop() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        let action = Action::ApothecaryChoice { player_state: 0, serious_injury: None };
        // Should not panic
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(step.injury_result.is_none());
    }

    #[test]
    fn roll_apothecary_serious_injury_emits_apothecary_roll_event() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        ir.injury_context.defender_id = Some("d1".to_string());
        ir.injury_context.set_injury(PlayerState::new(PS_SERIOUS_INJURY));
        step.injury_result = Some(ir);
        let (_showed_dialog, events) = step.roll_apothecary(&mut game, &mut GameRng::new(0));
        // Serious injury triggers a real roll → first event is ApothecaryRoll
        assert!(!events.is_empty());
        assert!(matches!(events[0], GameEvent::ApothecaryRoll { .. }));
    }

    #[test]
    fn roll_apothecary_ko_emits_apothecary_choice_event() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        ir.injury_context.defender_id = Some("d1".to_string());
        ir.injury_context.set_injury(PlayerState::new(PS_KNOCKED_OUT));
        step.injury_result = Some(ir);
        let (_showed_dialog, events) = step.roll_apothecary(&mut game, &mut GameRng::new(0));
        // KO → no die roll, player healed to STUNNED → ApothecaryChoice { healed: true }
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], GameEvent::ApothecaryChoice { healed: true, .. }));
    }

    #[test]
    fn roll_apothecary_decrements_defender_team_apothecaries() {
        let mut game = make_game();
        game.turn_data_away.apothecaries = 2;
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        ir.injury_context.defender_id = Some("away_p1".to_string());
        ir.injury_context.set_injury(PlayerState::new(PS_KNOCKED_OUT));
        step.injury_result = Some(ir);
        step.roll_apothecary(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_data_away.apothecaries, 1);
    }

    #[test]
    fn do_not_use_apothecary_adds_apothecary_roll_report() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::DoNotUseApothecary);
        step.injury_result = Some(ir);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::APOTHECARY_ROLL));
    }

    #[test]
    fn roll_apothecary_serious_injury_adds_apothecary_roll_report() {
        let mut game = make_game();
        let mut step = StepApothecary::default();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        ir.injury_context.defender_id = Some("d1".to_string());
        ir.injury_context.set_injury(PlayerState::new(PS_SERIOUS_INJURY));
        step.injury_result = Some(ir);
        step.roll_apothecary(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::APOTHECARY_ROLL));
    }
}
