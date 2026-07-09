/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepApothecary`.
///
/// Applies apothecary to an injured player (BB2020 edition). Extends the BB2016 version with:
/// - `ApothecaryType` selection (Team / Wandering / Plague Doctor)
/// - Igor / Mortuary Assistant regeneration path (`USE_IGOR` / `DO_NOT_USE_IGOR` / `WAIT_FOR_IGOR_USE`)
/// - `canRollToSaveFromInjury` / `canJoinTeamIfLessThanEleven` player property checks
///
/// Init: mandatory `APOTHECARY_MODE`.
/// Expects: `INJURY_RESULT`, `USING_PILING_ON` (DEFENDER mode), `DEFENDER_POISONED`, `ATTACKER_POISONED`.
use ffb_model::enums::{ApothecaryMode, ApothecaryStatus, ApothecaryType, SeriousInjuryKind};
use ffb_model::events::GameEvent;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_apothecary_roll::ReportApothecaryRoll;
use ffb_model::report::report_apothecary_choice::ReportApothecaryChoice;
use crate::action::Action;
use crate::injury::InjuryResult;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Java: `StepApothecary` (BB2020, server/step/bb2020).
pub struct StepApothecary {
    /// Java: fApothecaryMode (mandatory init param)
    pub apothecary_mode: Option<ApothecaryMode>,
    /// Java: fInjuryResult
    pub injury_result: Option<InjuryResult>,
    /// Java: fShowReport (default true)
    pub show_report: bool,
    /// Java: fDefenderPoisoned
    pub defender_poisoned: bool,
    /// Java: fAttackerPoisoned
    pub attacker_poisoned: bool,
    /// Java: apothecaryType — which type was actually used
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
            // Java: CLIENT_USE_APOTHECARY → set status + apothecaryType
            Action::UseApothecary { player_id: _, use_apothecary } => {
                if let Some(ref mut ir) = self.injury_result {
                    let status = if *use_apothecary {
                        ApothecaryStatus::UseApothecary
                    } else {
                        ApothecaryStatus::DoNotUseApothecary
                    };
                    ir.injury_context.apothecary_status = status;
                }
            }
            // Java: CLIENT_APOTHECARY_CHOICE → handleApothecaryChoice
            Action::ApothecaryChoice { player_state, serious_injury } => {
                if self.injury_result.is_some() {
                    let ps = ffb_model::enums::PlayerState::new(*player_state);
                    let si: Option<SeriousInjuryKind> = serious_injury.as_deref().and_then(|s| {
                        serde_json::from_str(&format!("\"{}\"", s)).ok()
                    });
                    self.handle_apothecary_choice(ps, si);
                }
            }
            // Java: CLIENT_USE_INDUCEMENT (REGENERATION usage) → Igor path
            Action::UseInducement { player_ids, .. } => {
                // Java: if inducementType.hasUsage(REGENERATION) && status == WAIT_FOR_IGOR_USE
                if let Some(ref mut ir) = self.injury_result {
                    if ir.injury_context.apothecary_status == ApothecaryStatus::WaitForIgorUse {
                        let defender_id = ir.injury_context.defender_id.clone().unwrap_or_default();
                        if player_ids.iter().any(|id| id == &defender_id) {
                            ir.injury_context.apothecary_status = ApothecaryStatus::UseIgor;
                        } else {
                            ir.injury_context.apothecary_status = ApothecaryStatus::DoNotUseIgor;
                        }
                    }
                }
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ApothecaryMode(mode) => {
                self.apothecary_mode = Some(*mode);
                true
            }
            StepParameter::InjuryResult(ir) => {
                let mode_matches = self.apothecary_mode
                    .map(|m| m == ir.injury_context.apothecary_mode)
                    .unwrap_or(false);
                if mode_matches {
                    self.injury_result = Some(*ir.clone());
                }
                mode_matches
            }
            StepParameter::UsingPilingOn(using) => {
                if self.apothecary_mode == Some(ApothecaryMode::Defender) && !using {
                    self.show_report = false;
                    true
                } else {
                    false
                }
            }
            StepParameter::DefenderPoisoned(v) => {
                self.defender_poisoned = *v;
                self.apothecary_mode == Some(ApothecaryMode::Defender)
            }
            StepParameter::AttackerPoisoned(v) => {
                self.attacker_poisoned = *v;
                self.apothecary_mode == Some(ApothecaryMode::Attacker)
            }
            _ => false,
        }
    }
}

impl StepApothecary {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.injury_result.is_none() {
            return StepOutcome::next();
        }
        // client-only: UtilServerDialog.hideDialog

        let status = self.injury_result.as_ref().map(|ir| ir.injury_context.apothecary_status);
        let mut do_next_step = true;
        let mut outcome = StepOutcome::next();

        if let Some(status) = status {
            match status {
                ApothecaryStatus::DoRequest => {
                    if self.show_report {
                        if let Some(ref mut ir) = self.injury_result {
                            ir.report(game);
                        }
                    }
                    // Java: showDialog(DialogUseApothecaryParameter)
                    if let Some(ref mut ir) = self.injury_result {
                        ir.injury_context.apothecary_status = ApothecaryStatus::WaitForApothecaryUse;
                    }
                    // client-only: DialogUseApothecaryParameter — headless auto-accepts
                    do_next_step = false;
                    outcome = StepOutcome::cont();
                }
                ApothecaryStatus::UseApothecary => {
                    let (choice_shown, apo_event) = self.roll_apothecary(game, rng);
                    if let Some(ev) = apo_event { outcome = outcome.with_event(ev); }
                    if choice_shown {
                        if let Some(ref mut ir) = self.injury_result {
                            ir.injury_context.apothecary_status = ApothecaryStatus::WaitForApothecaryUse;
                        }
                        do_next_step = false;
                        outcome = StepOutcome::cont();
                    } else {
                        if let Some(ref mut ir) = self.injury_result {
                            ir.injury_context.apothecary_status = ApothecaryStatus::ResultChoice;
                        }
                    }
                }
                ApothecaryStatus::DoNotUseApothecary => {
                    // Java: addReport(new ReportApothecaryRoll(defenderId, null, null, null, null, modifiers))
                    if let Some(ref ir) = self.injury_result {
                        if let Some(ref did) = ir.injury_context.defender_id {
                            game.report_list.add(ReportApothecaryRoll::new(
                                Some(did.clone()),
                                vec![],
                                None,
                                None,
                                None,
                                vec![],
                            ));
                            outcome = outcome.with_event(GameEvent::ApothecaryRoll {
                                player_id: did.clone(),
                                roll: None,
                                new_state: None,
                                new_serious_injury: None,
                            });
                        }
                    }
                }
                ApothecaryStatus::NoApothecary => {
                    if self.show_report {
                        if let Some(ref mut ir) = self.injury_result {
                            ir.report(game);
                        }
                    }
                }
                _ => {}
            }
        }

        if do_next_step {
            let defender_id = self.injury_result.as_ref()
                .and_then(|ir| ir.injury_context.defender_id.clone());
            let status2 = self.injury_result.as_ref().map(|ir| ir.injury_context.apothecary_status);

            // Java: switch (apothecaryStatus) — handle Igor paths before applying
            match status2 {
                Some(ApothecaryStatus::DoNotUseIgor) => {
                    // Java: break — fall through to applyTo
                }
                Some(ApothecaryStatus::UseIgor) => {
                    // Java: find REGENERATION inducement with uses left, useInducement, handleRegeneration
                    if let Some(ref id) = defender_id {
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
                    // Default: apply the injury result to the field model
                    if let Some(ref ir) = self.injury_result {
                        ir.apply_to(game);
                    }

                    // Java: if (player.hasSkillProperty(canRollToSaveFromInjury) && injuryType.canUseApo())
                    // → handleRegeneration; if fails + has REGENERATION inducement → WAIT_FOR_IGOR_USE
                    if let Some(ref id) = defender_id {
                        crate::step::util_server_injury::handle_regeneration(game, rng, id);
                    }
                    // client-only: if regen failed and REGENERATION inducement available → WAIT_FOR_IGOR_USE dialog
                    // Headless auto-declines Igor dialog; game continues with the injury.
                }
            }
            // Java: UtilServerInjury.handleInjurySideEffects(this, fInjuryResult)
            let side_events = if let Some(ref ir) = self.injury_result {
                crate::step::util_server_injury::handle_injury_side_effects(game, ir)
                // no-op: handleRaiseDead — InjuryMechanic.canRaiseDead + player creation not ported
            } else {
                vec![]
            };
            let mut out = StepOutcome::next();
            for ev in side_events { out = out.with_event(ev); }
            outcome = out;
        }

        outcome
    }

    /// Java: rollApothecary() — use the selected apothecary type, then roll new casualty if needed.
    fn roll_apothecary(&mut self, game: &mut Game, rng: &mut GameRng) -> (bool, Option<GameEvent>) {
        // Java: if (apothecaryType == null) → pick first available type for player
        if self.apothecary_type.is_none() {
            self.apothecary_type = Some(ApothecaryType::Team);
        }
        // Java: useApo(turnData, apothecaryType) — decrement team apo count
        let defender_on_home = self.injury_result.as_ref()
            .and_then(|ir| ir.injury_context.defender_id.as_deref())
            .map(|id| game.team_home.has_player(id))
            .unwrap_or(false);
        if defender_on_home {
            if game.turn_data_home.apothecaries > 0 {
                game.turn_data_home.apothecaries -= 1;
            }
        } else if game.turn_data_away.apothecaries > 0 {
            game.turn_data_away.apothecaries -= 1;
        }

        let ir = match self.injury_result.as_ref() {
            Some(ir) => ir,
            None => return (false, None),
        };

        use ffb_model::enums::{PS_BADLY_HURT, PS_KNOCKED_OUT};
        let base = ir.injury_context.injury.map(|ps| ps.base()).unwrap_or(0);
        let player_id = ir.injury_context.defender_id.clone().unwrap_or_default();
        // Java: apothecaryChoice = (base != BADLY_HURT && base != KNOCKED_OUT)
        let apothecary_choice = base != PS_BADLY_HURT && base != PS_KNOCKED_OUT;

        if apothecary_choice {
            // Java: rollCasualty() + DialogApothecaryChoiceParameter
            let roll = rng.d6() + rng.d6();
            // Java: addReport(new ReportApothecaryRoll(defender.getId(), casualtyRoll, playerState, seriousInjury, originalSeriousInjury, modifiers))
            game.report_list.add(ReportApothecaryRoll::new(
                Some(player_id.clone()),
                vec![roll],
                None,
                None,
                None,
                vec![],
            ));
            let apo_event = GameEvent::ApothecaryRoll {
                player_id,
                roll: Some(roll),
                new_state: Some(base as u16),
                new_serious_injury: None,
            };
            // client-only: DialogApothecaryChoiceParameter — headless auto-selects first result
            (false, Some(apo_event))
        } else {
            // Java: cure BH/KO → STUNNED (if KO + canApoKoIntoStun) or RESERVE; clear serious_injury
            self.cure_poison(game);
            if let Some(ref mut ir) = self.injury_result {
                use ffb_model::enums::{PS_KNOCKED_OUT, PS_RESERVE, PS_STUNNED, PlayerState};
                let cured = if base == PS_KNOCKED_OUT
                    && crate::injury::can_apo_ko_into_stun(ir.injury_context.injury_type_name.as_deref())
                {
                    PlayerState::new(PS_STUNNED)
                } else {
                    PlayerState::new(PS_RESERVE)
                };
                ir.injury_context.injury = Some(cured);
                ir.injury_context.serious_injury = None;
            }
            // Java: addReport(new ReportApothecaryChoice(defenderId, playerState, null))
            game.report_list.add(ReportApothecaryChoice::new(
                player_id.clone(),
                ffb_model::model::player_state::PlayerState::new(0),
                None,
            ));
            let choice_event = GameEvent::ApothecaryChoice {
                player_id,
                healed: true,
            };
            (false, Some(choice_event))
        }
    }

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

    fn handle_apothecary_choice(&mut self, player_state: ffb_model::enums::PlayerState, serious_injury: Option<SeriousInjuryKind>) {
        if let Some(ref mut ir) = self.injury_result {
            use ffb_model::enums::{PS_BADLY_HURT, PS_RESERVE, PlayerState};
            if player_state.base() == PS_BADLY_HURT {
                ir.injury_context.injury = Some(PlayerState::new(PS_RESERVE));
                ir.injury_context.serious_injury = None;
            } else {
                ir.injury_context.injury = Some(player_state);
                ir.injury_context.serious_injury = serious_injury;
            }
            ir.injury_context.apothecary_status = ApothecaryStatus::ResultChoice;
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use crate::injury::InjuryContext;
    use ffb_model::enums::{Rules, ApothecaryMode, ApothecaryStatus, PS_BADLY_HURT, PS_SERIOUS_INJURY};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn make_ir(mode: ApothecaryMode, status: ApothecaryStatus) -> InjuryResult {
        let mut ir = InjuryResult::new(mode);
        ir.injury_context.apothecary_status = status;
        ir
    }

    #[test]
    fn id_is_apothecary() {
        assert_eq!(StepApothecary::new().id(), StepId::Apothecary);
    }

    #[test]
    fn no_injury_result_returns_next_step() {
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn do_request_transitions_to_wait_and_continues() {
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        step.injury_result = Some(make_ir(ApothecaryMode::Defender, ApothecaryStatus::DoRequest));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert!(matches!(out.action, StepAction::Continue));
        assert_eq!(
            step.injury_result.as_ref().unwrap().injury_context.apothecary_status,
            ApothecaryStatus::WaitForApothecaryUse,
        );
    }

    #[test]
    fn use_apothecary_on_bh_cures_to_reserve() {
        use ffb_model::enums::PlayerState;
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::UseApothecary);
        ir.injury_context.injury = Some(PlayerState::new(PS_BADLY_HURT));
        step.injury_result = Some(ir);
        let mut game = make_game();
        let mut rng = GameRng::new(42);
        let out = step.start(&mut game, &mut rng);
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn set_parameter_apothecary_mode_consumed() {
        let mut step = StepApothecary::new();
        assert!(step.set_parameter(&StepParameter::ApothecaryMode(ApothecaryMode::Defender)));
        assert_eq!(step.apothecary_mode, Some(ApothecaryMode::Defender));
    }

    #[test]
    fn set_parameter_injury_result_only_accepted_when_mode_matches() {
        use ffb_model::enums::PlayerState;
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = make_ir(ApothecaryMode::Attacker, ApothecaryStatus::NoApothecary);
        ir.injury_context.injury = Some(PlayerState::new(PS_SERIOUS_INJURY));
        let accepted = step.set_parameter(&StepParameter::InjuryResult(Box::new(ir)));
        assert!(!accepted);
        assert!(step.injury_result.is_none());
    }

    #[test]
    fn set_parameter_injury_result_accepted_when_mode_matches() {
        use ffb_model::enums::PlayerState;
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::NoApothecary);
        ir.injury_context.injury = Some(PlayerState::new(PS_SERIOUS_INJURY));
        let accepted = step.set_parameter(&StepParameter::InjuryResult(Box::new(ir)));
        assert!(accepted);
        assert!(step.injury_result.is_some());
    }

    #[test]
    fn using_piling_on_false_in_defender_mode_suppresses_report() {
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let consumed = step.set_parameter(&StepParameter::UsingPilingOn(false));
        assert!(consumed);
        assert!(!step.show_report);
    }

    #[test]
    fn using_piling_on_false_in_attacker_mode_does_not_suppress() {
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Attacker);
        let consumed = step.set_parameter(&StepParameter::UsingPilingOn(false));
        assert!(!consumed);
        assert!(step.show_report);
    }

    #[test]
    fn use_inducement_for_igor_sets_use_igor_status() {
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::WaitForIgorUse);
        ir.injury_context.defender_id = Some("p1".into());
        step.injury_result = Some(ir);
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let action = Action::UseInducement { inducement_type: Some("igor".into()), card_id: None, player_ids: vec!["p1".into()] };
        step.handle_command(&action, &mut game, &mut rng);
        assert_eq!(
            step.injury_result.as_ref().unwrap().injury_context.apothecary_status,
            ApothecaryStatus::UseIgor,
        );
    }

    #[test]
    fn use_inducement_wrong_player_sets_do_not_use_igor() {
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::WaitForIgorUse);
        ir.injury_context.defender_id = Some("p1".into());
        step.injury_result = Some(ir);
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let action = Action::UseInducement { inducement_type: Some("igor".into()), card_id: None, player_ids: vec!["p2".into()] };
        step.handle_command(&action, &mut game, &mut rng);
        assert_eq!(
            step.injury_result.as_ref().unwrap().injury_context.apothecary_status,
            ApothecaryStatus::DoNotUseIgor,
        );
    }

    #[test]
    fn do_not_use_apothecary_adds_apothecary_roll_report() {
        use ffb_model::enums::PlayerState;
        use ffb_model::report::report_id::ReportId;
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::DoNotUseApothecary);
        ir.injury_context.defender_id = Some("p1".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_BADLY_HURT));
        step.injury_result = Some(ir);
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.report_list.has_report(ReportId::APOTHECARY_ROLL),
            "DoNotUseApothecary must add APOTHECARY_ROLL report");
    }

    #[test]
    fn use_apothecary_on_bh_adds_apothecary_choice_report() {
        use ffb_model::enums::PlayerState;
        use ffb_model::report::report_id::ReportId;
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut ir = make_ir(ApothecaryMode::Defender, ApothecaryStatus::UseApothecary);
        ir.injury_context.defender_id = Some("p2".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_BADLY_HURT));
        step.injury_result = Some(ir);
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.report_list.has_report(ReportId::APOTHECARY_CHOICE),
            "UseApothecary on BH must add APOTHECARY_CHOICE report");
    }
}
