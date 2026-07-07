/// 1:1 translation of com.fumbbl.ffb.server.step.action.common.StepBoneHead
/// and its BB2025 hook com.fumbbl.ffb.server.skillbehaviour.bb2025.BoneHeadBehaviour.
///
/// Resolves the Bone Head negatrait roll. Needs GOTO_LABEL_ON_FAILURE init parameter.
/// On failure: publishes END_PLAYER_ACTION, cancels the current player action,
/// and jumps to goToLabelOnFailure.
use ffb_model::enums::{ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::mechanics::minimum_roll_confusion;
use ffb_model::report::report_confusion_roll::ReportConfusionRoll;
use ffb_model::report::report_id::ReportId;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use super::cancel_negatrait_player_action;

pub struct StepBoneHead {
    /// Java: state.goToLabelOnFailure — GOTO_LABEL_ON_FAILURE init parameter.
    pub goto_label_on_failure: String,
    // AbstractStepWithReRoll stubs (TODO: translate full re-roll infrastructure)
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepBoneHead {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepBoneHead {
    fn default() -> Self { Self::new() }
}

impl Step for StepBoneHead {
    fn id(&self) -> StepId { StepId::BoneHead }

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

impl StepBoneHead {
    /// Java: BoneHeadBehaviour.handleExecuteStepHook
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
        let has_bone_head = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::BoneHead))
            .unwrap_or(false);

        if !has_bone_head {
            return StepOutcome::next();
        }

        // Java: if (BONE_HEAD == reRolledAction) { if (source == null || !useReRoll) doRoll = false }
        let mut skip_roll = false;
        if self.re_rolled_action.as_deref() == Some("BONE_HEAD") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &player_id) {
                    skip_roll = true; // token exhausted
                }
                // else: re-roll consumed, proceed to re-roll
            } else {
                skip_roll = true; // player declined (handle_command cleared source)
            }
        }

        if skip_roll {
            let confusion_event = GameEvent::ConfusionRoll { player_id: player_id.clone(), roll: 1, confused: true };
            cancel_negatrait_player_action(game, &player_id);
            return StepOutcome::goto(&self.goto_label_on_failure)
                .with_event(confusion_event)
                .publish(StepParameter::EndPlayerAction(true));
        }

        // Java: step.commitTargetSelection() → targetSelectionState.commit()
        if let Some(ref mut ts) = game.field_model.target_selection_state {
            ts.commit();
        }
        let roll = rng.d6();
        // Java: minimumRollConfusion(true) — BoneHead always uses good-conditions threshold (2+)
        let min_roll = minimum_roll_confusion(true);
        let successful = roll >= min_roll;

        // Java: actingPlayer.markSkillUsed(skill)
        let is_home = game.team_home.player(&player_id).is_some();
        if is_home {
            if let Some(p) = game.team_home.player_mut(&player_id) {
                p.used_skills.insert(SkillId::BoneHead);
            }
        } else if let Some(p) = game.team_away.player_mut(&player_id) {
            p.used_skills.insert(SkillId::BoneHead);
        }

        // Java: addReport(new ReportConfusionRoll(...))
        let re_rolled = self.re_rolled_action.as_deref() == Some("BONE_HEAD")
            && self.re_roll_source.is_some();
        game.report_list.add(ReportConfusionRoll::new(
            Some(player_id.clone()),
            successful,
            roll,
            min_roll,
            re_rolled,
            Some(SkillId::BoneHead.class_name().to_string()),
        ));
        let confusion_event = GameEvent::ConfusionRoll {
            player_id: player_id.clone(),
            roll,
            confused: !successful,
        };

        if successful {
            StepOutcome::next().with_event(confusion_event)
        } else {
            // Java: if (reRolledAction != BONE_HEAD && askForReRollIfAvailable(...)) → CONTINUE
            if self.re_rolled_action.is_none() {
                if let Some(prompt) = ask_for_reroll_if_available(game, "BONE_HEAD", min_roll, false) {
                    self.re_rolled_action = Some("BONE_HEAD".into());
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

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, TurnMode, PlayerAction, PS_STANDING, PS_PRONE};
    use ffb_model::enums::PlayerState;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use crate::action::Action;
    use crate::step::framework::{StepAction, StepParameter};
    use crate::step::framework::test_team;

    fn make_game_with_bone_head_player() -> (Game, String) {
        let player_id = "p1".to_string();
        let mut home = test_team("home", 0);
        home.players.push(ffb_model::model::player::Player {
            id: player_id.clone(),
            name: "BoneHead".into(),
            nr: 1,
            position_id: "pos1".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::BoneHead, value: None }],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
        });
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.acting_player.player_id = Some(player_id.clone());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.field_model.set_player_state(&player_id, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&player_id, FieldCoordinate::new(5, 7));
        (game, player_id)
    }

    /// Find first seed that gives a specific d6 value.
    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target { return s; }
        }
        panic!("no seed for d6={}", target);
    }

    #[test]
    fn negatraits_disabled_skips_roll() {
        let (mut game, _) = make_game_with_bone_head_player();
        game.turn_mode = TurnMode::KickoffReturn; // check_negatraits() = false
        let mut step = StepBoneHead::new();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty(), "no roll should occur");
    }

    #[test]
    fn player_without_bone_head_skips_roll() {
        let (mut game, _) = make_game_with_bone_head_player();
        game.team_home.players[0].starting_skills.clear();
        let mut step = StepBoneHead::new();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn successful_roll_returns_next_step_with_event() {
        let seed = seed_for_d6(4);
        let (mut game, _) = make_game_with_bone_head_player();
        let mut step = StepBoneHead::new();
        step.goto_label_on_failure = "FAIL".into();
        let mut rng = GameRng::new(seed);
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(outcome.events.len(), 1);
        match &outcome.events[0] {
            GameEvent::ConfusionRoll { confused, roll, .. } => {
                assert!(!confused, "roll >= 2 should not confuse");
                assert_eq!(*roll, 4);
            }
            _ => panic!("expected ConfusionRoll"),
        }
        assert!(outcome.published.is_empty());
    }

    #[test]
    fn failed_roll_goes_to_label_and_confuses_player() {
        let seed = seed_for_d6(1);
        let (mut game, player_id) = make_game_with_bone_head_player();
        let mut step = StepBoneHead::new();
        step.goto_label_on_failure = "FAIL_LABEL".into();
        let mut rng = GameRng::new(seed);
        let outcome = step.start(&mut game, &mut rng);

        assert_eq!(outcome.action, StepAction::GotoLabel);
        assert_eq!(outcome.goto_label.as_deref(), Some("FAIL_LABEL"));
        assert_eq!(outcome.events.len(), 1);
        match &outcome.events[0] {
            GameEvent::ConfusionRoll { confused, .. } => assert!(confused),
            _ => panic!("expected ConfusionRoll"),
        }
        assert!(matches!(outcome.published.first(), Some(StepParameter::EndPlayerAction(true))));

        let state = game.field_model.player_state(&player_id).unwrap();
        assert!(state.is_confused(), "standing player should be confused");
        assert!(!state.is_active());
    }

    #[test]
    fn failed_roll_standing_up_makes_player_prone() {
        let seed = seed_for_d6(1);
        let (mut game, player_id) = make_game_with_bone_head_player();
        game.acting_player.standing_up = true;
        let mut step = StepBoneHead::new();
        step.goto_label_on_failure = "FAIL".into();
        let mut rng = GameRng::new(seed);
        step.start(&mut game, &mut rng);

        let state = game.field_model.player_state(&player_id).unwrap();
        assert_eq!(state.base(), PS_PRONE);
        assert!(!state.is_active());
    }

    #[test]
    fn failed_roll_blitz_marks_blitz_used() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_game_with_bone_head_player();
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        let mut step = StepBoneHead::new();
        step.goto_label_on_failure = "FAIL".into();
        let mut rng = GameRng::new(seed);
        step.start(&mut game, &mut rng);
        assert!(game.turn_data_home.blitz_used);
    }

    #[test]
    fn failed_roll_move_does_not_mark_blitz_used() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_game_with_bone_head_player();
        game.acting_player.player_action = Some(PlayerAction::Move);
        let mut step = StepBoneHead::new();
        step.goto_label_on_failure = "FAIL".into();
        let mut rng = GameRng::new(seed);
        step.start(&mut game, &mut rng);
        assert!(!game.turn_data_home.blitz_used);
    }

    #[test]
    fn failed_roll_clears_pass_coordinate() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_game_with_bone_head_player();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 5));
        let mut step = StepBoneHead::new();
        step.goto_label_on_failure = "FAIL".into();
        let mut rng = GameRng::new(seed);
        step.start(&mut game, &mut rng);
        assert!(game.pass_coordinate.is_none());
    }

    #[test]
    fn bone_head_marked_used_after_roll() {
        let seed = seed_for_d6(4);
        let (mut game, player_id) = make_game_with_bone_head_player();
        let mut step = StepBoneHead::new();
        let mut rng = GameRng::new(seed);
        step.start(&mut game, &mut rng);
        assert!(game.team_home.player(&player_id).unwrap().used_skills.contains(&SkillId::BoneHead));
    }

    #[test]
    fn set_parameter_stores_goto_label() {
        let mut step = StepBoneHead::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("X".into())));
        assert_eq!(step.goto_label_on_failure, "X");
    }

    #[test]
    fn set_parameter_ignores_other_keys() {
        let mut step = StepBoneHead::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn failed_roll_with_trr_offers_reroll_prompt() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_game_with_bone_head_player();
        game.turn_data_home.rerolls = 1; // TRR available
        let mut step = StepBoneHead::new();
        step.goto_label_on_failure = "FAIL".into();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue, "should offer re-roll and wait");
        assert!(out.prompt.is_some(), "should include re-roll prompt");
        assert_eq!(step.re_rolled_action.as_deref(), Some("BONE_HEAD"));
    }

    #[test]
    fn decline_reroll_clears_source_and_fails() {
        let (mut game, _) = make_game_with_bone_head_player();
        let mut step = StepBoneHead::new();
        step.goto_label_on_failure = "FAIL".into();
        step.re_rolled_action = Some("BONE_HEAD".into());
        step.re_roll_source = Some("TRR".into());
        // Simulate decline
        let out = step.handle_command(
            &Action::UseReRoll { use_reroll: false },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("FAIL"));
    }

    #[test]
    fn successful_roll_adds_confusion_roll_report() {
        let seed = seed_for_d6(4);
        let (mut game, _) = make_game_with_bone_head_player();
        let mut step = StepBoneHead::new();
        step.goto_label_on_failure = "FAIL".into();
        let mut rng = GameRng::new(seed);
        step.start(&mut game, &mut rng);
        assert!(
            game.report_list.has_report(ReportId::CONFUSION_ROLL),
            "CONFUSION_ROLL report must be added after a successful roll"
        );
    }

    #[test]
    fn failed_roll_adds_confusion_roll_report() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_game_with_bone_head_player();
        let mut step = StepBoneHead::new();
        step.goto_label_on_failure = "FAIL".into();
        let mut rng = GameRng::new(seed);
        step.start(&mut game, &mut rng);
        assert!(
            game.report_list.has_report(ReportId::CONFUSION_ROLL),
            "CONFUSION_ROLL report must be added after a failed roll"
        );
    }
}
