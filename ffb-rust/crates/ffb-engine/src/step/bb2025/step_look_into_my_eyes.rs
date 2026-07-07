/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepLookIntoMyEyes (BB2025).
///
/// Resolves the Look Into My Eyes skill: roll to steal the ball from the defender (2+ success).
///
/// Init params: PUSH_SELECT, GOTO_LABEL_ON_END.
/// Runtime params: END_TURN, END_PLAYER_ACTION.
///
/// Re-roll and EndPlayerAction sequence push are not translated; random agent always
/// declines re-rolls so the failure path collapses to NEXT_STEP.
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::report::mixed::report_look_into_my_eyes_roll::ReportLookIntoMyEyesRoll;
use ffb_model::report::mixed::report_skill_wasted::ReportSkillWasted;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepLookIntoMyEyes {
    /// Java: endPlayerAction — set by END_PLAYER_ACTION parameter.
    pub end_player_action: bool,
    /// Java: endTurn — set by END_TURN parameter.
    pub end_turn: bool,
    /// Java: pushSelect — PUSH_SELECT init parameter (push a Select sequence on failure).
    pub push_select: bool,
    /// Java: gotoOnEnd — GOTO_LABEL_ON_END init parameter.
    pub goto_on_end: String,
}

impl StepLookIntoMyEyes {
    pub fn new() -> Self {
        Self {
            end_player_action: false,
            end_turn: false,
            push_select: false,
            goto_on_end: String::new(),
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

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: super.handleCommand → re-roll path (AbstractStepWithReRoll)
        // Random agent always declines → fall through to execute_step
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
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
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
            return StepOutcome::next();
        }

        // Java: if (endTurn || endPlayerAction) → ReportSkillWasted + leave(endPlayerAction, endTurn)
        if self.end_turn || self.end_player_action {
            // Java: getResult().addReport(new ReportSkillWasted(actingPlayer.getPlayerId(), skill))
            game.report_list.add(ReportSkillWasted::new(
                Some(player_id.clone()),
                Some(SkillId::LookIntoMyEyes),
            ));
            Self::mark_skill_used(game, &player_id);
            // Stub: cancelPlayerAction + EndPlayerAction sequence not translated → NEXT_STEP
            return StepOutcome::next();
        }

        if game.defender_id.is_some() {
            // Java: roll = rollSkill(); successful = roll > 1
            let roll = rng.d6();
            let successful = roll > 1;

            // Java: getResult().addReport(new ReportLookIntoMyEyesRoll(playerId, successful, roll, reRolled))
            game.report_list.add(ReportLookIntoMyEyesRoll::new(
                Some(player_id.clone()),
                successful,
                roll,
                2,
                false, // headless: re-roll never taken
            ));

            if successful {
                if let Some(player_coord) = game.field_model.player_coordinate(&player_id) {
                    game.field_model.ball_coordinate = Some(player_coord);
                }
                // Java: leave(actingPlayer, skill, true, false) — getResult().addReport(new ReportSkillUse(..., true, LOOK_INTO_MY_EYES))
                game.report_list.add(ReportSkillUse::new(
                    Some(player_id.clone()),
                    SkillId::LookIntoMyEyes,
                    true,
                    SkillUse::LOOK_INTO_MY_EYES,
                ));
                Self::mark_skill_used(game, &player_id);
            } else {
                // Java: random agent declines re-roll → leave(false, false)
                Self::mark_skill_used(game, &player_id);
            }
        } else {
            // Java: no defender → leave(actingPlayer, skill, true, false)
            // Stub: EndPlayerAction sequence not translated → NEXT_STEP
            Self::mark_skill_used(game, &player_id);
        }

        StepOutcome::next()
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
    fn no_skill_returns_next_step() {
        let (mut game, _) = make_game_lime();
        game.team_home.players[0].starting_skills.clear();
        let mut step = StepLookIntoMyEyes::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_returns_next_step_and_marks_used() {
        let (mut game, actor_id) = make_game_lime();
        let mut step = StepLookIntoMyEyes::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
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
    fn failure_does_not_steal_ball() {
        let seed = seed_for_d6(1); // == 1, not > 1
        let (mut game, _) = make_game_lime();
        game.defender_id = Some("def".into());
        let ball_coord = FieldCoordinate::new(11, 7);
        game.field_model.ball_coordinate = Some(ball_coord);

        let mut step = StepLookIntoMyEyes::new();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.ball_coordinate, Some(ball_coord));
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
}
