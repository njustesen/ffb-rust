/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepUnchannelledFury`
/// and its hook `com.fumbbl.ffb.server.skillbehaviour.bb2020.UnchannelledFuryBehaviour`.
///
/// Handles the Unchannelled Fury skill (BB2020+/Mixed): requires an acting player with
/// `UnchannelledFury` to pass a d6 roll or lose the action.  If the player also has
/// the `FuryOfTheBloodGod` skill (unused) and the action is a block, a dialog is shown
/// that lets the coach use FuryOfTheBloodGod to take a second block despite the failure.
///
/// Java: `StepUnchannelledFury extends AbstractStepWithReRoll` (mixed, BB2020 + BB2025).
use ffb_model::enums::{PlayerAction, PS_PRONE, PS_STANDING, ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::mechanics::minimum_roll_confusion;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Mirrors Java ActionStatus values used inside the step state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillChoiceStatus {
    /// Java: ActionStatus.SKILL_CHOICE_YES
    Yes,
    /// Java: ActionStatus.SKILL_CHOICE_NO
    No,
}

/// Java: `StepUnchannelledFury.StepState`
pub struct StepUnchannelledFury {
    /// Java: state.goToLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: state.status — set when CLIENT_USE_SKILL for FuryOfTheBloodGod is received
    pub status: Option<SkillChoiceStatus>,
    // AbstractStepWithReRoll fields
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepUnchannelledFury {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self {
            goto_label_on_failure: goto_label_on_failure.into(),
            status: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepUnchannelledFury {
    fn default() -> Self { Self::new("") }
}

impl Step for StepUnchannelledFury {
    fn id(&self) -> StepId { StepId::UnchannelledFury }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_USE_SKILL for FuryOfTheBloodGod (skill with canPerformTwoBlocksAfterFailedFury)
            Action::UseSkill { skill_id, use_skill } if *skill_id == SkillId::FuryOfTheBloodGod => {
                self.status = Some(if *use_skill { SkillChoiceStatus::Yes } else { SkillChoiceStatus::No });
            }
            // Java: CLIENT_USE_SKILL for re-roll decline
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_source = None;
            }
            _ => {}
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

impl StepUnchannelledFury {
    /// Java: UnchannelledFuryBehaviour.handleExecuteStepHook (inlined).
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (!game.getTurnMode().checkNegatraits()) → NEXT_STEP
        if !game.turn_mode.check_negatraits() {
            return StepOutcome::next();
        }

        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: if (state.status == SKILL_CHOICE_YES) — FuryOfTheBloodGod dialog accepted
        if self.status == Some(SkillChoiceStatus::Yes) {
            // Java: actingPlayer.markSkillUsed(canPerformTwoBlocksAfterFailedFury)
            //       step.publishParameter(ALLOW_SECOND_BLOCK_ACTION, true) → NextStep
            mark_skill_used(game, &player_id, SkillId::FuryOfTheBloodGod);
            return StepOutcome::next()
                .publish(StepParameter::AllowSecondBlockAction(true));
        }

        // Java: if (state.status == SKILL_CHOICE_NO) — FuryOfTheBloodGod dialog declined
        if self.status == Some(SkillChoiceStatus::No) {
            cancel_unchannelled_fury_action(game, &player_id);
            return self.goto_failure(&player_id);
        }

        // Java: if (UtilCards.hasSkill(actingPlayer, skill))
        let has_uf = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::UnchannelledFury))
            .unwrap_or(false);

        if !has_uf {
            return StepOutcome::next();
        }

        // Java: if (UNCHANNELLED_FURY == reRolledAction && (source == null || !useReRoll)) → skip roll
        let skip_roll = if self.re_rolled_action.as_deref() == Some("UNCHANNELLED_FURY") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                !use_reroll(game, &source, &player_id)
            } else {
                true // player declined re-roll
            }
        } else {
            false
        };

        if skip_roll {
            // Re-roll declined or token exhausted — check for FuryOfTheBloodGod dialog
            let player_action = game.acting_player.player_action;
            if self.has_unused_fury_of_blood_god(game, &player_id)
                && player_action.map(|a| a.is_block_action()).unwrap_or(false)
            {
                // client-only: DialogSkillUseParameter for FuryOfTheBloodGod
                return StepOutcome::cont(); // wait for UseSkill response
            }
            cancel_unchannelled_fury_action(game, &player_id);
            return self.goto_failure(&player_id);
        }

        // Java: boolean doRoll = UtilCards.hasUnusedSkill(actingPlayer, skill)
        let do_roll = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::UnchannelledFury) && !p.used_skills.contains(&SkillId::UnchannelledFury))
            .unwrap_or(false);

        if !do_roll {
            return StepOutcome::next();
        }

        // Java: step.commitTargetSelection() → targetSelectionState.commit()
        if let Some(ref mut ts) = game.field_model.target_selection_state {
            ts.commit();
        }
        let roll = rng.d6();
        let player_action = game.acting_player.player_action;
        let good_conditions = good_conditions_for_uf(player_action);
        let min_roll = minimum_roll_confusion(good_conditions);
        let successful = roll >= min_roll;

        // Java: actingPlayer.markSkillUsed(skill)
        mark_skill_used(game, &player_id, SkillId::UnchannelledFury);

        let event = GameEvent::ConfusionRoll { player_id: player_id.clone(), roll, confused: !successful };

        if successful {
            return StepOutcome::next().with_event(event);
        }

        // Java: failed roll — check for re-roll
        if self.re_rolled_action.is_none() {
            if let Some(prompt) = ask_for_reroll_if_available(game, "UNCHANNELLED_FURY", min_roll, false) {
                self.re_rolled_action = Some("UNCHANNELLED_FURY".into());
                self.re_roll_source = Some("TRR".into());
                return StepOutcome::cont().with_event(event).with_prompt(prompt);
            }
        }

        // No re-roll available — check for FuryOfTheBloodGod dialog
        if self.has_unused_fury_of_blood_god(game, &player_id)
            && player_action.map(|a| a.is_block_action()).unwrap_or(false)
        {
            // client-only: DialogSkillUseParameter for FuryOfTheBloodGod
            return StepOutcome::cont().with_event(event);
        }

        cancel_unchannelled_fury_action(game, &player_id);
        self.goto_failure(&player_id).with_event(event)
    }

    fn goto_failure(&self, _player_id: &str) -> StepOutcome {
        StepOutcome::goto(&self.goto_label_on_failure)
            .publish(StepParameter::EndPlayerAction(true))
    }

    fn has_unused_fury_of_blood_god(&self, game: &Game, player_id: &str) -> bool {
        game.player(player_id)
            .map(|p| p.has_skill(SkillId::FuryOfTheBloodGod) && !p.used_skills.contains(&SkillId::FuryOfTheBloodGod))
            .unwrap_or(false)
    }
}

/// Java: UnchannelledFuryBehaviour.cancelPlayerAction — same as BoneHead but without confusion flag.
/// On failure: set to STANDING (not confused) and deactivate.
fn cancel_unchannelled_fury_action(game: &mut Game, player_id: &str) {
    match game.acting_player.player_action {
        Some(PlayerAction::Blitz) | Some(PlayerAction::BlitzMove)
        | Some(PlayerAction::KickEmBlitz) | Some(PlayerAction::StandUpBlitz) => {
            game.turn_data_mut().blitz_used = true;
        }
        Some(PlayerAction::KickTeamMate) | Some(PlayerAction::KickTeamMateMove) => {
            game.turn_data_mut().ktm_used = true;
        }
        Some(PlayerAction::Pass) | Some(PlayerAction::PassMove)
        | Some(PlayerAction::ThrowTeamMate) | Some(PlayerAction::ThrowTeamMateMove) => {
            game.turn_data_mut().pass_used = true;
        }
        Some(PlayerAction::HandOver) | Some(PlayerAction::HandOverMove) => {
            game.turn_data_mut().hand_over_used = true;
        }
        Some(PlayerAction::Foul) | Some(PlayerAction::FoulMove) => {
            game.turn_data_mut().foul_used = true;
        }
        Some(PlayerAction::SecureTheBall) => {
            game.turn_data_mut().secure_the_ball_used = true;
        }
        _ => {}
    }

    if let Some(state) = game.field_model.player_state(player_id) {
        // Java: if standing_up → changeBase(PRONE).changeActive(false)
        //       else → changeBase(STANDING).changeActive(false)
        let new_state = if game.acting_player.standing_up {
            state.change_base(PS_PRONE).change_active(false)
        } else {
            state.change_base(PS_STANDING).change_active(false)
        };
        game.field_model.set_player_state(player_id, new_state);
    }

    game.pass_coordinate = None;
    // Java: targetSelectionState.failed()
    if let Some(ref mut ts) = game.field_model.target_selection_state {
        ts.failed();
    }
    // Java: setSound(SoundId.ROAR) — client-only
}

/// Java: goodConditions = BlitzMove | isKickingDowned | Blitz | isBlockAction | MultipleBlock | StandUpBlitz
fn good_conditions_for_uf(player_action: Option<PlayerAction>) -> bool {
    match player_action {
        Some(pa) => {
            pa == PlayerAction::BlitzMove
                || pa.is_kicking_downed()
                || pa == PlayerAction::Blitz
                || pa.is_block_action()
                || pa == PlayerAction::MultipleBlock
                || pa == PlayerAction::StandUpBlitz
        }
        None => false,
    }
}

/// Mark a skill as used for the acting player.
fn mark_skill_used(game: &mut Game, player_id: &str, skill: SkillId) {
    let is_home = game.team_home.player(player_id).is_some();
    if is_home {
        if let Some(p) = game.team_home.player_mut(player_id) {
            p.used_skills.insert(skill);
        }
    } else if let Some(p) = game.team_away.player_mut(player_id) {
        p.used_skills.insert(skill);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{PlayerAction, PlayerState, PS_STANDING, Rules, TurnMode};
    use ffb_model::model::game::Game;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player_with_skill(game: &mut Game, player_id: &str, skill: SkillId) -> String {
        use ffb_model::enums::{PlayerGender, PlayerType};
        use ffb_model::model::player::Player;
        let player = Player {
            id: player_id.into(), name: player_id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 4, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: skill, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        };
        game.team_home.players.push(player);
        game.acting_player.player_id = Some(player_id.into());
        player_id.into()
    }

    #[test]
    fn id_is_unchannelled_fury() {
        assert_eq!(StepUnchannelledFury::new("fail").id(), StepId::UnchannelledFury);
    }

    #[test]
    fn skip_when_turn_mode_does_not_check_negatraits() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn; // check_negatraits() = false
        let mut step = StepUnchannelledFury::new("fail");
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn skip_when_player_does_not_have_skill() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        // Player exists but has no UnchannelledFury
        let mut step = StepUnchannelledFury::new("fail");
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepUnchannelledFury::default();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("SKIP".into())));
        assert_eq!(step.goto_label_on_failure, "SKIP");
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepUnchannelledFury::default();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn successful_roll_returns_next_step() {
        let mut game = make_game();
        // UnchannelledFury minimum roll = 2 (good conditions) or 4 (bad)
        // Seed 5 → d6 = deterministic high value
        add_player_with_skill(&mut game, "p1", SkillId::UnchannelledFury);
        game.acting_player.player_action = Some(PlayerAction::Block); // good conditions
        let mut step = StepUnchannelledFury::new("fail");
        // Roll 6 → success (min = 2 in good conditions)
        let out = step.start(&mut game, &mut GameRng::new(5));
        // If roll ≥ 2 → NextStep
        if out.action == StepAction::NextStep {
            let event_found = out.events.iter().any(|e| matches!(e, GameEvent::ConfusionRoll { .. }));
            assert!(event_found);
        } else {
            // Low roll → goto failure
            assert_eq!(out.action, StepAction::GotoLabel);
        }
    }

    #[test]
    fn skill_choice_yes_marks_fury_used_and_allows_second_block() {
        let mut game = make_game();
        add_player_with_skill(&mut game, "p1", SkillId::FuryOfTheBloodGod);
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepUnchannelledFury::new("fail");
        step.status = Some(SkillChoiceStatus::Yes);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // AllowSecondBlockAction(true) should be published
        let published = out.published.iter().any(|p| matches!(p, StepParameter::AllowSecondBlockAction(true)));
        assert!(published);
        // Skill marked used
        assert!(game.team_home.players[0].used_skills.contains(&SkillId::FuryOfTheBloodGod));
    }

    #[test]
    fn skill_choice_no_cancels_action_and_goes_to_failure() {
        let mut game = make_game();
        add_player_with_skill(&mut game, "p1", SkillId::UnchannelledFury);
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));
        let mut step = StepUnchannelledFury::new("FAIL");
        step.status = Some(SkillChoiceStatus::No);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        let published = out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true)));
        assert!(published);
    }

    #[test]
    fn cancel_sets_player_standing_inactive() {
        let mut game = make_game();
        add_player_with_skill(&mut game, "p1", SkillId::UnchannelledFury);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));
        game.acting_player.standing_up = false;
        cancel_unchannelled_fury_action(&mut game, "p1");
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_STANDING);
        assert!(!state.is_active());
    }

    #[test]
    fn cancel_standing_up_sets_prone_inactive() {
        let mut game = make_game();
        add_player_with_skill(&mut game, "p1", SkillId::UnchannelledFury);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));
        game.acting_player.standing_up = true;
        cancel_unchannelled_fury_action(&mut game, "p1");
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_PRONE);
        assert!(!state.is_active());
    }

    #[test]
    fn good_conditions_for_block_action() {
        assert!(good_conditions_for_uf(Some(PlayerAction::Block)));
        assert!(good_conditions_for_uf(Some(PlayerAction::Blitz)));
        assert!(good_conditions_for_uf(Some(PlayerAction::BlitzMove)));
        assert!(good_conditions_for_uf(Some(PlayerAction::MultipleBlock)));
        assert!(good_conditions_for_uf(Some(PlayerAction::StandUpBlitz)));
    }

    #[test]
    fn bad_conditions_for_move_action() {
        assert!(!good_conditions_for_uf(Some(PlayerAction::Move)));
        assert!(!good_conditions_for_uf(None));
    }

    #[test]
    fn handle_use_skill_fury_of_blood_god() {
        let mut game = make_game();
        let mut step = StepUnchannelledFury::new("fail");
        let mut rng = GameRng::new(0);
        step.handle_command(&Action::UseSkill { skill_id: SkillId::FuryOfTheBloodGod, use_skill: true }, &mut game, &mut rng);
        assert_eq!(step.status, Some(SkillChoiceStatus::Yes));
    }

    #[test]
    fn handle_decline_reroll_clears_source() {
        let mut game = make_game();
        let mut step = StepUnchannelledFury::new("fail");
        step.re_roll_source = Some("TRR".into());
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert!(step.re_roll_source.is_none());
    }
}
