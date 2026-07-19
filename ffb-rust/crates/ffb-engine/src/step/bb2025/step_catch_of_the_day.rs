/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepCatchOfTheDay (BB2025).
///
/// Resolves the Catch of the Day skill: grab a moving ball within 3 squares on a 3+ roll.
///
/// Init params: GOTO_LABEL_ON_FAILURE.
/// Runtime params: END_TURN, END_PLAYER_ACTION.
use ffb_model::enums::{ReRollSource, SkillId, PlayerAction};
use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::mixed::report_catch_of_the_day_roll::ReportCatchOfTheDayRoll;
use ffb_model::report::mixed::report_skill_wasted::ReportSkillWasted;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

pub struct StepCatchOfTheDay {
    /// Java: endPlayerAction — set by END_PLAYER_ACTION parameter.
    pub end_player_action: bool,
    /// Java: endTurn — set by END_TURN parameter.
    pub end_turn: bool,
    /// Java: goToLabelOnFailure — GOTO_LABEL_ON_FAILURE init parameter.
    pub goto_label_on_failure: String,
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepCatchOfTheDay {
    pub fn new() -> Self {
        Self {
            end_player_action: false,
            end_turn: false,
            goto_label_on_failure: String::new(),
            re_rolled_action: None,
            re_roll_source: None,
        }
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

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: super.handleCommand → AbstractStepWithReRoll: CLIENT_USE_RE_ROLL(false) clears
        // the re-roll source (player declined).
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
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
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let is_rerolled = self.re_rolled_action.as_deref() == Some("CATCH_OF_THE_DAY");

        // Java: skill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canGetBallOnGround)
        //       if (skill != null || getReRolledAction() == CATCH_OF_THE_DAY)
        let has_skill = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::CatchOfTheDay) && !p.used_skills.contains(&SkillId::CatchOfTheDay))
            .unwrap_or(false);

        if !has_skill && !is_rerolled {
            return StepOutcome::next();
        }

        // Java: if (endTurn || endPlayerAction) → ReportSkillWasted + GOTO_LABEL + markUsages
        if self.end_turn || self.end_player_action {
            game.report_list.add(ReportSkillWasted::new(Some(player_id.clone()), Some(SkillId::CatchOfTheDay)));
            Self::mark_action_used(game, &player_id);
            Self::mark_skill_used(game, &player_id);
            return StepOutcome::goto(&self.goto_label_on_failure);
        }

        if is_rerolled {
            // Java: if (getReRollSource() == null || !UtilServerReRoll.useReRoll(...)) {
            //         getResult().setSound(SoundId.BOUNCE); return; }
            let declined_or_failed = match self.re_roll_source.clone() {
                Some(ref source_name) => {
                    let source = ReRollSource::new(source_name.as_str());
                    !use_reroll(game, &source, &player_id)
                }
                None => true,
            };
            if declined_or_failed {
                // client-only: SoundId.BOUNCE — sound playback is client-side only (dropped).
                return StepOutcome::next();
            }
        } else {
            // Java: markUsages(game, actingPlayer, skill) — only on the non-re-rolled pass.
            Self::mark_action_used(game, &player_id);
            Self::mark_skill_used(game, &player_id);
        }

        // Java: addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skill, true, GET_BALL_ON_GROUND))
        game.report_list.add(ReportSkillUse::new(Some(player_id.clone()), SkillId::CatchOfTheDay, true, SkillUse::GET_BALL_ON_GROUND));

        let player_coord = match game.field_model.player_coordinate(&player_id) {
            Some(c) => c,
            None => return StepOutcome::next(),
        };
        let ball_coord = match game.field_model.ball_coordinate {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        // Java: if (game.getFieldModel().isBallMoving() && distance <= 3)
        let ball_moving = game.field_model.ball_moving;
        let in_range = player_coord.distance_in_steps(ball_coord) <= 3;

        if ball_moving && in_range {
            // Java: roll = getDiceRoller().rollDice(6); success = roll >= 3
            let roll = rng.d6();
            let success = roll >= 3;

            if success {
                game.field_model.ball_coordinate = Some(player_coord);
                game.field_model.ball_moving = false;
            }

            // Java: addReport(new ReportCatchOfTheDayRoll(actingPlayer.getPlayerId(), success, roll, 3, reRolled))
            game.report_list.add(ReportCatchOfTheDayRoll::new(Some(player_id.clone()), success, roll, 3, is_rerolled));

            if !success {
                // Java: else if (getReRolledAction() != CATCH_OF_THE_DAY &&
                //         UtilServerReRoll.askForReRollIfAvailable(..., CATCH_OF_THE_DAY, 3, false)) {
                //         setReRolledAction(CATCH_OF_THE_DAY); getResult().setNextAction(CONTINUE); }
                //       else { getResult().setSound(SoundId.BOUNCE); }
                if !is_rerolled {
                    if let Some(prompt) = ask_for_reroll_if_available(game, "CATCH_OF_THE_DAY", 3, false) {
                        self.re_rolled_action = Some("CATCH_OF_THE_DAY".into());
                        self.re_roll_source = Some("TRR".into());
                        return StepOutcome::cont().with_prompt(prompt);
                    }
                }
                // client-only: SoundId.BOUNCE — sound playback is client-side only (dropped).
            }
        } else {
            // Java: not in range → addReport(new ReportSkillWasted(actingPlayer.getPlayerId(), skill))
            game.report_list.add(ReportSkillWasted::new(Some(player_id.clone()), Some(SkillId::CatchOfTheDay)));
        }

        StepOutcome::next()
    }

