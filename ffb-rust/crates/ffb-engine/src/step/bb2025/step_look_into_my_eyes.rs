/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepLookIntoMyEyes (BB2025).
///
/// Resolves the Look Into My Eyes skill: roll to steal the ball from the defender (2+ success).
///
/// Init params: PUSH_SELECT, GOTO_LABEL_ON_END.
/// Runtime params: END_TURN, END_PLAYER_ACTION.
///
/// Java's `leave()` either (a) cancels the player's action and pushes the EndPlayerAction
/// sequence, or (b) shows an "action failed" info dialog and optionally pushes the Select
/// sequence (PUSH_SELECT). Both are wired here using the same conventions as the sibling
/// `StepHypnoticGaze` (re-roll offer via `AgentPrompt::ReRollOffer`) and
/// `StepWeatherMage` (informational dialog via `AgentPrompt::InformationOkay` +
/// `Action::Acknowledge`).
use ffb_model::enums::{PlayerAction, ReRollSource, SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::prompts::AgentPrompt;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::report::mixed::report_look_into_my_eyes_roll::ReportLookIntoMyEyesRoll;
use ffb_model::report::mixed::report_skill_wasted::ReportSkillWasted;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2025::EndPlayerAction;
use crate::step::generator::bb2025::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2025::Select;
use crate::step::generator::bb2025::select::SelectParams;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java: `ReRolledActions.LOOK_INTO_MY_EYES`.
const RE_ROLLED_ACTION: &str = "LOOK_INTO_MY_EYES";

pub struct StepLookIntoMyEyes {
    /// Java: endPlayerAction — set by END_PLAYER_ACTION parameter.
    pub end_player_action: bool,
    /// Java: endTurn — set by END_TURN parameter.
    pub end_turn: bool,
    /// Java: pushSelect — PUSH_SELECT init parameter (push a Select sequence on failure).
    pub push_select: bool,
    /// Java: gotoOnEnd — GOTO_LABEL_ON_END init parameter (unused in executeStep, mirrors
    /// Java where the field is only read/written via toJsonValue/initFrom).
    pub goto_on_end: String,
    /// AbstractStepWithReRoll stubs — mirrors `StepHypnoticGaze`.
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
    /// True after we've shown the "action failed" info dialog and are waiting for
    /// `Action::Acknowledge` before (optionally) pushing the Select sequence.
    awaiting_ack: bool,
}

impl StepLookIntoMyEyes {
    pub fn new() -> Self {
        Self {
            end_player_action: false,
            end_turn: false,
            push_select: false,
            goto_on_end: String::new(),
            re_rolled_action: None,
            re_roll_source: None,
            awaiting_ack: false,
        }
    }
}

impl Default for StepLookIntoMyEyes {
    fn default() -> Self { Self::new() }
}

impl Step for StepLookIntoMyEyes {
    fn id(&self) -> StepId { StepId::LookIntoMyEyes }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.awaiting_ack {
            self.awaiting_ack = false;
            return self.finish_after_ack();
        }

        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v)           => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)   => { self.end_player_action = *v; true }
            StepParameter::PushSelect(v)        => { self.push_select = *v; true }
            StepParameter::GotoLabelOnEnd(v)    => { self.goto_on_end = v.clone(); true }
            _ => false,
        }
    }
}

