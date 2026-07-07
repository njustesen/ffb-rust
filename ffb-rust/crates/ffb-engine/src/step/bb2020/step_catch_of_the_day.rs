/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepCatchOfTheDay` (BB2020).
///
/// Resolves the Catch of the Day skill: grab a moving ball within 3 squares on a 3+ roll.
///
/// Differs from BB2025 in `mark_action_used`:
///  - `ThrowTeamMate` → `pass_used` (grouped with PASS, not `ttm_used`)
///  - Includes `KickEmBlitz` → `blitz_used` and `KickTeamMate` → `ktm_used`
///  - No PUNT case
use ffb_model::enums::{SkillId, PlayerAction};
use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::mixed::report_catch_of_the_day_roll::ReportCatchOfTheDayRoll;
use ffb_model::report::mixed::report_skill_wasted::ReportSkillWasted;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepCatchOfTheDay {
    pub end_player_action: bool,
    pub end_turn: bool,
    pub goto_label_on_failure: String,
}

impl StepCatchOfTheDay {
    pub fn new() -> Self {
        Self { end_player_action: false, end_turn: false, goto_label_on_failure: String::new() }
    }
}

impl Default for StepCatchOfTheDay {
    fn default() -> Self { Self::new() }
}

impl Step for StepCatchOfTheDay {
    fn id(&self) -> StepId { StepId::CatchOfTheDay }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v)               => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)       => { self.end_player_action = *v; true }
            StepParameter::GotoLabelOnFailure(v)    => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepCatchOfTheDay {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let has_skill = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::CatchOfTheDay) && !p.used_skills.contains(&SkillId::CatchOfTheDay))
            .unwrap_or(false);

        if !has_skill {
            return StepOutcome::next();
        }

        Self::mark_action_used(game, &player_id);
        Self::mark_skill_used(game, &player_id);

        if self.end_turn || self.end_player_action {
            // Java: addReport(new ReportSkillWasted(actingPlayer.getPlayerId(), skill))
            game.report_list.add(ReportSkillWasted::new(Some(player_id.clone()), Some(SkillId::CatchOfTheDay)));
            return StepOutcome::goto(&self.goto_label_on_failure);
        }

        let player_coord = match game.field_model.player_coordinate(&player_id) {
            Some(c) => c,
            None => return StepOutcome::next(),
        };
        let ball_coord = match game.field_model.ball_coordinate {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        let ball_moving = game.field_model.ball_moving;
        let in_range = player_coord.distance_in_steps(ball_coord) <= 3;

        if ball_moving && in_range {
            // Java: addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skill, true, GET_BALL_ON_GROUND))
            game.report_list.add(ReportSkillUse::new(Some(player_id.clone()), SkillId::CatchOfTheDay, true, SkillUse::GET_BALL_ON_GROUND));

            let roll = rng.d6();
            let success = roll >= 3;

            if success {
                game.field_model.ball_coordinate = Some(player_coord);
                game.field_model.ball_moving = false;
            }

            // Java: addReport(new ReportCatchOfTheDayRoll(actingPlayer.getPlayerId(), success, roll, 3, reRolled))
            game.report_list.add(ReportCatchOfTheDayRoll::new(Some(player_id.clone()), success, roll, 3, false));
        } else {
            // Java: not in range → addReport(new ReportSkillWasted(actingPlayer.getPlayerId(), skill))
            game.report_list.add(ReportSkillWasted::new(Some(player_id.clone()), Some(SkillId::CatchOfTheDay)));
        }

        StepOutcome::next()
    }

    /// BB2020: ThrowTeamMate → pass_used; includes KickEmBlitz/KickTeamMate; no PUNT.
    fn mark_action_used(game: &mut Game, player_id: &str) {
        let action = game.acting_player.player_action;
        let turn = game.turn_data_mut();
        match action {
            Some(PlayerAction::Blitz | PlayerAction::BlitzMove | PlayerAction::KickEmBlitz) => turn.blitz_used = true,
            Some(PlayerAction::KickTeamMate | PlayerAction::KickTeamMateMove) => turn.ktm_used = true,
            Some(
                PlayerAction::Pass | PlayerAction::PassMove |
                PlayerAction::ThrowTeamMate | PlayerAction::ThrowTeamMateMove
            ) => turn.pass_used = true,
            Some(PlayerAction::HandOver | PlayerAction::HandOverMove) => turn.hand_over_used = true,
            Some(PlayerAction::Foul | PlayerAction::FoulMove) => turn.foul_used = true,
            _ => {}
        }
        let _ = player_id;
    }

    fn mark_skill_used(game: &mut Game, player_id: &str) {
        let is_home = game.team_home.player(player_id).is_some();
        if is_home {
            if let Some(p) = game.team_home.player_mut(player_id) {
                p.used_skills.insert(SkillId::CatchOfTheDay);
            }
        } else if let Some(p) = game.team_away.player_mut(player_id) {
            p.used_skills.insert(SkillId::CatchOfTheDay);
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
            ..Default::default()
        }
    }

    fn make_game_cotd() -> (Game, String) {
        let pid = "actor".to_string();
        let mut home = test_team("home", 0);
        home.players.push(make_player(&pid, Some(SkillId::CatchOfTheDay)));
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2020);
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(10, 7));
        (game, pid)
    }

    fn seed_for_d6_gte3() -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() >= 3 { return s; }
        }
        panic!("no seed found");
    }

    fn seed_for_d6_lt3() -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() < 3 { return s; }
        }
        panic!("no seed found");
    }

    #[test]
    fn no_skill_returns_next_step() {
        let (mut game, _) = make_game_cotd();
        game.team_home.players[0].starting_skills.clear();
        let mut step = StepCatchOfTheDay::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_goes_to_label() {
        let (mut game, _) = make_game_cotd();
        let mut step = StepCatchOfTheDay::new();
        step.goto_label_on_failure = "FAIL".into();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn success_moves_ball_to_player() {
        let seed = seed_for_d6_gte3();
        let (mut game, pid) = make_game_cotd();
        let ball_coord = FieldCoordinate::new(11, 7);
        game.field_model.ball_coordinate = Some(ball_coord);
        game.field_model.ball_moving = true;

        let mut step = StepCatchOfTheDay::new();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(10, 7)));
        assert!(!game.field_model.ball_moving);
        assert!(game.team_home.player(&pid).unwrap().used_skills.contains(&SkillId::CatchOfTheDay));
    }

    #[test]
    fn failure_does_not_move_ball() {
        let seed = seed_for_d6_lt3();
        let (mut game, _) = make_game_cotd();
        let ball_coord = FieldCoordinate::new(11, 7);
        game.field_model.ball_coordinate = Some(ball_coord);
        game.field_model.ball_moving = true;

        let mut step = StepCatchOfTheDay::new();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.ball_coordinate, Some(ball_coord));
    }

    #[test]
    fn mark_action_used_ttm_sets_pass_used() {
        let (mut game, _) = make_game_cotd();
        game.acting_player.player_action = Some(PlayerAction::ThrowTeamMate);
        StepCatchOfTheDay::mark_action_used(&mut game, "actor");
        assert!(game.turn_data_mut().pass_used);
        assert!(!game.turn_data_mut().ttm_used);
    }

    #[test]
    fn set_parameter_wiring() {
        let mut step = StepCatchOfTheDay::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("F".into())));
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(!step.set_parameter(&StepParameter::NrOfDice(2)));
    }

    #[test]
    fn end_turn_adds_skill_wasted_report() {
        use ffb_model::report::report_id::ReportId;
        let (mut game, _) = make_game_cotd();
        let mut step = StepCatchOfTheDay::new();
        step.goto_label_on_failure = "FAIL".into();
        step.end_turn = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_WASTED));
    }

    #[test]
    fn success_adds_skill_use_and_cotd_roll_reports() {
        use ffb_model::report::report_id::ReportId;
        let seed = seed_for_d6_gte3();
        let (mut game, _) = make_game_cotd();
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(11, 7));
        game.field_model.ball_moving = true;
        let mut step = StepCatchOfTheDay::new();
        step.start(&mut game, &mut GameRng::new(seed));
        assert!(game.report_list.has_report(ReportId::SKILL_USE));
        assert!(game.report_list.has_report(ReportId::CATCH_OF_THE_DAY));
    }
}
