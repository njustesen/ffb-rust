/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepBalefulHex (BB2025).
///
/// Resolves the Baleful Hex skill: distract an opponent within 5 squares (confuse + deactivate) on 2+.
///
/// Init params: GOTO_LABEL_ON_FAILURE.
/// Runtime params: END_TURN, END_PLAYER_ACTION.
/// Commands: CLIENT_PLAYER_CHOICE (target selection or decline).
use ffb_model::enums::{SkillId, PlayerAction};
use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::mixed::report_baleful_hex_roll::ReportBalefulHexRoll;
use ffb_model::report::mixed::report_skill_wasted::ReportSkillWasted;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepBalefulHex {
    /// Java: endPlayerAction — set by END_PLAYER_ACTION parameter.
    pub end_player_action: bool,
    /// Java: endTurn — set by END_TURN parameter.
    pub end_turn: bool,
    /// Java: goToLabelOnFailure — GOTO_LABEL_ON_FAILURE init parameter.
    pub goto_label_on_failure: String,
    /// Java: playerId — set from CLIENT_PLAYER_CHOICE command.
    pub player_id: Option<String>,
    /// Java: roll — the skill die result.
    pub roll: i32,
}

impl StepBalefulHex {
    pub fn new() -> Self {
        Self {
            end_player_action: false,
            end_turn: false,
            goto_label_on_failure: String::new(),
            player_id: None,
            roll: 0,
        }
    }
}

impl Default for StepBalefulHex {
    fn default() -> Self { Self::new() }
}

