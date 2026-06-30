use ffb_model::enums::{ApothecaryMode, ApothecaryStatus};
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
            // Java: CLIENT_APOTHECARY_CHOICE → handleApothecaryChoice
            // In Rust, handled via Acknowledge (player accepted the result)
            // TODO: dedicated ApothecaryChoice action variant when added to Action enum
            // Java: CLIENT_USE_INDUCEMENT with REGENERATION usage
            // BB2016: no Igor / Mortuary Assistant — deliberately no-op here.
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

        // Java: UtilServerDialog.hideDialog(getGameState())
        // TODO: dialog hiding (no-op in headless)

        let status = self.injury_result.as_ref()
            .map(|ir| ir.injury_context.apothecary_status);

        let mut do_next_step = true;
        let mut outcome = StepOutcome::next();

        if let Some(status) = status {
            match status {
                ApothecaryStatus::DoRequest => {
                    // Java: if (fShowReport) fInjuryResult.report(this)
                    // Java: showDialog(DialogUseApothecaryParameter)
                    // Java: setApothecaryStatus(WAIT_FOR_APOTHECARY_USE)
                    if let Some(ref mut ir) = self.injury_result {
                        ir.injury_context.apothecary_status = ApothecaryStatus::WaitForApothecaryUse;
                    }
                    // TODO: show dialog DialogUseApothecaryParameter
                    do_next_step = false;
                    outcome = StepOutcome::cont();
                }
                ApothecaryStatus::UseApothecary => {
                    // Java: rollApothecary() → if choice dialog shown → WAIT; else RESULT_CHOICE
                    let choice_shown = self.roll_apothecary(game, rng);
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
                    // Java: addReport(ReportApothecaryRoll(defenderId, null, null, null))
                    // TODO: emit ApothecaryRoll event
                }
                ApothecaryStatus::NoApothecary => {
                    // Java: if (fShowReport) fInjuryResult.report(this)
                    // TODO: emit injury report
                }
                _ => {}
            }
        }

        if do_next_step {
            // Java: fInjuryResult.applyTo(this) — apply injury to field model
            // TODO: InjuryResult.apply_to — full injury side-effects
            // Java: handleInjurySideEffects(this, fInjuryResult)
            // TODO: UtilServerInjury.handleInjurySideEffects
            let _ = game;
            outcome = StepOutcome::next();
        }

        outcome
    }

    /// Java: rollApothecary() — roll new casualty, compare, show choice dialog if needed.
    /// Returns true if a choice dialog was shown (caller must wait), false otherwise.
    fn roll_apothecary(&mut self, game: &mut Game, rng: &mut GameRng) -> bool {
        // Java: useApothecary() for the correct team
        // TODO: game.getTurnDataHome/Away().useApothecary()

        let ir = match self.injury_result.as_ref() {
            Some(ir) => ir,
            None => return false,
        };

        use ffb_model::enums::{PS_BADLY_HURT, PS_KNOCKED_OUT};
        let base = ir.injury_context.injury.map(|ps| ps.base()).unwrap_or(0);
        // Java: apothecaryChoice = (base != BADLY_HURT && base != KNOCKED_OUT)
        let apothecary_choice = base != PS_BADLY_HURT && base != PS_KNOCKED_OUT;

        if apothecary_choice {
            // Java: roll new casualty, show DialogApothecaryChoiceParameter
            let _new_roll = rng.d6(); // TODO: rollCasualty uses 2d6
            // TODO: emit ApothecaryRoll event with new result
            // TODO: if new result also not BADLY_HURT → show choice dialog → return true
            // Approximation: always treat as BADLY_HURT (no choice needed)
            false
        } else {
            // Java: cure to STUNNED (if was KO and canApoKoIntoStun) or RESERVE
            self.cure_poison(game);
            // TODO: apply new player state
            false
        }
    }

    /// Java: curePoison() — remove POISONED CardEffect based on mode.
    fn cure_poison(&self, _game: &mut Game) {
        // TODO: game.getFieldModel().removeCardEffect(player, CardEffect.POISONED)
        // when card effects are implemented in ffb-model
    }

    /// Java: handleApothecaryChoice(pPlayerState, pSeriousInjury).
    fn handle_apothecary_choice(&mut self, player_state: ffb_model::enums::PlayerState, _serious_injury: Option<String>) {
        if let Some(ref mut ir) = self.injury_result {
            use ffb_model::enums::PS_BADLY_HURT;
            if player_state.base() == PS_BADLY_HURT {
                // Java: setInjury(RESERVE), setSeriousInjury(null)
                // TODO: apply reserve state to injury context
                ir.injury_context.serious_injury = None;
            } else {
                // Java: setInjury(pPlayerState), setSeriousInjury(pSeriousInjury)
                ir.injury_context.injury = Some(player_state);
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
}
