use ffb_model::enums::TurnMode;
use ffb_model::events::GameEvent;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::model::SpecialEffect;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::{Action, WizardSpellChoice};
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2016::special_effect::{SpecialEffect as SpecialEffectGenerator, SpecialEffectParams};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepWizard.
///
/// Handles the Wizard inducement step. Shows a spell-selection dialog,
/// then uses the chosen spell and pushes SpecialEffect sequences for
/// affected players.
///
/// Init: optional HOME_TEAM.
/// Handles: CLIENT_WIZARD_SPELL.
///   - null spell → cancel (fEndInducement = true)
///   - spell + coordinate → cast (use inducement, collect targets, push sequences)
///
/// Spells:
///   - ZAP: 1 target (at coordinate)
///   - LIGHTNING: 1 target (at coordinate)
///   - FIREBALL: 3×3 area (coordinate + adjacent)
pub struct StepWizard {
    /// Java: fWizardSpell
    pub wizard_spell: Option<SpecialEffect>,
    /// Java: fTargetCoordinate
    pub target_coordinate: Option<FieldCoordinate>,
    /// Java: fEndInducement
    pub end_inducement: bool,
    /// Java: fOldTurnMode
    pub old_turn_mode: Option<TurnMode>,
    /// Java: fHomeTeam (optional init, default false)
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
}

impl Default for StepWizard {
    fn default() -> Self { Self::new() }
}

impl Step for StepWizard {
    fn id(&self) -> StepId { StepId::Wizard }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_WIZARD_SPELL
        if let Action::WizardSpell { spell, coord } = action {
            match spell {
                // Java: if (wizardSpellCommand.getWizardSpell() == null) → fEndInducement = true
                // Note: in Rust the spell type doesn't carry null — we use a dedicated Cancel action
                // For WizardSpellChoice, treat any valid spell as cast
                WizardSpellChoice::Lightning => {
                    self.wizard_spell = Some(SpecialEffect::LIGHTNING);
                    self.target_coordinate = Some(self.transform_coord(*coord, game));
                }
                WizardSpellChoice::Fireball => {
                    self.wizard_spell = Some(SpecialEffect::FIREBALL);
                    self.target_coordinate = Some(self.transform_coord(*coord, game));
                }
            }
        }
        // Java: if (wizardSpellCommand.getWizardSpell() == null) → cancel
        // Handled via Acknowledge (cancel = send Acknowledge to decline a wizard dialog)
        if let Action::Acknowledge = action {
            self.end_inducement = true;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::HomeTeam(v) => {
                self.home_team = *v;
                true
            }
            _ => false,
        }
    }
}

impl StepWizard {
    /// Java: if (game.isHomePlaying()) targetCoord else targetCoord.transform()
    fn transform_coord(&self, coord: FieldCoordinate, game: &Game) -> FieldCoordinate {
        if game.home_playing {
            coord
        } else {
            coord.transform()
        }
    }

    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(getGameState())

        if self.end_inducement {
            // Java: cancel spellcasting — restore old turn mode if set
            if let Some(old_mode) = self.old_turn_mode {
                game.turn_mode = old_mode;
            }
            return StepOutcome::next();
        }

        if let (Some(spell), Some(coord)) = (self.wizard_spell, self.target_coordinate) {
            if game.turn_mode == TurnMode::Wizard {
                // Restore old turn mode
                if let Some(old_mode) = self.old_turn_mode {
                    game.turn_mode = old_mode;
                }
                // Java: UtilServerInducementUse.useInducement — mark wizard inducement as used
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
                let team_id = if self.home_team { game.team_home.id.clone() } else { game.team_away.id.clone() };
                // Java: getResult().addReport(new ReportWizardUse(team.getId(), fWizardSpell))
                {
                    use ffb_model::report::report_wizard_use::ReportWizardUse;
                    game.report_list.add(ReportWizardUse::new(team_id.clone(), spell));
                }
                let wizard_event = GameEvent::WizardUse {
                    team_id,
                    spell: format!("{:?}", spell),
                    coord: Some(self.transform_coord(coord, game)),
                };
                // Collect affected player IDs: ZAP/LIGHTNING → player at coord; FIREBALL → 3×3 area.
                // Java: addToAffectedPlayers() filters out PRONE and STUNNED players for
                // LIGHTNING/FIREBALL (ZAP adds directly without this filter, but ZAP is
                // not reachable through WizardSpellChoice in this translation).
                let spell_key = format!("{:?}", spell);
                let affected: Vec<String> = {
                    let coords: Vec<FieldCoordinate> = if spell == SpecialEffect::FIREBALL {
                        let mut coords = vec![coord];
                        coords.extend(game.field_model.adjacent_on_pitch(coord));
                        coords
                    } else {
                        vec![coord]
                    };
                    coords.iter()
                        .filter_map(|c| game.field_model.player_at(*c).cloned())
                        .filter(|id| {
                            game.field_model.player_state(id)
                                .map(|s| !s.is_prone() && !s.is_stunned())
                                .unwrap_or(false)
                        })
                        .collect()
                };
                let mut out = StepOutcome::next().with_event(wizard_event);
                for player_id in affected {
                    let seq = SpecialEffectGenerator::build_sequence(&SpecialEffectParams {
                        special_effect: Some(spell_key.clone()),
                        player_id: Some(player_id),
                        roll_for_effect: true,
                    });
                    out = out.push_seq(seq);
                }
                return out;
            }
        }

