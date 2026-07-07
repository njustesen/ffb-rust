/// 1:1 translation of com.fumbbl.ffb.server.step.action.common.StepReallyStupid
/// and its BB2025 hook com.fumbbl.ffb.server.skillbehaviour.bb2025.ReallyStupidBehaviour.
///
/// Resolves the Really Stupid negatrait roll. Needs GOTO_LABEL_ON_FAILURE init parameter.
/// goodConditions = true if an adjacent non-ReallyStupid teammate is present (2+ threshold),
/// else goodConditions = false (4+ threshold).
/// On failure: publishes END_PLAYER_ACTION, cancels the current player action,
/// and jumps to goToLabelOnFailure.
use ffb_model::enums::{PlayerAction, ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::types::FieldCoordinate;
use ffb_model::report::report_confusion_roll::ReportConfusionRoll;
use ffb_mechanics::mechanics::minimum_roll_confusion;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use super::cancel_negatrait_player_action;

pub struct StepReallyStupid {
    /// Java: state.goToLabelOnFailure — GOTO_LABEL_ON_FAILURE init parameter.
    pub goto_label_on_failure: String,
    // AbstractStepWithReRoll stubs (TODO: translate full re-roll infrastructure)
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepReallyStupid {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepReallyStupid {
    fn default() -> Self { Self::new() }
}

impl Step for StepReallyStupid {
    fn id(&self) -> StepId { StepId::ReallyStupid }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepReallyStupid {
    /// Java: ReallyStupidBehaviour.handleExecuteStepHook
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (!game.getTurnMode().checkNegatraits()) → NEXT_STEP
        if !game.turn_mode.check_negatraits() {
            return StepOutcome::next();
        }

        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: if (UtilCards.hasSkill(actingPlayer, skill))
        let has_really_stupid = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::ReallyStupid))
            .unwrap_or(false);

        if !has_really_stupid {
            return StepOutcome::next();
        }

        // Java: if (REALLY_STUPID == reRolledAction) { if (source == null || !useReRoll) skip roll }
        let mut skip_roll = false;
        if self.re_rolled_action.as_deref() == Some("REALLY_STUPID") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &player_id) {
                    skip_roll = true;
                }
            } else {
                skip_roll = true; // player declined
            }
        }

        let good_conditions = matches!(game.acting_player.player_action,
            Some(PlayerAction::ThrowTeamMate) | Some(PlayerAction::ThrowTeamMateMove)
            | Some(PlayerAction::KickTeamMate) | Some(PlayerAction::KickTeamMateMove)
        ) || has_non_really_stupid_adjacent_teammate(game, &player_id);

        let min_roll = minimum_roll_confusion(good_conditions);

        if skip_roll {
            let confusion_event = GameEvent::ConfusionRoll { player_id: player_id.clone(), roll: 1, confused: true };
            // Java: addReport(new ReportConfusionRoll(...))
            game.report_list.add(ReportConfusionRoll::new(
                Some(player_id.clone()),
                false,
                1,
                min_roll,
                true,
                Some(SkillId::ReallyStupid.class_name().to_string()),
            ));
            cancel_negatrait_player_action(game, &player_id);
            return StepOutcome::goto(&self.goto_label_on_failure)
                .with_event(confusion_event)
                .publish(StepParameter::EndPlayerAction(true));
        }

        let roll = rng.d6();
        let successful = roll >= min_roll;

        // Java: actingPlayer.markSkillUsed(skill)
        let is_home = game.team_home.player(&player_id).is_some();
        if is_home {
            if let Some(p) = game.team_home.player_mut(&player_id) {
                p.used_skills.insert(SkillId::ReallyStupid);
            }
        } else if let Some(p) = game.team_away.player_mut(&player_id) {
            p.used_skills.insert(SkillId::ReallyStupid);
        }

        // Java: addReport(new ReportConfusionRoll(...))
        let re_rolled_flag = self.re_rolled_action.as_deref() == Some("REALLY_STUPID")
            && self.re_roll_source.is_some();
        game.report_list.add(ReportConfusionRoll::new(
            Some(player_id.clone()),
            successful,
            roll,
            min_roll,
            re_rolled_flag,
            Some(SkillId::ReallyStupid.class_name().to_string()),
        ));

        let confusion_event = GameEvent::ConfusionRoll {
            player_id: player_id.clone(),
            roll,
            confused: !successful,
        };

        if successful {
            StepOutcome::next().with_event(confusion_event)
        } else {
            // Java: if (reRolledAction != REALLY_STUPID && askForReRollIfAvailable(...)) → CONTINUE
            if self.re_rolled_action.is_none() {
                if let Some(prompt) = ask_for_reroll_if_available(game, "REALLY_STUPID", min_roll, false) {
                    self.re_rolled_action = Some("REALLY_STUPID".into());
                    self.re_roll_source = Some("TRR".into());
                    return StepOutcome::cont().with_event(confusion_event).with_prompt(prompt);
                }
            }
            cancel_negatrait_player_action(game, &player_id);
            StepOutcome::goto(&self.goto_label_on_failure)
                .with_event(confusion_event)
                .publish(StepParameter::EndPlayerAction(true))
        }
    }
}

