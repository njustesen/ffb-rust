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
///      - Push SpecialEffect sequences → next_step.
///   3. Otherwise: set turn_mode = WIZARD, show dialog → CONTINUE.
///
/// Java fields: `fWizardSpell`, `fTargetCoordinate`, `fEndInducement`,
///              `fOldTurnMode`, `fHomeTeam`.
///
/// Java: `StepWizard extends AbstractStep` (mixed, BB2020 + BB2025).
use ffb_model::enums::{Rules, TurnMode};
use ffb_model::events::GameEvent;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::model::special_effect::SpecialEffect;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::{Action, WizardSpellChoice};
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::bb2020::special_effect::{
    SpecialEffect as SeGen2020, SpecialEffectParams as SeParams2020,
};
use crate::step::generator::bb2025::special_effect::{
    SpecialEffect as SeGen2025, SpecialEffectParams as SeParams2025,
};
use crate::step::generator::sequence::SequenceStep;

/// Java: `StepWizard` (mixed, BB2020 + BB2025).
pub struct StepWizard {
    /// Java: `fWizardSpell`
    pub wizard_spell: Option<SpecialEffect>,
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
            // Java: cancel spellcasting — restore old turn mode
            if let Some(old) = self.old_turn_mode {
                game.turn_mode = old;
            }
            return StepOutcome::next();
        }

        if let (Some(spell), Some(coord)) = (self.wizard_spell, self.target_coordinate) {
            if game.turn_mode == TurnMode::Wizard {
                let spell_name = spell.get_name().to_string();
                let rules = game.rules;

                // Java: addReport(new ReportWizardUse(team.getId(), fWizardSpell))
                let team_id = if self.home_team {
                    game.team_home.id.clone()
                } else {
                    game.team_away.id.clone()
                };
                let wizard_event = GameEvent::WizardUse {
                    team_id,
                    spell: spell_name.clone(),
                    coord: Some(coord),
                };

                // Java: find matching inducement with SPELL usage, use it
                let inducement_id = if self.home_team {
                    game.turn_data_home.inducement_set.for_usage(Usage::SPELL).map(|s| s.to_string())
                } else {
                    game.turn_data_away.inducement_set.for_usage(Usage::SPELL).map(|s| s.to_string())
                };
                if let Some(type_id) = inducement_id {
                    if self.home_team {
                        game.turn_data_home.inducement_set.use_one_of(&type_id);
                    } else {
                        game.turn_data_away.inducement_set.use_one_of(&type_id);
                    }
                }

                // Java: collect affected players
                // addToAffectedPlayers: skip prone or stunned
                let affected: Vec<String> = {
                    let coords: Vec<FieldCoordinate> = if spell == SpecialEffect::FIREBALL {
                        let mut v = vec![coord];
                        v.extend(game.field_model.adjacent_on_pitch(coord));
                        v
                    } else {
                        vec![coord]
                    };
                    coords.iter()
                        .filter_map(|c| game.field_model.player_at(*c))
                        .filter(|id| {
                            game.field_model.player_state(id)
                                .map(|s| !s.is_prone() && !s.is_stunned())
                                .unwrap_or(false)
                        })
                        .map(|id| id.to_string())
                        .collect()
                };

                // Restore old turn mode
                if let Some(old) = self.old_turn_mode {
                    game.turn_mode = old;
                }

                // Push SpecialEffect sequences for each affected player
                let mut out = StepOutcome::next().with_event(wizard_event);
                for player_id in affected {
                    let seq = build_spell_seq(rules, &spell_name, &player_id);
                    out = out.push_seq(seq);
                }
                return out;
            }
        }

        // Java: else — set WIZARD turn mode, show dialog
        if game.turn_mode != TurnMode::Wizard {
            self.old_turn_mode = Some(game.turn_mode);
        }
        game.turn_mode = TurnMode::Wizard;
        // DEFERRED(dialog): UtilServerDialog.showDialog(DialogWizardSpellParameter)
        StepOutcome::cont()
    }
}