        // Java: else branch — set WIZARD turn mode, show dialog
        if game.turn_mode != TurnMode::Wizard {
            self.old_turn_mode = Some(game.turn_mode);
        }
        game.turn_mode = TurnMode::Wizard;
        // client-only: DialogWizardSpellParameter — headless skips spell selection dialog
        StepOutcome::cont()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{PlayerState, Rules, PS_PRONE, PS_STANDING, PS_STUNNED};
    use ffb_model::model::player::Player;
    use ffb_model::report::report_id::ReportId;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn step_id_is_wizard() {
        let step = StepWizard::new();
        assert_eq!(step.id(), StepId::Wizard);
    }

    #[test]
    fn home_team_parameter_accepted() {
        let mut step = StepWizard::new();
        let ok = step.set_parameter(&StepParameter::HomeTeam(true));
        assert!(ok);
        assert!(step.home_team);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepWizard::new();
        let ok = step.set_parameter(&StepParameter::EndTurn(true));
        assert!(!ok);
    }

    #[test]
    fn start_sets_wizard_turn_mode_and_returns_cont() {
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
    fn acknowledge_cancels_and_restores_turn_mode() {
        let mut step = StepWizard::new();
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        // First call: sets Wizard mode
        step.start(&mut game, &mut GameRng::new(0));
        // Acknowledge: cancel
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // Old turn mode restored
        assert_eq!(game.turn_mode, TurnMode::Regular);
    }

    #[test]
    fn wizard_spell_command_records_spell_and_coord() {
        let mut step = StepWizard::new();
        let mut game = make_game();
        game.turn_mode = TurnMode::Wizard;
        game.home_playing = true;
        let coord = FieldCoordinate::new(12, 7);
        step.handle_command(
            &Action::WizardSpell { spell: WizardSpellChoice::Lightning, coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.wizard_spell, Some(SpecialEffect::LIGHTNING));
        assert_eq!(step.target_coordinate, Some(coord));
    }

    #[test]
    fn fireball_spell_sets_fireball_effect() {
        let mut step = StepWizard::new();
        let mut game = make_game();
        game.turn_mode = TurnMode::Wizard;
        game.home_playing = true;
        let coord = FieldCoordinate::new(5, 5);
        step.handle_command(
            &Action::WizardSpell { spell: WizardSpellChoice::Fireball, coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.wizard_spell, Some(SpecialEffect::FIREBALL));
    }

    #[test]
    fn lightning_spell_emits_wizard_use_report() {
        // Java: getResult().addReport(new ReportWizardUse(team.getId(), fWizardSpell))
        // Verify ReportWizardUse is added when a spell is cast in WIZARD turn mode.
        let mut step = StepWizard::new();
        let mut game = make_game();
        game.turn_mode = TurnMode::Wizard;
        game.home_playing = true;
        let coord = FieldCoordinate::new(8, 4);
        step.handle_command(
            &Action::WizardSpell { spell: WizardSpellChoice::Lightning, coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(game.report_list.has_report(ReportId::WIZARD_USE), "ReportWizardUse must be emitted on spell cast");
    }

    #[test]
    fn fireball_spell_emits_wizard_use_report() {
        // Java: getResult().addReport(new ReportWizardUse(team.getId(), fWizardSpell))
        // Verify ReportWizardUse is added for fireball spells as well.
        let mut step = StepWizard::new();
        let mut game = make_game();
        game.turn_mode = TurnMode::Wizard;
        game.home_playing = true;
        let coord = FieldCoordinate::new(3, 6);
        step.handle_command(
            &Action::WizardSpell { spell: WizardSpellChoice::Fireball, coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(game.report_list.has_report(ReportId::WIZARD_USE), "ReportWizardUse must be emitted on fireball cast");
    }

    #[test]
    fn lightning_on_prone_player_does_not_push_sequence() {
        // Java: addToAffectedPlayers() excludes PRONE players from the affected list,
        // so no SpecialEffect sequence should be pushed for a prone target.
        let mut team = test_team("home", 0);
        team.players.push(Player { id: "p1".into(), name: "P1".into(), nr: 1, ..Default::default() });
        let away = test_team("away", 0);
        let mut game = Game::new(team, away, Rules::Bb2016);
        game.turn_mode = TurnMode::Wizard;
        game.home_playing = true;
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_PRONE));

        let mut step = StepWizard::new();
        let out = step.handle_command(
            &Action::WizardSpell { spell: WizardSpellChoice::Lightning, coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(out.pushes.is_empty(), "prone player should not be affected by Lightning");
    }

    #[test]
    fn fireball_on_stunned_player_does_not_push_sequence() {
        // Java: addToAffectedPlayers() excludes STUNNED players from the affected list.
        let mut team = test_team("home", 0);
        team.players.push(Player { id: "p1".into(), name: "P1".into(), nr: 1, ..Default::default() });
        let away = test_team("away", 0);
        let mut game = Game::new(team, away, Rules::Bb2016);
        game.turn_mode = TurnMode::Wizard;
        game.home_playing = true;
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STUNNED));

        let mut step = StepWizard::new();
        let out = step.handle_command(
            &Action::WizardSpell { spell: WizardSpellChoice::Fireball, coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(out.pushes.is_empty(), "stunned player should not be affected by Fireball");
    }

    #[test]
    fn lightning_on_standing_player_pushes_sequence() {
        // Sanity check: a standing player at the target coordinate IS affected.
        let mut team = test_team("home", 0);
        team.players.push(Player { id: "p1".into(), name: "P1".into(), nr: 1, ..Default::default() });
        let away = test_team("away", 0);
        let mut game = Game::new(team, away, Rules::Bb2016);
        game.turn_mode = TurnMode::Wizard;
        game.home_playing = true;
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));

        let mut step = StepWizard::new();
        let out = step.handle_command(
            &Action::WizardSpell { spell: WizardSpellChoice::Lightning, coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(!out.pushes.is_empty(), "standing player should be affected by Lightning");
    }
}