impl StepLookIntoMyEyes {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: skill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canStealBallFromOpponent)
        let has_skill = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::LookIntoMyEyes) && !p.used_skills.contains(&SkillId::LookIntoMyEyes))
            .unwrap_or(false);

        if !has_skill {
            // Java: leave(actingPlayer, skill=null, false, false) — markSkillUsed(null) is no-op
            return self.leave(game, &player_id, false, false, false);
        }

        // Java: if (endTurn || endPlayerAction) → ReportSkillWasted + leave(endPlayerAction, endTurn)
        if self.end_turn || self.end_player_action {
            game.report_list.add(ReportSkillWasted::new(
                Some(player_id.clone()),
                Some(SkillId::LookIntoMyEyes),
            ));
            return self.leave(game, &player_id, true, self.end_player_action, self.end_turn);
        }

        // Java: if (getReRolledAction() == RE_ROLLED_ACTION) {
        //         if (getReRollSource() == null || !useReRoll(...)) { leave(false, false); return; }
        //       }
        if self.re_rolled_action.as_deref() == Some(RE_ROLLED_ACTION) {
            let consumed = match self.re_roll_source.clone() {
                Some(source_name) => use_reroll(game, &ReRollSource::new(source_name.as_str()), &player_id),
                None => false,
            };
            if !consumed {
                return self.leave(game, &player_id, true, false, false);
            }
        }

        if game.defender_id.is_some() {
            let player_coord = game.field_model.player_coordinate(&player_id);
            // Java: getResult().addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skill, true, LOOK_INTO_MY_EYES))
            game.report_list.add(ReportSkillUse::new(
                Some(player_id.clone()),
                SkillId::LookIntoMyEyes,
                true,
                SkillUse::LOOK_INTO_MY_EYES,
            ));

            let roll = rng.d6();
            let successful = roll > 1;
            let re_rolled = self.re_rolled_action.is_some();

            // Java: addReport(new ReportLookIntoMyEyesRoll(playerId, successful, roll, reRolled))
            game.report_list.add(ReportLookIntoMyEyesRoll::new(
                Some(player_id.clone()),
                successful,
                roll,
                2,
                re_rolled,
            ));

            if successful {
                if let Some(coord) = player_coord {
                    game.field_model.ball_coordinate = Some(coord);
                }
                // Java: leave(actingPlayer, skill, true, false)
                return self.leave(game, &player_id, true, true, false);
            } else if self.re_rolled_action.is_none() {
                // Java: else if (reRolledAction != null || !askForReRollIfAvailable(...)) leave(false, false)
                if let Some(prompt) = ask_for_reroll_if_available(game, RE_ROLLED_ACTION, 2, false) {
                    let source_name = match &prompt {
                        AgentPrompt::ReRollOffer { source, .. } => source.name.clone(),
                        _ => String::new(),
                    };
                    self.re_rolled_action = Some(RE_ROLLED_ACTION.into());
                    self.re_roll_source = Some(source_name);
                    return StepOutcome::cont().with_prompt(prompt);
                }
                return self.leave(game, &player_id, true, false, false);
            } else {
                // Already re-rolled once and failed again — no further re-roll offered.
                return self.leave(game, &player_id, true, false, false);
            }
        } else {
            // Java: leave(actingPlayer, skill, true, false)
            self.leave(game, &player_id, true, true, false)
        }
    }

    /// Java: `leave(ActingPlayer actingPlayer, Skill skill, boolean endPlayerAction, boolean endTurn)`.
    fn leave(&mut self, game: &mut Game, player_id: &str, has_skill: bool, end_player_action: bool, end_turn: bool) -> StepOutcome {
        if has_skill {
            Self::mark_skill_used(game, player_id);
        }

        if end_player_action || end_turn {
            let clear_stack = Self::cancel_player_action(game);
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: false,
                end_player_action,
                end_turn,
                check_forgo: false,
            });
            let mut out = StepOutcome::next().push_seq(seq);
            if clear_stack {
                out = out.with_clear_stack();
            }
            out
        } else {
            // Java: UtilServerDialog.showDialog(gameState, new DialogInformationOkayParameter(
            //         "Look Into My Eyes", "Look Into My Eyes failed, you may continue your action", false), false)
            //       if (pushSelect) { ... pushSequence(new Select.SequenceParams(gameState, true)) }
            self.awaiting_ack = true;
            StepOutcome::cont().with_prompt(AgentPrompt::InformationOkay {
                message: "Look Into My Eyes failed, you may continue your action".into(),
            })
        }
    }

    /// Continuation of `leave()`'s else-branch after the agent acknowledges the info dialog.
    fn finish_after_ack(&self) -> StepOutcome {
        if self.push_select {
            let seq = Select::build_sequence(&SelectParams {
                update_persistence: true,
                ..Default::default()
            });
            StepOutcome::next().push_seq(seq)
        } else {
            StepOutcome::next()
        }
    }

    /// Java: `private void cancelPlayerAction()`.
    /// Returns true if the step stack must be cleared (BLITZ/GAZE/THROW_KEG cases).
    fn cancel_player_action(game: &mut Game) -> bool {
        match game.acting_player.player_action {
            Some(PlayerAction::Blitz) | Some(PlayerAction::BlitzMove) | Some(PlayerAction::KickEmBlitz) => {
                game.turn_data_mut().blitz_used = true;
                game.turn_mode = game.last_turn_mode.unwrap_or(game.turn_mode);
                true
            }
            Some(PlayerAction::Gaze) | Some(PlayerAction::GazeMove) => {
                game.turn_mode = game.last_turn_mode.unwrap_or(game.turn_mode);
                true
            }
            Some(PlayerAction::ThrowKeg) => {
                game.turn_mode = game.last_turn_mode.unwrap_or(game.turn_mode);
                true
            }
            Some(PlayerAction::KickTeamMate) | Some(PlayerAction::KickTeamMateMove) => {
                game.turn_data_mut().ktm_used = true;
                false
            }
            Some(PlayerAction::ThrowTeamMate) | Some(PlayerAction::ThrowTeamMateMove) => {
                game.turn_data_mut().ttm_used = true;
                false
            }
            Some(PlayerAction::Foul) | Some(PlayerAction::FoulMove) => {
                let allows_additional = game.acting_player.player_id.clone()
                    .and_then(|id| game.player(&id).map(|p| p.has_skill_property(NamedProperties::ALLOWS_ADDITIONAL_FOUL)))
                    .unwrap_or(false);
                if !allows_additional {
                    game.turn_data_mut().foul_used = true;
                }
                false
            }
            _ => false,
        }
    }

    fn mark_skill_used(game: &mut Game, player_id: &str) {
        let is_home = game.team_home.player(player_id).is_some();
        if is_home {
            if let Some(p) = game.team_home.player_mut(player_id) {
                p.used_skills.insert(SkillId::LookIntoMyEyes);
            }
        } else if let Some(p) = game.team_away.player_mut(player_id) {
            p.used_skills.insert(SkillId::LookIntoMyEyes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PlayerState, PS_STANDING, PlayerAction, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn make_player(id: &str, skill: Option<SkillId>) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skill.map(|s| vec![SkillWithValue { skill_id: s, value: None }])
                .unwrap_or_default(),
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_game_lime() -> (Game, String) {
        let pid = "actor".to_string();
        let mut home = test_team("home", 0);
        home.players.push(make_player(&pid, Some(SkillId::LookIntoMyEyes)));
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(10, 7));
        (game, pid)
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target { return s; }
        }
        panic!("no seed for d6={}", target);
    }

    #[test]
    fn no_skill_shows_failed_dialog_and_waits_for_ack() {
        let (mut game, _) = make_game_lime();
        game.team_home.players[0].starting_skills.clear();
        let mut step = StepLookIntoMyEyes::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Java: leave(null, false, false) → info dialog, not an immediate NEXT_STEP.
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::InformationOkay { .. })));
    }

    #[test]
    fn no_skill_ack_without_push_select_returns_next_step() {
        let (mut game, _) = make_game_lime();
        game.team_home.players[0].starting_skills.clear();
        let mut step = StepLookIntoMyEyes::new();
        step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.pushes.is_empty());
    }

    #[test]
    fn no_skill_ack_with_push_select_pushes_select_sequence() {
        let (mut game, _) = make_game_lime();
        game.team_home.players[0].starting_skills.clear();
        let mut step = StepLookIntoMyEyes::new();
        step.push_select = true;
        step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting);
    }

    #[test]
    fn end_turn_returns_next_step_and_marks_used() {
        let (mut game, actor_id) = make_game_lime();
        let mut step = StepLookIntoMyEyes::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1, "end_turn must push the EndPlayerAction sequence");
        assert!(game.team_home.player(&actor_id).unwrap().used_skills.contains(&SkillId::LookIntoMyEyes));
    }

    #[test]
    fn success_steals_ball() {
        let seed = seed_for_d6(4); // > 1
        let (mut game, actor_id) = make_game_lime();
        game.defender_id = Some("def".into());
        let ball_coord = FieldCoordinate::new(11, 7);
        game.field_model.ball_coordinate = Some(ball_coord);

        let mut step = StepLookIntoMyEyes::new();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(10, 7)));
        assert!(game.team_home.player(&actor_id).unwrap().used_skills.contains(&SkillId::LookIntoMyEyes));
    }

    #[test]
    fn failure_without_reroll_available_shows_failed_dialog() {
        let seed = seed_for_d6(1); // == 1, not > 1
        let (mut game, _) = make_game_lime();
        game.defender_id = Some("def".into());
        let ball_coord = FieldCoordinate::new(11, 7);
        game.field_model.ball_coordinate = Some(ball_coord);
        game.turn_data_home.rerolls = 0;

        let mut step = StepLookIntoMyEyes::new();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        // No TRR / skill re-roll available → leave(true, false, false) → info dialog, waits for ack.
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::InformationOkay { .. })));
        assert_eq!(game.field_model.ball_coordinate, Some(ball_coord));
    }

    #[test]
    fn failure_with_reroll_available_offers_reroll() {
        let seed = seed_for_d6(1); // == 1, not > 1
        let (mut game, _) = make_game_lime();
        game.defender_id = Some("def".into());
        game.turn_data_home.rerolls = 1;

        let mut step = StepLookIntoMyEyes::new();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::ReRollOffer { .. })));
    }

    #[test]
    fn declining_reroll_shows_failed_dialog() {
        let seed = seed_for_d6(1);
        let (mut game, _) = make_game_lime();
        game.defender_id = Some("def".into());
        game.turn_data_home.rerolls = 1;

        let mut step = StepLookIntoMyEyes::new();
        step.start(&mut game, &mut GameRng::new(seed));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::InformationOkay { .. })));
        assert_eq!(game.turn_data_home.rerolls, 1, "declined re-roll must not consume the TRR");
    }

    #[test]
    fn accepting_reroll_consumes_trr_and_rerolls() {
        let fail_seed = seed_for_d6(1);
        let (mut game, actor_id) = make_game_lime();
        game.defender_id = Some("def".into());
        game.turn_data_home.rerolls = 1;
        game.field_model.ball_coordinate = None;

        let mut step = StepLookIntoMyEyes::new();
        step.start(&mut game, &mut GameRng::new(fail_seed));
        assert_eq!(game.turn_data_home.rerolls, 1, "re-roll not yet consumed before acceptance");

        let success_seed = seed_for_d6(4);
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(success_seed));
        assert_eq!(game.turn_data_home.rerolls, 0, "accepted re-roll must consume the TRR");
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.field_model.ball_coordinate.is_some(), "re-rolled success should steal the ball");
        assert!(game.team_home.player(&actor_id).unwrap().used_skills.contains(&SkillId::LookIntoMyEyes));
    }

    #[test]
    fn set_parameter_wiring() {
        let mut step = StepLookIntoMyEyes::new();
        assert!(step.set_parameter(&StepParameter::PushSelect(true)));
        assert!(step.push_select);
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("END".into())));
        assert_eq!(step.goto_on_end, "END");
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }

    #[test]
    fn end_turn_adds_skill_wasted_report() {
        use ffb_model::report::report_id::ReportId;
        let (mut game, _) = make_game_lime();
        let mut step = StepLookIntoMyEyes::new();
        step.end_turn = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_WASTED),
            "end_turn should add ReportSkillWasted");
    }

    #[test]
    fn successful_roll_adds_look_into_my_eyes_roll_report_and_skill_use_report() {
        use ffb_model::report::report_id::ReportId;
        let seed = seed_for_d6(4); // > 1 → success
        let (mut game, _) = make_game_lime();
        game.defender_id = Some("def".into());
        let mut step = StepLookIntoMyEyes::new();
        step.start(&mut game, &mut GameRng::new(seed));
        assert!(game.report_list.has_report(ReportId::LOOK_INTO_MY_EYES_ROLL),
            "successful roll should add ReportLookIntoMyEyesRoll");
        assert!(game.report_list.has_report(ReportId::SKILL_USE),
            "successful roll should add ReportSkillUse(true, LOOK_INTO_MY_EYES)");
    }

    #[test]
    fn end_player_action_cancel_sets_blitz_used_and_clears_stack() {
        let (mut game, _) = make_game_lime();
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        game.turn_mode = ffb_model::enums::TurnMode::Blitz;
        game.last_turn_mode = Some(ffb_model::enums::TurnMode::Regular);
        let mut step = StepLookIntoMyEyes::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(game.turn_data_home.blitz_used, "cancelPlayerAction must set blitz_used for BLITZ");
        assert_eq!(game.turn_mode, ffb_model::enums::TurnMode::Regular);
        assert!(out.clear_stack, "BLITZ cancelPlayerAction must clear the step stack");
    }

    #[test]
    fn no_defender_leaves_with_end_player_action() {
        let (mut game, actor_id) = make_game_lime();
        game.defender_id = None;
        let mut step = StepLookIntoMyEyes::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Java: else branch → leave(actingPlayer, skill, true, false) → EndPlayerAction pushed.
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert!(game.team_home.player(&actor_id).unwrap().used_skills.contains(&SkillId::LookIntoMyEyes));
    }
}