impl Step for StepBalefulHex {
    fn id(&self) -> StepId { StepId::BalefulHex }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_PLAYER_CHOICE → playerId set or declined (empty id → SKIP_STEP / NEXT_STEP)
        match action {
            Action::SelectPlayer { player_id } => {
                if player_id.is_empty() {
                    // Java: getResult().addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skill, false, MAKE_OPPONENT_MISS_TURN))
                    let acting_id = game.acting_player.player_id.clone();
                    game.report_list.add(ReportSkillUse::new(
                        acting_id,
                        SkillId::BalefulHex,
                        false,
                        SkillUse::MAKE_OPPONENT_MISS_TURN,
                    ));
                    return StepOutcome::next();
                }
                self.player_id = Some(player_id.clone());
            }
            _ => {}
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

impl StepBalefulHex {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: skill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canMakeOpponentMissTurn)
        let has_skill = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::BalefulHex) && !p.used_skills.contains(&SkillId::BalefulHex))
            .unwrap_or(false);

        if !has_skill {
            return StepOutcome::next();
        }

        // Java: markActionUsed(game, actingPlayer)
        Self::mark_action_used(game, &player_id);

        // Java: if (endTurn || endPlayerAction) → ReportSkillWasted + GOTO_LABEL + markSkillUsed
        if self.end_turn || self.end_player_action {
            // Java: getResult().addReport(new ReportSkillWasted(actingPlayer.getPlayerId(), skill))
            game.report_list.add(ReportSkillWasted::new(
                Some(player_id.clone()),
                Some(SkillId::BalefulHex),
            ));
            Self::mark_skill_used(game, &player_id);
            return StepOutcome::goto(&self.goto_label_on_failure);
        }

        // Java: if (!StringTool.isProvided(playerId)) → find eligible opponents (within 5 squares)
        if self.player_id.is_none() {
            let eligibles = Self::find_eligible_targets(game, &player_id);
            if eligibles.is_empty() {
                return StepOutcome::next();
            }
            // Java: getResult().addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skill, true, MAKE_OPPONENT_MISS_TURN))
            game.report_list.add(ReportSkillUse::new(
                Some(player_id.clone()),
                SkillId::BalefulHex,
                true,
                SkillUse::MAKE_OPPONENT_MISS_TURN,
            ));
            if eligibles.len() == 1 {
                self.player_id = Some(eligibles[0].clone());
            } else {
                // Multiple targets: show dialog; random agent will decline
                return StepOutcome::cont();
            }
        }

        // Java: roll = rollSkill(); successful = roll > 1
        if self.player_id.is_some() {
            self.roll = rng.d6();
            let successful = self.roll > 1;

            // Java: getResult().addReport(new ReportBalefulHexRoll(actingPlayer.getPlayerId(), playerId, successful, roll, reRolled))
            game.report_list.add(ReportBalefulHexRoll::new(
                Some(player_id.clone()),
                successful,
                self.roll,
                2, // minimum roll: roll > 1 means 2+
                false, // re_rolled: simplified — re-roll handling deferred
                self.player_id.clone(),
            ));

            if successful {
                if let Some(ref target_id) = self.player_id.clone() {
                    if let Some(state) = game.field_model.player_state(target_id) {
                        // Java: changeConfused(true).changeActive(false)
                        game.field_model.set_player_state(
                            target_id,
                            state.change_confused(true).change_active(false),
                        );
                    }
                }
                Self::mark_skill_used(game, &player_id);
            } else {
                // Failure: random agent declines re-roll → mark used
                Self::mark_skill_used(game, &player_id);
            }
        }

        StepOutcome::next()
    }

    /// Java: findPlayers — opponents within 5 Chebyshev steps.
    fn find_eligible_targets(game: &Game, actor_id: &str) -> Vec<String> {
        let actor_coord = match game.field_model.player_coordinate(actor_id) {
            Some(c) => c,
            None => return vec![],
        };
        game.inactive_team().players.iter()
            .filter(|p| {
                game.field_model.player_coordinate(&p.id)
                    .map(|c| c.distance_in_steps(actor_coord) <= 5)
                    .unwrap_or(false)
            })
            .map(|p| p.id.clone())
            .collect()
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
                p.used_skills.insert(SkillId::BalefulHex);
            }
        } else if let Some(p) = game.team_away.player_mut(player_id) {
            p.used_skills.insert(SkillId::BalefulHex);
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

    fn make_game_bh() -> (Game, String) {
        let pid = "actor".to_string();
        let mut home = test_team("home", 0);
        home.players.push(make_player(&pid, Some(SkillId::BalefulHex)));
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
        let (mut game, _) = make_game_bh();
        game.team_home.players[0].starting_skills.clear();
        let mut step = StepBalefulHex::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_goes_to_label() {
        let (mut game, _) = make_game_bh();
        let mut step = StepBalefulHex::new();
        step.goto_label_on_failure = "FAIL".into();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn no_eligible_targets_returns_next_step() {
        let (mut game, _) = make_game_bh();
        let mut step = StepBalefulHex::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn success_confuses_and_deactivates_target() {
        let seed = seed_for_d6(4); // > 1
        let (mut game, actor_id) = make_game_bh();
        let target_id = "opp1".to_string();
        game.team_away.players.push(make_player(&target_id, None));
        let near = FieldCoordinate::new(12, 7); // 2 steps away
        game.field_model.set_player_coordinate(&target_id, near);
        game.field_model.set_player_state(&target_id, PlayerState::new(PS_STANDING).change_active(true));

        let mut step = StepBalefulHex::new();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);

        let state = game.field_model.player_state(&target_id).unwrap();
        assert!(state.is_confused(), "target should be confused");
        assert!(!state.is_active(), "target should be inactive");
        assert!(game.team_home.player(&actor_id).unwrap().used_skills.contains(&SkillId::BalefulHex));
    }

    #[test]
    fn failure_does_not_confuse() {
        let seed = seed_for_d6(1); // == 1, not > 1
        let (mut game, _) = make_game_bh();
        let target_id = "opp1".to_string();
        game.team_away.players.push(make_player(&target_id, None));
        let near = FieldCoordinate::new(12, 7);
        game.field_model.set_player_coordinate(&target_id, near);
        game.field_model.set_player_state(&target_id, PlayerState::new(PS_STANDING).change_active(true));

        let mut step = StepBalefulHex::new();
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        let state = game.field_model.player_state(&target_id).unwrap();
        assert!(!state.is_confused(), "target should NOT be confused on failure");
    }

    #[test]
    fn set_parameter_wiring() {
        let mut step = StepBalefulHex::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("X".into())));
        assert_eq!(step.goto_label_on_failure, "X");
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }

    #[test]
    fn skill_wasted_report_on_end_player_action() {
        use ffb_model::report::report_id::ReportId;
        let (mut game, _) = make_game_bh();
        let mut step = StepBalefulHex::new();
        step.goto_label_on_failure = "FAIL".into();
        step.end_player_action = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_WASTED),
            "expected SKILL_WASTED when end_player_action is set");
    }

    #[test]
    fn baleful_hex_roll_and_skill_use_reports_added_on_success() {
        use ffb_model::report::report_id::ReportId;
        let seed = seed_for_d6(4); // > 1 → success
        let (mut game, _) = make_game_bh();
        let target_id = "opp1".to_string();
        game.team_away.players.push(make_player(&target_id, None));
        game.field_model.set_player_coordinate(&target_id, FieldCoordinate::new(12, 7));
        game.field_model.set_player_state(&target_id, PlayerState::new(PS_STANDING).change_active(true));
        let mut step = StepBalefulHex::new();
        step.start(&mut game, &mut GameRng::new(seed));
        assert!(game.report_list.has_report(ReportId::SKILL_USE),
            "expected SKILL_USE report when eligible target found");
        assert!(game.report_list.has_report(ReportId::BALEFUL_HEX),
            "expected BALEFUL_HEX roll report");
    }
}
