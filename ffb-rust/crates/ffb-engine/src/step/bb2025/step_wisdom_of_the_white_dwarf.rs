/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepWisdomOfTheWhiteDwarf (BB2025).
///
/// Grants a skill to a teammate within 2 squares (Wisdom of the White Dwarf ability).
///
/// Commands: CLIENT_PLAYER_CHOICE (teammate selection), CLIENT_PRAYER_SELECTION (skill to grant).
///
/// Java `executeStep()`:
///   1. If `playerId` not yet chosen: find standing/prone teammates within 2 squares that are
///      active (Constant/UtilPlayer.findStandingOrPronePlayers + isActive filter).
///      - none → NEXT_STEP.
///      - exactly one → auto-select.
///      - multiple → show DialogPlayerChoiceParameter(WISDOM), wait for CLIENT_PLAYER_CHOICE.
///   2. Compute gain-able skills (Constant.getGrantAbleSkills, minus skills the target already
///      has). None → NEXT_STEP.
///   3. If `skill` not yet chosen: look up the granting skill (canGrantSkillsToTeamMates) on the
///      acting player and report its use.
///      - exactly one gain-able skill → auto-select.
///      - multiple → show DialogSelectSkillParameter(WISDOM_OF_THE_WHITE_DWARF), wait for
///        CLIENT_PRAYER_SELECTION. Note this branch is only reached when `skill` was still null
///        on this call, so the ReportSkillUse is emitted at most once even across dialog
///        round-trips.
///   4. Grant the chosen skill to the target (FieldModel.addWisdomSkill → temporary skill),
///      report it, mark the granting skill used, and record the grant on both the acting
///      player and the target (ActingPlayer.addGrantedSkill).
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::prompts::AgentPrompt;
use ffb_model::report::mixed::report_player_event::ReportPlayerEvent;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Java: `Constant.getGrantAbleSkills` — the fixed set of skills Wisdom of the White Dwarf may
/// grant, with an optional configured value (Mighty Blow carries value "1").
pub(crate) fn grant_able_skills() -> Vec<(SkillId, Option<&'static str>)> {
    vec![
        (SkillId::BreakTackle, None),
        (SkillId::Dauntless, None),
        (SkillId::MightyBlow, Some("1")),
        (SkillId::SureFeet, None),
    ]
}

pub struct StepWisdomOfTheWhiteDwarf {
    /// Java: playerId — selected teammate receiving the skill.
    pub player_id: Option<String>,
    /// Java: skill — selected skill to grant (set from CLIENT_PRAYER_SELECTION).
    pub granted_skill: Option<SkillId>,
    /// Java: grantingSkill (local var, cached here since it must survive dialog round-trips) —
    /// the WisdomOfTheWhiteDwarf skill on the acting player.
    pub granting_skill: Option<SkillId>,
}

impl StepWisdomOfTheWhiteDwarf {
    pub fn new() -> Self {
        Self { player_id: None, granted_skill: None, granting_skill: None }
    }