/// Build the edition-specific SpecialEffect step sequence.
fn build_spell_seq(rules: Rules, spell_name: &str, player_id: &str) -> Vec<SequenceStep> {
    match rules {
        Rules::Bb2025 => SeGen2025::build_sequence(&SeParams2025 {
            special_effect_key: spell_name.to_string(),
            player_id: player_id.to_string(),
            roll_for_effect: true,
        }),
        _ => SeGen2020::build_sequence(&SeParams2020 {
            special_effect_key: spell_name.to_string(),
            player_id: player_id.to_string(),
            roll_for_effect: true,
        }),
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
        match action {
            Action::WizardSpell { spell, coord } => {
                self.wizard_spell = Some(match spell {
                    WizardSpellChoice::Lightning => SpecialEffect::LIGHTNING,
                    WizardSpellChoice::Fireball => SpecialEffect::FIREBALL,
                });
                self.target_coordinate = Some(if self.home_team { *coord } else { coord.transform() });
            }
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
    use ffb_model::enums::{PlayerState, Rules};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::util::rng::GameRng;
    use ffb_model::enums::{PS_STANDING, PS_PRONE, PS_STUNNED};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn make_bb2020_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
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
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert_eq!(game.turn_mode, TurnMode::Wizard);
    }

    #[test]
    fn start_saves_old_turn_mode() {
        let mut step = StepWizard::new();
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.old_turn_mode, Some(TurnMode::Regular));
    }

    #[test]
    fn end_inducement_restores_turn_mode_and_next_steps() {
        let mut step = StepWizard::new();
        step.old_turn_mode = Some(TurnMode::Regular);
        step.end_inducement = true;
        let mut game = make_game();
        game.turn_mode = TurnMode::Wizard;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Regular);
    }

    #[test]
    fn set_parameter_home_team() {
        let mut step = StepWizard::new();
        assert!(step.set_parameter(&StepParameter::HomeTeam(true)));
        assert!(step.home_team);
    }

    #[test]
    fn spell_and_target_in_wizard_mode_next_steps() {
        let mut step = StepWizard::new();
        step.wizard_spell = Some(SpecialEffect::LIGHTNING);
        step.target_coordinate = Some(FieldCoordinate::new(5, 5));
        step.old_turn_mode = Some(TurnMode::Regular);
        let mut game = make_game();
        game.turn_mode = TurnMode::Wizard;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::Regular);
    }

    #[test]
    fn lightning_on_standing_player_pushes_sequence() {
        let mut step = StepWizard::new();
        step.home_team = true;
        step.old_turn_mode = Some(TurnMode::Regular);
        step.wizard_spell = Some(SpecialEffect::LIGHTNING);
        let coord = FieldCoordinate::new(5, 5);
        step.target_coordinate = Some(coord);

        let mut game = make_game();
        game.turn_mode = TurnMode::Wizard;

        // Add a standing player at the target coordinate
        let mut team = test_team("home", 0);
        team.players.push(Player { id: "p1".into(), name: "P1".into(), nr: 1, ..Default::default() });
        game = Game::new(team, test_team("away", 0), Rules::Bb2025);
        game.turn_mode = TurnMode::Wizard;
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty(), "should push a sequence for the standing player");
    }

    #[test]
    fn lightning_on_prone_player_skips_sequence() {
        let mut step = StepWizard::new();
        step.home_team = true;
        step.old_turn_mode = Some(TurnMode::Regular);
        step.wizard_spell = Some(SpecialEffect::LIGHTNING);
        let coord = FieldCoordinate::new(5, 5);
        step.target_coordinate = Some(coord);

        let mut team = test_team("home", 0);
        team.players.push(Player { id: "p1".into(), name: "P1".into(), nr: 1, ..Default::default() });
        let mut game = Game::new(team, test_team("away", 0), Rules::Bb2025);
        game.turn_mode = TurnMode::Wizard;
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_PRONE));

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.is_empty(), "prone player should not be affected");
    }

    #[test]
    fn bb2020_rules_also_pushes_sequence() {
        let mut step = StepWizard::new();
        step.home_team = true;
        step.old_turn_mode = Some(TurnMode::Regular);
        step.wizard_spell = Some(SpecialEffect::LIGHTNING);
        let coord = FieldCoordinate::new(5, 5);
        step.target_coordinate = Some(coord);

        let mut team = test_team("home", 0);
        team.players.push(Player { id: "p1".into(), name: "P1".into(), nr: 1, ..Default::default() });
        let mut game = Game::new(team, test_team("away", 0), Rules::Bb2020);
        game.turn_mode = TurnMode::Wizard;
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.pushes.is_empty());
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepWizard::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn stunned_player_not_affected_by_fireball() {
        let mut step = StepWizard::new();
        step.home_team = true;
        step.old_turn_mode = Some(TurnMode::Regular);
        step.wizard_spell = Some(SpecialEffect::FIREBALL);
        let coord = FieldCoordinate::new(5, 5);
        step.target_coordinate = Some(coord);

        let mut team = test_team("home", 0);
        team.players.push(Player { id: "p1".into(), name: "P1".into(), nr: 1, ..Default::default() });
        let mut game = Game::new(team, test_team("away", 0), Rules::Bb2025);
        game.turn_mode = TurnMode::Wizard;
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STUNNED));

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.pushes.is_empty(), "stunned player should not be affected");
    }
}
