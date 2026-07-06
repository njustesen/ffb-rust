use ffb_model::enums::{ApothecaryMode, ApothecaryStatus, SeriousInjuryKind};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::injury::InjuryResult;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepApothecary.
///
/// Applies apothecary to an injured player (BB2016 edition).
/// BB2016 has no Igor (no Mortuary Assistant) — USE_IGOR path is intentionally excluded.
///
/// Init: mandatory APOTHECARY_MODE.
/// Expects: INJURY_RESULT (InjuryResult whose apothecaryMode matches),
///          USING_PILING_ON (bool, DEFENDER mode only),
///          DEFENDER_POISONED / ATTACKER_POISONED (bool).
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
}

impl StepApothecary {
    pub fn new() -> Self {
        Self {
            apothecary_mode: None,
            injury_result: None,
            show_report: true,
            defender_poisoned: false,
            attacker_poisoned: false,
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
            // Java: CLIENT_USE_APOTHECARY → set ApothecaryStatus
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
            // Java: CLIENT_APOTHECARY_CHOICE → handleApothecaryChoice (if injury_result present and player matches)
            Action::ApothecaryChoice { player_state, serious_injury } => {
                if self.injury_result.is_some() {
                    let ps = ffb_model::enums::PlayerState::new(*player_state);
                    // Java: getSeriousInjury() returns a SeriousInjury enum; convert string name via serde
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
            StepParameter::ApothecaryMode(mode) => {
                self.apothecary_mode = Some(*mode);
                true
            }
            StepParameter::InjuryResult(ir) => {
                // Java: only accept if apothecaryMode matches
                let mode_matches = self.apothecary_mode
                    .map(|m| m == ir.injury_context.apothecary_mode)
                    .unwrap_or(false);
                if mode_matches {
                    self.injury_result = Some(*ir.clone());
                }
                mode_matches
            }
            StepParameter::UsingPilingOn(using) => {
                // Java: DEFENDER mode + !usingPilingOn → suppress report
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

        let status = self.injury_result.as_ref()
            .map(|ir| ir.injury_context.apothecary_status);

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
                    // Java: setApothecaryStatus(WAIT_FOR_APOTHECARY_USE)
                    if let Some(ref mut ir) = self.injury_result {
                        ir.injury_context.apothecary_status = ApothecaryStatus::WaitForApothecaryUse;
                    }
                    // client-only: DialogUseApothecaryParameter — headless auto-accepts
                    do_next_step = false;
                    outcome = StepOutcome::cont();
                }
                ApothecaryStatus::UseApothecary => {
                    // Java: rollApothecary() → if choice dialog shown → WAIT; else RESULT_CHOICE
                    let (choice_shown, apo_roll_event) = self.roll_apothecary(game, rng);
                    if let Some(ev) = apo_roll_event { outcome = outcome.with_event(ev); }
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
            // Java: fInjuryResult.applyTo(this) — apply injury outcome to field model
            let side_events = if let Some(ref ir) = self.injury_result {
                ir.apply_to(game);
                // Java: UtilServerInjury.handleInjurySideEffects(this, fInjuryResult)
                crate::step::util_server_injury::handle_injury_side_effects(game, ir)
                // headless: raise-dead path inside handle_injury_side_effects not yet ported
            } else {
                vec![]
            };
            let mut out = StepOutcome::next();
            for ev in side_events { out = out.with_event(ev); }
            outcome = out;
        }

        outcome
    }

    /// Java: rollApothecary() — roll new casualty, compare, show choice dialog if needed.
    /// Returns true if a choice dialog was shown (caller must wait), false otherwise.
    fn roll_apothecary(&mut self, game: &mut Game, rng: &mut GameRng) -> (bool, Option<GameEvent>) {
        // Java: useApothecary() — decrement the correct team's apothecary count
        let defender_on_home_team = self.injury_result.as_ref()
            .and_then(|ir| ir.injury_context.defender_id.as_deref())
            .map(|id| game.team_home.has_player(id))
            .unwrap_or(false);
        if defender_on_home_team {
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
            // Java: addReport(new ReportApothecaryRoll(defender, casualtyRoll, newState, newSI, origSI, mods))
            let apo_event = GameEvent::ApothecaryRoll {
                player_id,
                roll: Some(roll),
                new_state: Some(base as u16),
                new_serious_injury: None,
            };
            // client-only: DialogApothecaryChoiceParameter — headless auto-selects first result
            (false, Some(apo_event))
        } else {
            // Java: cure to STUNNED (if was KO and canApoKoIntoStun) or RESERVE
            self.cure_poison(game);
            // Java: setPlayerState(defender, STUNNED or RESERVE) — apply cured state
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
            let choice_event = GameEvent::ApothecaryChoice {
                player_id,
                healed: true,
            };
            (false, Some(choice_event))
        }
    }

    /// Java: curePoison() — remove POISONED CardEffect based on mode.
    fn cure_poison(&self, game: &mut Game) {
        use ffb_model::enums::CardEffect;
        let player_id = self.injury_result.as_ref()
            .and_then(|ir| ir.injury_context.defender_id.clone());
        let player_id = match player_id {
            Some(id) => id,
            None => return,
        };
        let mode = match &self.apothecary_mode {
            Some(m) => *m,
            None => return,
        };
        let should_cure = matches!(mode, ApothecaryMode::Defender) && self.defender_poisoned
            || matches!(mode, ApothecaryMode::Attacker) && self.attacker_poisoned;
        if should_cure {
            game.field_model.remove_card_effect(&player_id, CardEffect::Poisoned);
        }
    }

    /// Java: handleApothecaryChoice(pPlayerState, pSeriousInjury).
    fn handle_apothecary_choice(&mut self, player_state: ffb_model::enums::PlayerState, serious_injury: Option<SeriousInjuryKind>) {
        if let Some(ref mut ir) = self.injury_result {
            use ffb_model::enums::{PS_BADLY_HURT, PS_RESERVE, PlayerState};
            if player_state.base() == PS_BADLY_HURT {
                // Java: setInjury(RESERVE), setSeriousInjury(null)
                ir.injury_context.injury = Some(PlayerState::new(PS_RESERVE));
                ir.injury_context.serious_injury = None;
            } else {
                // Java: setInjury(pPlayerState), setSeriousInjury(pSeriousInjury)
                ir.injury_context.injury = Some(player_state);
                ir.injury_context.serious_injury = serious_injury;
            }
            ir.injury_context.apothecary_status = ApothecaryStatus::ResultChoice;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, ApothecaryMode};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn no_injury_result_returns_next() {
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn apothecary_mode_parameter_accepted() {
        let mut step = StepApothecary::new();
        let accepted = step.set_parameter(&StepParameter::ApothecaryMode(ApothecaryMode::Attacker));
        assert!(accepted);
        assert_eq!(step.apothecary_mode, Some(ApothecaryMode::Attacker));
    }

    #[test]
    fn defender_poisoned_only_accepted_in_defender_mode() {
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        let ok = step.set_parameter(&StepParameter::DefenderPoisoned(true));
        assert!(ok);
        assert!(step.defender_poisoned);

        let mut step2 = StepApothecary::new();
        step2.apothecary_mode = Some(ApothecaryMode::Attacker);
        let not_ok = step2.set_parameter(&StepParameter::DefenderPoisoned(true));
        assert!(!not_ok);
    }

    #[test]
    fn attacker_poisoned_only_accepted_in_attacker_mode() {
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Attacker);
        let ok = step.set_parameter(&StepParameter::AttackerPoisoned(true));
        assert!(ok);
        assert!(step.attacker_poisoned);
    }

    #[test]
    fn using_piling_on_false_suppresses_report_in_defender_mode() {
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        step.show_report = true;
        let accepted = step.set_parameter(&StepParameter::UsingPilingOn(false));
        assert!(accepted);
        assert!(!step.show_report);
    }

    #[test]
    fn using_piling_on_not_accepted_in_attacker_mode() {
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Attacker);
        let accepted = step.set_parameter(&StepParameter::UsingPilingOn(false));
        assert!(!accepted);
    }

    #[test]
    fn default_show_report_is_true() {
        let step = StepApothecary::new();
        assert!(step.show_report);
    }

    fn make_injury_result(mode: ApothecaryMode, base: u32) -> InjuryResult {
        use ffb_model::enums::{PlayerState, ApothecaryStatus};
        let mut ir = InjuryResult::new(mode);
        ir.injury_context.apothecary_status = ApothecaryStatus::UseApothecary;
        ir.injury_context.injury = Some(PlayerState::new(base));
        ir.injury_context.defender_id = Some("def1".into());
        ir
    }

    #[test]
    fn apothecary_choice_badly_hurt_cures_to_reserve() {
        use ffb_model::enums::{PS_BADLY_HURT, PS_RESERVE, ApothecaryStatus};
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        step.injury_result = Some(make_injury_result(ApothecaryMode::Defender, PS_BADLY_HURT));
        let mut game = make_game();

        let action = Action::ApothecaryChoice { player_state: PS_BADLY_HURT, serious_injury: None };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));

        let ir = step.injury_result.as_ref().unwrap();
        assert_eq!(ir.injury_context.injury.unwrap().base(), PS_RESERVE);
        assert_eq!(ir.injury_context.apothecary_status, ApothecaryStatus::ResultChoice);
    }

    #[test]
    fn apothecary_choice_serious_injury_sets_state_and_injury() {
        use ffb_model::enums::{PS_BADLY_HURT, ApothecaryStatus, SeriousInjuryKind};
        // Use a state that is NOT BADLY_HURT so the else branch runs
        let non_badly_hurt = 0x03u32; // e.g. CASUALTY base
        let mut step = StepApothecary::new();
        step.apothecary_mode = Some(ApothecaryMode::Defender);
        step.injury_result = Some(make_injury_result(ApothecaryMode::Defender, PS_BADLY_HURT));
        let mut game = make_game();

        let action = Action::ApothecaryChoice {
            player_state: non_badly_hurt,
            serious_injury: Some("BrokenRibs".into()),
        };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));

        let ir = step.injury_result.as_ref().unwrap();
        assert_eq!(ir.injury_context.injury.unwrap().id(), non_badly_hurt);
        assert_eq!(ir.injury_context.serious_injury, Some(SeriousInjuryKind::BrokenRibs));
        assert_eq!(ir.injury_context.apothecary_status, ApothecaryStatus::ResultChoice);
    }

    #[test]
    fn apothecary_choice_no_injury_result_is_noop() {
        let mut step = StepApothecary::new();
        let mut game = make_game();
        let action = Action::ApothecaryChoice { player_state: 0, serious_injury: None };
        // Should not panic when injury_result is None
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(step.injury_result.is_none());
    }
}