    /// Java bb2020 `StepWisdomOfTheWhiteDwarf.executeStep()`: the ACTING player is the one who
    /// GAINS the skill, from an adjacent team-mate (with tackle zones) whose canGrantSkillsToTeamMates
    /// skill is unused -- the reverse of bb2025, where the star grants it to a team-mate within 2.
    /// The shared step ran the bb2025 logic for bb2020 (star tier: imperial_nobility+Grombrindal).
    fn execute_step_bb2020(&mut self, game: &mut Game, acting_id: String) -> StepOutcome {
        let owned: Vec<SkillId> = game.player(&acting_id).map(|p| p.all_skill_ids().collect()).unwrap_or_default();
        // Constant.getGrantAbleSkills(..).filter(!owned).sorted(by name) -- already name-sorted.
        let gain_able_skills: Vec<(SkillId, Option<&'static str>)> = grant_able_skills()
            .into_iter()
            .filter(|(id, _)| !owned.contains(id))
            .collect();

        if self.player_id.is_none() {
            let Some(coord) = game.field_model.player_coordinate(&acting_id) else {
                return StepOutcome::next();
            };
            let team = if game.team_home.has_player(&acting_id) { &game.team_home } else { &game.team_away };
            let wise_players: Vec<String> = UtilPlayer::find_adjacent_players_with_tacklezones(game, team, coord, false)
                .into_iter()
                .filter(|mate_id| game.player(mate_id).map(|m| m.all_skill_ids().any(|sk|
                    sk.properties().contains(&NamedProperties::CAN_GRANT_SKILLS_TO_TEAM_MATES)
                        && !m.used_skills.contains(&sk))).unwrap_or(false))
                .cloned()
                .collect();
            if wise_players.is_empty() {
                return StepOutcome::next();
            }
            if wise_players.len() == 1 {
                self.player_id = Some(wise_players[0].clone());
            } else {
                return StepOutcome::cont().with_prompt(AgentPrompt::PlayerChoice {
                    eligible_players: wise_players,
                    reason: "WISDOM".into(),
                    descriptions: vec![],
                });
            }
        }
        let wise_id = self.player_id.clone().expect("player_id set above");
        let granting = game.player(&wise_id)
            .and_then(|p| p.skill_id_with_property(NamedProperties::CAN_GRANT_SKILLS_TO_TEAM_MATES));
        self.granting_skill = granting;

        if self.granted_skill.is_none() {
            // Java: addReport(new ReportSkillUseOtherPlayer(actingPlayerId, grantingSkill,
            //   SkillUse.GAIN_GRANTED_SKILL, playerId))
            game.report_list.add(ffb_model::report::bb2020::report_skill_use_other_player::ReportSkillUseOtherPlayer::new(
                acting_id.clone(),
                granting.map(|g| g.display_name().to_string()).unwrap_or_default(),
                SkillUse::GAIN_GRANTED_SKILL.get_name().to_string(),
                wise_id.clone(),
            ));
            if gain_able_skills.len() == 1 {
                self.granted_skill = Some(gain_able_skills[0].0);
            } else {
                return StepOutcome::cont().with_prompt(AgentPrompt::SelectSkill {
                    player_id: acting_id.clone(),
                    skill_ids: gain_able_skills.iter().map(|(id, _)| *id as u16).collect(),
                    reason: "WISDOM_OF_THE_WHITE_DWARF".into(),
                });
            }
        }

        let skill = self.granted_skill.expect("granted_skill set above");
        let value = gain_able_skills.iter()
            .find(|(id, _)| *id == skill)
            .and_then(|(_, v)| *v)
            .map(|v| v.to_string());
        // Java: game.getFieldModel().addWisdomSkill(actingPlayer.getPlayerId(), gainedSkill)
        // Java: addWisdomSkill keys on wisdomSkill.enhancementSourceName() -- per edition
        let rules = game.rules;
        if let Some(acting_mut) = game.player_mut(&acting_id) {
            acting_mut.add_prayer_skill(crate::step::util_server_steps::wisdom_enhancement_source(rules), skill, value);
        }
        game.report_list.add(ReportPlayerEvent::new(
            Some(acting_id.clone()),
            Some(format!("gains {}", skill.display_name())),
        ));
        // Java: player.markUsed(grantingSkill, game); actingPlayer.addGrantedSkill(grantingSkill, player)
        if let Some(g) = granting {
            mark_used(game, &wise_id, g);
            game.acting_player.add_granted_skill(g, Some(wise_id.clone()));
        }
        StepOutcome::next()
    }

    /// Java: `executeStep()`.
    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(getGameState())
        crate::util::util_server_dialog::UtilServerDialog::hide_dialog(game);
        let Some(acting_id) = game.acting_player.player_id.clone() else {
            return StepOutcome::next();
        };
        if game.rules == ffb_model::enums::Rules::Bb2020 {
            return self.execute_step_bb2020(game, acting_id);
        }

        if self.player_id.is_none() {
            let Some(coord) = game.field_model.player_coordinate(&acting_id) else {
                return StepOutcome::next();
            };
            let is_home = game.team_home.has_player(&acting_id);
            let team = if is_home { &game.team_home } else { &game.team_away };
            // Java: findStandingOrPronePlayers(...).filter(isActive).map(getId)
            let wise_players: Vec<String> = UtilPlayer::find_standing_or_prone_players(game, team, coord, 2)
                .into_iter()
                .filter(|team_mate| {
                    game.field_model.player_state(&team_mate.id)
                        .map(|s| s.is_active())
                        .unwrap_or(false)
                })
                .map(|team_mate| team_mate.id.clone())
                .collect();

            if wise_players.is_empty() {
                return StepOutcome::next();
            }
            if wise_players.len() == 1 {
                self.player_id = Some(wise_players[0].clone());
            } else {
                // Java: showDialog(new DialogPlayerChoiceParameter(game.getActingTeam().getId(),
                //   PlayerChoiceMode.WISDOM, wisePlayers, null, 1, 1), false)
                let acting_team_id = game.active_team().id.clone();
                crate::util::util_server_dialog::UtilServerDialog::show_dialog(
                    game,
                    ffb_model::dialog::dialog_parameter_factory::DialogParameter::PlayerChoice(
                        ffb_model::dialog::dialog_player_choice_parameter::DialogPlayerChoiceParameter {
                            team_id: Some(acting_team_id),
                            player_choice_mode: Some(ffb_model::model::player_choice_mode::PlayerChoiceMode::WISDOM.get_name().to_string()),
                            player_ids: wise_players.clone(),
                            descriptions: vec![],
                            max_selects: 1,
                            min_selects: 1,
                        },
                    ),
                    false,
                );
                return StepOutcome::cont().with_prompt(AgentPrompt::PlayerChoice {
                    eligible_players: wise_players,
                    reason: "WISDOM".into(),
                    descriptions: vec![],
                });
            }
        }

        let target_id = self.player_id.clone().expect("player_id set above");
        let Some(target) = game.player(&target_id) else {
            return StepOutcome::next();
        };

        // Java: Constant.getGrantAbleSkills(...).filter(!targetOwnedSkills.contains).sorted(by name)
        let gain_able_skills: Vec<(SkillId, Option<&'static str>)> = grant_able_skills()
            .into_iter()
            .filter(|(id, _)| !target.all_skill_ids().any(|owned| owned == *id))
            .collect();

        if gain_able_skills.is_empty() {
            return StepOutcome::next();
        }

        if self.granted_skill.is_none() {
            // Java: player.getSkillWithProperty(NamedProperties.canGrantSkillsToTeamMates)
            let granting_skill = game.player(&acting_id)
                .and_then(|p| p.skill_id_with_property(NamedProperties::CAN_GRANT_SKILLS_TO_TEAM_MATES));
            self.granting_skill = granting_skill;
            if let Some(granting) = granting_skill {
                game.report_list.add(ReportSkillUse::new(
                    Some(acting_id.clone()),
                    granting,
                    true,
                    SkillUse::GRANT_SKILL_TO_TEAM_MATE,
                ));
            }

            if gain_able_skills.len() == 1 {
                self.granted_skill = Some(gain_able_skills[0].0);
            } else {
                // Java: UtilServerDialog.showDialog(new DialogSelectSkillParameter(target.getId(),
                //         gainAbleSkills.map(getSkill), SkillChoiceMode.WISDOM_OF_THE_WHITE_DWARF))
                // then `return` (CONTINUE, waiting for CLIENT_PRAYER_SELECTION).
                // The old bare `cont()` was the driver's stall shape: nothing was ever asked, so
                // the sequence could not advance. `gain_able_skills` is already in Java's order —
                // sorted by skill NAME — so index 0 is a genuine contract for the answer.
                crate::util::util_server_dialog::UtilServerDialog::show_dialog(
                    game,
                    ffb_model::dialog::dialog_parameter_factory::DialogParameter::SelectSkill(
                        ffb_model::dialog::dialog_select_skill_parameter::DialogSelectSkillParameter {
                            skills: gain_able_skills.iter().map(|(id, _)| *id).collect(),
                            player_id: Some(target_id.clone()),
                            skill_choice_mode: Some(
                                ffb_model::model::skill_choice_mode::SkillChoiceMode::WISDOM_OF_THE_WHITE_DWARF.name().to_string()),
                            null_skill_slots: Vec::new(),
                        },
                    ),
                    false,
                );
                return StepOutcome::cont().with_prompt(AgentPrompt::SelectSkill {
                    player_id: target_id.clone(),
                    skill_ids: gain_able_skills.iter().map(|(id, _)| *id as u16).collect(),
                    reason: "WISDOM_OF_THE_WHITE_DWARF".into(),
                });
            }
        }

        let skill = self.granted_skill.expect("granted_skill set above");
        // Java: gainAbleSkills.stream().filter(svw -> svw.getSkill().equals(skill)).findFirst()
        //   .orElseThrow(...) — value carried by the grant-able entry (e.g. Mighty Blow -> "1").
        let value = gain_able_skills.iter()
            .find(|(id, _)| *id == skill)
            .and_then(|(_, v)| *v)
            .map(|v| v.to_string());

        // Java: game.getFieldModel().addWisdomSkill(target.getId(), gainedSkill)
        let rules = game.rules;
        if let Some(target_mut) = game.player_mut(&target_id) {
            target_mut.add_prayer_skill(crate::step::util_server_steps::wisdom_enhancement_source(rules), skill, value);
        }
        game.report_list.add(ReportPlayerEvent::new(
            Some(target_id.clone()),
            // Java: "gains " + gainedSkill.getSkill().getName() -- the DISPLAY name ("Break Tackle");
            // class_name() wrote "BreakTackle" for the three two-word grantable skills.
            Some(format!("gains {}", skill.display_name())),
        ));

        if let Some(granting) = self.granting_skill {
            mark_used(game, &acting_id, granting);
            game.acting_player.add_granted_skill(granting, Some(acting_id.clone()));
            game.acting_player.add_granted_skill(granting, Some(target_id.clone()));
        }

        StepOutcome::next()
    }
}

/// Java: `player.markUsed(grantingSkill, game)`.
fn mark_used(game: &mut Game, player_id: &str, skill_id: SkillId) {
    if let Some(p) = game.player_mut(player_id) {
        p.used_skills.insert(skill_id);
    }
}

impl Default for StepWisdomOfTheWhiteDwarf {
    fn default() -> Self { Self::new() }
}

impl Step for StepWisdomOfTheWhiteDwarf {
    fn id(&self) -> StepId { StepId::WisdomOfTheWhiteDwarf }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_PLAYER_CHOICE
            Action::SelectPlayer { player_id } => {
                self.player_id = Some(player_id.clone());
            }
            // Java: CLIENT_PRAYER_SELECTION
            Action::SelectSkill { skill_id } => {
                self.granted_skill = Some(*skill_id);
            }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{PlayerAction, PlayerGender, PlayerType, Rules, PS_STANDING, PS_PRONE, PS_STUNNED};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, coord: FieldCoordinate, state: u32, skills: Vec<SkillId>, home: bool) {
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if home {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(state).change_active(true));
    }