    fn mark_action_used(game: &mut Game, player_id: &str) {
        let action = game.acting_player.player_action;
        let turn = game.turn_data_mut();
        match action {
            Some(PlayerAction::Blitz | PlayerAction::BlitzMove) => turn.blitz_used = true,
            Some(PlayerAction::Pass | PlayerAction::PassMove) => turn.pass_used = true,
            Some(PlayerAction::HandOver | PlayerAction::HandOverMove) => turn.hand_over_used = true,
            Some(PlayerAction::Foul | PlayerAction::FoulMove) => turn.foul_used = true,
            Some(PlayerAction::ThrowTeamMate | PlayerAction::ThrowTeamMateMove) => turn.ttm_used = true,
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
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_game_cotd() -> (Game, String) {
        let pid = "actor".to_string();
        let mut home = test_team("home", 0);
        home.players.push(make_player(&pid, Some(SkillId::CatchOfTheDay)));
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(10, 7));
        (game, pid)
    }

    fn seed_for_d6_gte3(at_least: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() >= at_least { return s; }
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
    fn ball_not_moving_does_not_move_ball() {
        let (mut game, pid) = make_game_cotd();
        let ball_coord = FieldCoordinate::new(11, 7);
        game.field_model.ball_coordinate = Some(ball_coord);
        game.field_model.ball_moving = false;
        let mut step = StepCatchOfTheDay::new();
        let _ = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.ball_coordinate, Some(ball_coord));
        let _ = &pid;
    }

    #[test]
    fn success_moves_ball_to_player() {
        let seed = seed_for_d6_gte3(3);
        let (mut game, pid) = make_game_cotd();
        let ball_coord = FieldCoordinate::new(11, 7);
        game.field_model.ball_coordinate = Some(ball_coord);
        game.field_model.ball_moving = true;

        let mut step = StepCatchOfTheDay::new();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        let player_coord = FieldCoordinate::new(10, 7);
        assert_eq!(game.field_model.ball_coordinate, Some(player_coord));
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
        assert!(game.field_model.ball_moving);
    }

    #[test]
    fn out_of_range_does_not_move_ball() {
        let (mut game, _) = make_game_cotd();
        let ball_coord = FieldCoordinate::new(15, 7); // 5 steps away
        game.field_model.ball_coordinate = Some(ball_coord);
        game.field_model.ball_moving = true;

        let mut step = StepCatchOfTheDay::new();
        let _ = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.ball_coordinate, Some(ball_coord));
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
    fn out_of_range_adds_skill_wasted_report() {
        use ffb_model::report::report_id::ReportId;
        let (mut game, _) = make_game_cotd();
        let ball_coord = FieldCoordinate::new(15, 7); // 5 steps away — out of range
        game.field_model.ball_coordinate = Some(ball_coord);
        game.field_model.ball_moving = true;
        let mut step = StepCatchOfTheDay::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_WASTED));
    }

    // ── Bug fix regression tests ──────────────────────────────────────────
    // Previously a failed roll never offered a team re-roll (Java:
    // `askForReRollIfAvailable(..., CATCH_OF_THE_DAY, 3, false)` → StepAction.CONTINUE),
    // and the skill/action-used bookkeeping was unconditional on every call instead of
    // being gated on `reRolledAction` as in Java's `markUsages` branch.

    #[test]
    fn failed_roll_with_team_reroll_available_offers_reroll() {
        let seed = seed_for_d6_lt3();
        let (mut game, pid) = make_game_cotd();
        game.turn_data_home.rerolls = 1;
        let ball_coord = FieldCoordinate::new(11, 7);
        game.field_model.ball_coordinate = Some(ball_coord);
        game.field_model.ball_moving = true;

        let mut step = StepCatchOfTheDay::new();
        let out = step.start(&mut game, &mut GameRng::new(seed));

        assert_eq!(out.action, StepAction::Continue, "expected CONTINUE to offer a re-roll");
        assert!(out.prompt.is_some());
        assert_eq!(step.re_rolled_action.as_deref(), Some("CATCH_OF_THE_DAY"));
        // Ball must not have moved yet — the roll is being re-attempted.
        assert_eq!(game.field_model.ball_coordinate, Some(ball_coord));
        assert!(game.field_model.ball_moving);
        let _ = &pid;
    }

    #[test]
    fn declining_reroll_leaves_ball_unmoved_and_returns_next_step() {
        let seed = seed_for_d6_lt3();
        let (mut game, _) = make_game_cotd();
        game.turn_data_home.rerolls = 1;
        let ball_coord = FieldCoordinate::new(11, 7);
        game.field_model.ball_coordinate = Some(ball_coord);
        game.field_model.ball_moving = true;

        let mut step = StepCatchOfTheDay::new();
        step.start(&mut game, &mut GameRng::new(seed));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.ball_coordinate, Some(ball_coord));
        assert!(game.field_model.ball_moving);
    }

    #[test]
    fn skill_used_is_marked_exactly_once_across_reroll_round_trip() {
        let seed_fail = seed_for_d6_lt3();
        let (mut game, pid) = make_game_cotd();
        game.turn_data_home.rerolls = 1;
        let ball_coord = FieldCoordinate::new(11, 7);
        game.field_model.ball_coordinate = Some(ball_coord);
        game.field_model.ball_moving = true;

        let mut step = StepCatchOfTheDay::new();
        step.start(&mut game, &mut GameRng::new(seed_fail));
        assert!(
            game.team_home.player(&pid).unwrap().used_skills.contains(&SkillId::CatchOfTheDay),
            "skill must be marked used on the first (non-re-rolled) pass"
        );

        // Re-roll accepted (re_roll_source stays Some("TRR")); second roll may succeed or fail,
        // but the skill must not be double-marked or cause a panic.
        let seed_second = seed_for_d6_gte3(3);
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(seed_second));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