/// Java: UtilPlayer.findAdjacentBlockablePlayers on the acting player's own team,
/// then check if any teammate does NOT have ReallyStupid.
/// Returns true if at least one such teammate is adjacent (goodConditions = true).
fn has_non_really_stupid_adjacent_teammate(game: &Game, player_id: &str) -> bool {
    let coord = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return false,
    };
    let is_home = game.team_home.player(player_id).is_some();
    let teammates = if is_home { &game.team_home.players } else { &game.team_away.players };
    teammates.iter()
        .filter(|tp| tp.id != player_id)
        .filter(|tp| {
            game.field_model.player_coordinate(&tp.id)
                .map(|c| c.is_adjacent(coord))
                .unwrap_or(false)
        })
        .filter(|tp| {
            game.field_model.player_state(&tp.id)
                .map(|s| s.can_be_blocked())
                .unwrap_or(false)
        })
        .any(|tp| !tp.has_skill(SkillId::ReallyStupid))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, TurnMode, PS_STANDING};
    use ffb_model::enums::PlayerState;
    use ffb_model::model::skill_def::SkillWithValue;
    use crate::action::Action;
    use crate::step::framework::{StepAction, StepParameter};
    use crate::step::framework::test_team;

    fn add_player(team: &mut ffb_model::model::team::Team, id: &str, nr: i32, skills: Vec<SkillId>) {
        team.players.push(ffb_model::model::player::Player {
            id: id.into(),
            name: id.into(),
            nr,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            ..Default::default()
        });
    }

    fn make_game_with_really_stupid() -> (Game, String) {
        let pid = "p1".to_string();
        let mut home = test_team("home", 0);
        add_player(&mut home, &pid, 1, vec![SkillId::ReallyStupid]);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(5, 7));
        (game, pid)
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target { return s; }
        }
        panic!("no seed for d6={}", target);
    }

    #[test]
    fn negatraits_disabled_skips_roll() {
        let (mut game, _) = make_game_with_really_stupid();
        game.turn_mode = TurnMode::KickoffReturn;
        let outcome = StepReallyStupid::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn player_without_skill_skips_roll() {
        let (mut game, _) = make_game_with_really_stupid();
        game.team_home.players[0].starting_skills.clear();
        let outcome = StepReallyStupid::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn good_conditions_success_on_2() {
        let seed = seed_for_d6(2);
        let (mut game, _) = make_game_with_really_stupid();
        // Adjacent non-ReallyStupid teammate → goodConditions = true
        add_player(&mut game.team_home, "p2", 2, vec![]);
        game.field_model.set_player_state("p2", PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate("p2", FieldCoordinate::new(6, 7));

        let mut step = StepReallyStupid::new();
        step.goto_label_on_failure = "FAIL".into();
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn bad_conditions_fail_on_3() {
        let seed = seed_for_d6(3);
        let (mut game, _) = make_game_with_really_stupid();
        // Adjacent teammate ALSO has ReallyStupid → no support → bad conditions
        add_player(&mut game.team_home, "p2", 2, vec![SkillId::ReallyStupid]);
        game.field_model.set_player_state("p2", PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate("p2", FieldCoordinate::new(6, 7));

        let mut step = StepReallyStupid::new();
        step.goto_label_on_failure = "FAIL".into();
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(outcome.action, StepAction::GotoLabel, "3 < 4 should fail without support");
    }

    #[test]
    fn throw_team_mate_always_good_conditions() {
        let seed = seed_for_d6(2);
        let (mut game, _) = make_game_with_really_stupid();
        game.acting_player.player_action = Some(PlayerAction::ThrowTeamMate);
        // No teammates adjacent — would be bad conditions, but TTM overrides

        let mut step = StepReallyStupid::new();
        step.goto_label_on_failure = "FAIL".into();
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn failed_roll_goto_label_and_confused() {
        let seed = seed_for_d6(1);
        let (mut game, pid) = make_game_with_really_stupid();
        let mut step = StepReallyStupid::new();
        step.goto_label_on_failure = "RS_FAIL".into();
        let outcome = step.start(&mut game, &mut GameRng::new(seed));

        assert_eq!(outcome.action, StepAction::GotoLabel);
        assert_eq!(outcome.goto_label.as_deref(), Some("RS_FAIL"));
        assert!(matches!(outcome.published.first(), Some(StepParameter::EndPlayerAction(true))));

        let state = game.field_model.player_state(&pid).unwrap();
        assert!(state.is_confused());
        assert!(!state.is_active());
    }

    #[test]
    fn set_parameter_stores_goto_label() {
        let mut step = StepReallyStupid::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("Z".into())));
        assert_eq!(step.goto_label_on_failure, "Z");
    }

    #[test]
    fn really_stupid_marked_used_after_roll() {
        let seed = seed_for_d6(3); // might fail or succeed depending on conditions
        let (mut game, pid) = make_game_with_really_stupid();
        // Add adjacent non-RS teammate for good conditions (seed 3 → success)
        add_player(&mut game.team_home, "p2", 2, vec![]);
        game.field_model.set_player_state("p2", PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate("p2", FieldCoordinate::new(6, 7));

        StepReallyStupid::new().start(&mut game, &mut GameRng::new(seed));
        assert!(game.team_home.player(&pid).unwrap().used_skills.contains(&SkillId::ReallyStupid));
    }

    #[test]
    fn failed_roll_with_trr_offers_reroll_prompt() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_game_with_really_stupid();
        game.turn_data_home.rerolls = 1;
        let mut step = StepReallyStupid::new();
        step.goto_label_on_failure = "FAIL".into();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue, "TRR available → should offer re-roll");
        assert!(out.prompt.is_some());
        assert_eq!(step.re_rolled_action.as_deref(), Some("REALLY_STUPID"));
    }

    #[test]
    fn successful_roll_adds_confusion_roll_report() {
        let seed = seed_for_d6(2);
        let (mut game, _) = make_game_with_really_stupid();
        // Adjacent non-RS teammate → good conditions (min_roll=2)
        add_player(&mut game.team_home, "p2", 2, vec![]);
        game.field_model.set_player_state("p2", PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate("p2", FieldCoordinate::new(6, 7));
        let mut step = StepReallyStupid::new();
        step.goto_label_on_failure = "FAIL".into();
        step.start(&mut game, &mut GameRng::new(seed));
        assert!(
            game.report_list.has_report(ffb_model::report::report_id::ReportId::CONFUSION_ROLL),
            "successful roll should add ReportConfusionRoll"
        );
    }

    #[test]
    fn failed_roll_adds_confusion_roll_report() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_game_with_really_stupid();
        let mut step = StepReallyStupid::new();
        step.goto_label_on_failure = "FAIL".into();
        step.start(&mut game, &mut GameRng::new(seed));
        assert!(
            game.report_list.has_report(ffb_model::report::report_id::ReportId::CONFUSION_ROLL),
            "failed roll should add ReportConfusionRoll"
        );
    }

    #[test]
    fn decline_reroll_clears_source_and_fails() {
        let (mut game, _) = make_game_with_really_stupid();
        let mut step = StepReallyStupid::new();
        step.goto_label_on_failure = "FAIL".into();
        step.re_rolled_action = Some("REALLY_STUPID".into());
        step.re_roll_source = Some("TRR".into());
        let out = step.handle_command(
            &Action::UseReRoll { use_reroll: false },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("FAIL"));
    }
}