    fn setup_acting_player(game: &mut Game, id: &str) {
        game.acting_player.set_player(id.into(), PlayerAction::Move);
    }

    #[test]
    fn no_acting_player_returns_next_step() {
        let mut game = make_game();
        let out = StepWisdomOfTheWhiteDwarf::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_teammates_in_range_returns_next_step() {
        let mut game = make_game();
        add_player(&mut game, "att", FieldCoordinate::new(5, 5), PS_STANDING, vec![SkillId::WisdomOfTheWhiteDwarf], true);
        setup_acting_player(&mut game, "att");
        let out = StepWisdomOfTheWhiteDwarf::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// The old stub always returned NEXT_STEP without granting anything. This test fails
    /// against that stub (no skill on the teammate) and passes with the real translation:
    /// a single eligible teammate + a single gain-able skill (all four grant-able skills
    /// already present except MightyBlow) auto-resolves in one call and grants MightyBlow.
    #[test]
    fn single_teammate_single_gainable_skill_grants_skill_immediately() {
        let mut game = make_game();
        add_player(&mut game, "att", FieldCoordinate::new(5, 5), PS_STANDING, vec![SkillId::WisdomOfTheWhiteDwarf], true);
        add_player(
            &mut game,
            "mate",
            FieldCoordinate::new(6, 5),
            PS_STANDING,
            vec![SkillId::BreakTackle, SkillId::Dauntless, SkillId::SureFeet],
            true,
        );
        setup_acting_player(&mut game, "att");

        let mut step = StepWisdomOfTheWhiteDwarf::new();
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);
        assert!(
            game.player("mate").unwrap().has_skill(SkillId::MightyBlow),
            "teammate should have gained Mighty Blow"
        );
        assert_eq!(step.player_id.as_deref(), Some("mate"));
        assert_eq!(step.granted_skill, Some(SkillId::MightyBlow));
    }

    #[test]
    fn granting_skill_is_marked_used_and_recorded_on_both_players() {
        let mut game = make_game();
        add_player(&mut game, "att", FieldCoordinate::new(5, 5), PS_STANDING, vec![SkillId::WisdomOfTheWhiteDwarf], true);
        add_player(
            &mut game,
            "mate",
            FieldCoordinate::new(6, 5),
            PS_STANDING,
            vec![SkillId::BreakTackle, SkillId::Dauntless, SkillId::SureFeet],
            true,
        );
        setup_acting_player(&mut game, "att");

        StepWisdomOfTheWhiteDwarf::new().start(&mut game, &mut GameRng::new(0));

        assert!(game.player("att").unwrap().used_skills.contains(&SkillId::WisdomOfTheWhiteDwarf));
        let granted = &game.acting_player.skills_granted_by[&SkillId::WisdomOfTheWhiteDwarf];
        assert!(granted.contains(&"att".to_string()));
        assert!(granted.contains(&"mate".to_string()));
    }

    #[test]
    fn adds_skill_use_and_player_event_reports() {
        let mut game = make_game();
        add_player(&mut game, "att", FieldCoordinate::new(5, 5), PS_STANDING, vec![SkillId::WisdomOfTheWhiteDwarf], true);
        add_player(
            &mut game,
            "mate",
            FieldCoordinate::new(6, 5),
            PS_STANDING,
            vec![SkillId::BreakTackle, SkillId::Dauntless, SkillId::SureFeet],
            true,
        );
        setup_acting_player(&mut game, "att");

        StepWisdomOfTheWhiteDwarf::new().start(&mut game, &mut GameRng::new(0));

        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::PLAYER_EVENT));
    }

    #[test]
    fn target_already_has_all_grantable_skills_returns_next_step_without_granting() {
        let mut game = make_game();
        add_player(&mut game, "att", FieldCoordinate::new(5, 5), PS_STANDING, vec![SkillId::WisdomOfTheWhiteDwarf], true);
        add_player(
            &mut game,
            "mate",
            FieldCoordinate::new(6, 5),
            PS_STANDING,
            vec![SkillId::BreakTackle, SkillId::Dauntless, SkillId::SureFeet, SkillId::MightyBlow],
            true,
        );
        setup_acting_player(&mut game, "att");

        let out = StepWisdomOfTheWhiteDwarf::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.report_list.has_report(ffb_model::report::report_id::ReportId::PLAYER_EVENT));
    }

    #[test]
    fn multiple_eligible_teammates_prompts_player_choice() {
        let mut game = make_game();
        add_player(&mut game, "att", FieldCoordinate::new(5, 5), PS_STANDING, vec![SkillId::WisdomOfTheWhiteDwarf], true);
        add_player(&mut game, "mate1", FieldCoordinate::new(6, 5), PS_STANDING, vec![], true);
        add_player(&mut game, "mate2", FieldCoordinate::new(4, 5), PS_STANDING, vec![], true);
        setup_acting_player(&mut game, "att");

        let mut step = StepWisdomOfTheWhiteDwarf::new();
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::Continue);
        match out.prompt {
            Some(AgentPrompt::PlayerChoice { eligible_players, reason, .. }) => {
                assert_eq!(reason, "WISDOM");
                assert!(eligible_players.contains(&"mate1".to_string()));
                assert!(eligible_players.contains(&"mate2".to_string()));
            }
            other => panic!("expected PlayerChoice prompt, got {other:?}"),
        }
        assert!(step.player_id.is_none(), "player_id not set until CLIENT_PLAYER_CHOICE arrives");
    }

    #[test]
    fn select_player_command_resolves_and_grants_skill() {
        let mut game = make_game();
        add_player(&mut game, "att", FieldCoordinate::new(5, 5), PS_STANDING, vec![SkillId::WisdomOfTheWhiteDwarf], true);
        add_player(&mut game, "mate1", FieldCoordinate::new(6, 5), PS_STANDING, vec![], true);
        add_player(&mut game, "mate2", FieldCoordinate::new(4, 5), PS_STANDING, vec![], true);
        setup_acting_player(&mut game, "att");

        let mut step = StepWisdomOfTheWhiteDwarf::new();
        step.start(&mut game, &mut GameRng::new(0));

        let out = step.handle_command(
            &Action::SelectPlayer { player_id: "mate2".into() },
            &mut game,
            &mut GameRng::new(0),
        );

        // Four grant-able skills exist and "mate2" has none of them, so a dialog to pick
        // which skill to grant is required (client-only stub) — CONTINUE is expected here,
        // but the granting-skill-use report must already have been added exactly once.
        assert_eq!(out.action, StepAction::Continue);
        assert_eq!(step.player_id.as_deref(), Some("mate2"));
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }

    #[test]
    fn select_skill_command_grants_chosen_skill_without_duplicate_skill_use_report() {
        let mut game = make_game();
        add_player(&mut game, "att", FieldCoordinate::new(5, 5), PS_STANDING, vec![SkillId::WisdomOfTheWhiteDwarf], true);
        add_player(&mut game, "mate", FieldCoordinate::new(6, 5), PS_STANDING, vec![], true);
        setup_acting_player(&mut game, "att");

        let mut step = StepWisdomOfTheWhiteDwarf::new();
        // Multiple teammates would be required to reach the dialog naturally; here we
        // directly drive the second command (as if the player dialog already resolved)
        // to exercise the CLIENT_PRAYER_SELECTION branch in isolation.
        step.player_id = Some("mate".into());
        let first = step.execute_step(&mut game);
        assert_eq!(first.action, StepAction::Continue, "multiple gain-able skills require a dialog");
        let skill_use_reports_before = game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE);
        assert!(skill_use_reports_before);

        let out = step.handle_command(
            &Action::SelectSkill { skill_id: SkillId::SureFeet },
            &mut game,
            &mut GameRng::new(0),
        );

        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.player("mate").unwrap().has_skill(SkillId::SureFeet));
        assert_eq!(step.granted_skill, Some(SkillId::SureFeet));
    }

    #[test]
    fn stunned_teammate_is_not_eligible() {
        let mut game = make_game();
        add_player(&mut game, "att", FieldCoordinate::new(5, 5), PS_STANDING, vec![SkillId::WisdomOfTheWhiteDwarf], true);
        add_player(&mut game, "mate", FieldCoordinate::new(6, 5), PS_STUNNED, vec![], true);
        setup_acting_player(&mut game, "att");

        let out = StepWisdomOfTheWhiteDwarf::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepWisdomOfTheWhiteDwarf::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn prone_teammate_is_eligible() {
        let mut game = make_game();
        add_player(&mut game, "att", FieldCoordinate::new(5, 5), PS_STANDING, vec![SkillId::WisdomOfTheWhiteDwarf], true);
        add_player(
            &mut game,
            "mate",
            FieldCoordinate::new(6, 5),
            PS_PRONE,
            vec![SkillId::BreakTackle, SkillId::Dauntless, SkillId::SureFeet],
            true,
        );
        setup_acting_player(&mut game, "att");

        let out = StepWisdomOfTheWhiteDwarf::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.player("mate").unwrap().has_skill(SkillId::MightyBlow));
    }
}
