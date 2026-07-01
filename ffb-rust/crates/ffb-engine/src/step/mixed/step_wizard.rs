/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepWizard`.
///
/// Handles the wizard inducement (ZAP / LIGHTNING / FIREBALL spells).
/// Needs `HOME_TEAM` initialisation parameter.
///
/// Java logic (executeStep):
///   1. If `end_inducement` (cancelled): restore old_turn_mode → next_step.
///   2. If spell + target set AND turn_mode == WIZARD:
///      - Mark inducement used for the matching inducementType.
///      - Collect affected players (1 for ZAP/LIGHTNING, 3x3 for FIREBALL).
///      - Restore old_turn_mode.
///      - Add blood spot, push SpecialEffect sequences → next_step.
///   3. Otherwise: set turn_mode = WIZARD, show dialog → CONTINUE.
///
/// SpecialEffect generator and InducementSet not yet fully ported — stubbed.
///
/// Java fields: `fWizardSpell`, `fTargetCoordinate`, `fEndInducement`,
///              `fOldTurnMode`, `fHomeTeam`.
///
/// Java: `StepWizard extends AbstractStep` (mixed, BB2020 + BB2025).
use ffb_model::enums::TurnMode;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::{Action, WizardSpellChoice};
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Mirrors Java `SpecialEffect` (ZAP / LIGHTNING / FIREBALL).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardSpell {
    Zap,
    Lightning,
    Fireball,
}

/// Java: `StepWizard` (mixed, BB2020 + BB2025).
pub struct StepWizard {
    /// Java: `fWizardSpell`
    pub wizard_spell: Option<WizardSpell>,
    /// Java: `fTargetCoordinate`
    pub target_coordinate: Option<FieldCoordinate>,
    /// Java: `fEndInducement`
    pub end_inducement: bool,
    /// Java: `fOldTurnMode`
    pub old_turn_mode: Option<TurnMode>,
    /// Java: `fHomeTeam`
    pub home_team: bool,
}

impl StepWizard {
    pub fn new() -> Self {
        Self {
            wizard_spell: None,
            target_coordinate: None,
            end_inducement: false,
            old_turn_mode: None,
            home_team: false,
        }
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(getGameState())

        if self.end_inducement {
            // Java: cancelled spellcasting — restore old turn mode
            if let Some(old) = self.old_turn_mode {
                game.turn_mode = old;
            }
            return StepOutcome::next();
        }

        if self.wizard_spell.is_some() && self.target_coordinate.is_some()
            && game.turn_mode == TurnMode::Wizard
        {
            // Java: report + mark inducement used + collect affected players
            //   + restore old_turn_mode + push SpecialEffect sequences
            // DEFERRED(InducementSet/SpecialEffect port): mark wizard used, push sequences
            if let Some(old) = self.old_turn_mode {
                game.turn_mode = old;
            }
            return StepOutcome::next();
        }

        // Java: show dialog — set turn mode to WIZARD
        if game.turn_mode != TurnMode::Wizard {
            self.old_turn_mode = Some(game.turn_mode);
        }
        game.turn_mode = TurnMode::Wizard;
        // Java: UtilServerDialog.showDialog(… DialogWizardSpellParameter …)
        StepOutcome::cont()
    }
}

impl Default for StepWizard {
    fn default() -> Self { Self::new() }
}

impl Step for StepWizard {
    fn id(&self) -> StepId { StepId::Wizard }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_WIZARD_SPELL
        match action {
            Action::WizardSpell { spell, coord } => {
                self.wizard_spell = Some(match spell {
                    WizardSpellChoice::Lightning => WizardSpell::Lightning,
                    WizardSpellChoice::Fireball => WizardSpell::Fireball,
                });
                self.target_coordinate = Some(if self.home_team {
                    *coord
                } else {
                    coord.transform()
                });
            }
            // Cancel: wizard_spell stays None, end_inducement = true
            Action::Acknowledge => {
                self.end_inducement = true;
            }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::HomeTeam(v) => { self.home_team = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_wizard() {
        assert_eq!(StepWizard::new().id(), StepId::Wizard);
    }

    #[test]
    fn start_sets_wizard_turn_mode_and_continues() {
        let mut step = StepWizard::new();
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        // Should show dialog = Continue
        assert_eq!(out.action, StepAction::Continue);
        assert_eq!(game.turn_mode, TurnMode::Wizard);
    }

    #[test]
    fn end_inducement_restores_turn_mode_and_next_steps() {
        let mut step = StepWizard::new();
        step.old_turn_mode = Some(TurnMode::Regular);
        step.end_inducement = true;
        let mut game = make_game();
        game.turn_mode = TurnMode::Wizard;
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Regular);
    }

    #[test]
    fn set_parameter_home_team() {
        let mut step = StepWizard::new();
        let accepted = step.set_parameter(&StepParameter::HomeTeam(true));
        assert!(accepted);
        assert!(step.home_team);
    }

    #[test]
    fn spell_and_target_in_wizard_mode_transitions_to_next() {
        let mut step = StepWizard::new();
        step.wizard_spell = Some(WizardSpell::Lightning);
        step.target_coordinate = Some(FieldCoordinate::new(5, 5));
        step.old_turn_mode = Some(TurnMode::Regular);
        let mut game = make_game();
        game.turn_mode = TurnMode::Wizard;
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        // old_turn_mode restored
        assert_eq!(game.turn_mode, TurnMode::Regular);
    }
}
