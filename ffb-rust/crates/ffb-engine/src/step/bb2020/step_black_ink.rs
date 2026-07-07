/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.StepBlackInk (BB2020).
///
/// Applies the Black Ink skill: gaze at an adjacent opponent to confuse them.
///
/// Differs from BB2025: `find_adjacent_opponents` does NOT filter on `!is_distracted()`
/// (BB2020 uses SkillUse REMOVE_TACKLEZONE which targets any adjacent standing/prone opponent).
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_skill_use::ReportSkillUse;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepBlackInk {
    pub end_player_action: bool,
    pub end_turn: bool,
    pub goto_label_on_failure: String,
    pub player_id: Option<String>,
}

impl StepBlackInk {
    pub fn new() -> Self {
        Self {
            end_player_action: false,
            end_turn: false,
            goto_label_on_failure: String::new(),
            player_id: None,
        }
    }
}

impl Default for StepBlackInk {
    fn default() -> Self { Self::new() }
}

impl Step for StepBlackInk {
    fn id(&self) -> StepId { StepId::BlackInk }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { player_id } => {
                if player_id.is_empty() {
                    // Java: addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skill, false, SkillUse.REMOVE_TACKLEZONE));
                    if let Some(ref pid) = game.acting_player.player_id.clone() {
                        game.report_list.add(ReportSkillUse::new(
                            Some(pid.clone()),
                            SkillId::BlackInk,
                            false,
                            SkillUse::REMOVE_TACKLEZONE,
                        ));
                    }
                    return StepOutcome::next();
                }
                self.player_id = Some(player_id.clone());
            }
            Action::EndTurn => { self.end_turn = true; }
            _ => {}
        }
        self.execute_step(game)
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

impl StepBlackInk {
    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let has_skill = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::BlackInk) && !p.used_skills.contains(&SkillId::BlackInk))
            .unwrap_or(false);

        if !has_skill {
            return StepOutcome::next();
        }

        if self.end_turn || self.end_player_action || game.acting_player.standing_up {
            return StepOutcome::goto(&self.goto_label_on_failure);
        }

        if self.player_id.is_none() {
            let eligibles = Self::find_adjacent_opponents(game, &player_id);
            if eligibles.is_empty() {
                return StepOutcome::next();
            }
            // Java: getResult().addReport(new ReportSkillUse(actingPlayer.getPlayerId(), skill, true, SkillUse.REMOVE_TACKLEZONE));
            game.report_list.add(ReportSkillUse::new(
                Some(player_id.clone()),
                SkillId::BlackInk,
                true,
                SkillUse::REMOVE_TACKLEZONE,
            ));
            if eligibles.len() == 1 {
                self.player_id = Some(eligibles[0].clone());
            } else {
                return StepOutcome::cont();
            }
        }

        if let Some(ref target_id) = self.player_id.clone() {
            if let Some(state) = game.field_model.player_state(target_id) {
                game.field_model.set_player_state(target_id, state.change_confused(true));
            }
            let is_home = game.team_home.player(&player_id).is_some();
            if is_home {
                if let Some(p) = game.team_home.player_mut(&player_id) {
                    p.used_skills.insert(SkillId::BlackInk);
                }
            } else if let Some(p) = game.team_away.player_mut(&player_id) {
                p.used_skills.insert(SkillId::BlackInk);
            }
        }

        StepOutcome::next()
    }

    /// BB2020: no `!is_distracted()` filter (unlike BB2025).
    fn find_adjacent_opponents(game: &Game, actor_id: &str) -> Vec<String> {
        let actor_coord = match game.field_model.player_coordinate(actor_id) {
            Some(c) => c,
            None => return vec![],
        };
        game.inactive_team().players.iter()
            .filter(|p| {
                let state = game.field_model.player_state(&p.id);
                let coord = game.field_model.player_coordinate(&p.id);
                match (state, coord) {
                    (Some(s), Some(c)) => {
                        (s.is_standing() || s.is_prone()) && c.is_adjacent(actor_coord)
                    }
                    _ => false,
                }
            })
            .map(|p| p.id.clone())
            .collect()
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

    fn make_game_bi() -> (Game, String) {
        let pid = "actor".to_string();
        let mut home = test_team("home", 0);
        home.players.push(make_player(&pid, Some(SkillId::BlackInk)));
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2020);
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(10, 7));
        (game, pid)
    }

    #[test]
    fn no_skill_returns_next_step() {
        let (mut game, _) = make_game_bi();
        game.team_home.players[0].starting_skills.clear();
        let mut step = StepBlackInk::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_goes_to_label() {
        let (mut game, _) = make_game_bi();
        let mut step = StepBlackInk::new();
        step.goto_label_on_failure = "FAIL".into();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn no_eligible_opponents_returns_next_step() {
        let (mut game, _) = make_game_bi();
        let mut step = StepBlackInk::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn confuses_auto_selected_target() {
        let (mut game, actor_id) = make_game_bi();
        let target_id = "opp1".to_string();
        game.team_away.players.push(make_player(&target_id, None));
        let adj = FieldCoordinate::new(11, 7);
        game.field_model.set_player_coordinate(&target_id, adj);
        game.field_model.set_player_state(&target_id, PlayerState::new(PS_STANDING).change_active(true));

        let mut step = StepBlackInk::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);

        let state = game.field_model.player_state(&target_id).unwrap();
        assert!(state.is_confused(), "target should be confused");
        assert!(game.team_home.player(&actor_id).unwrap().used_skills.contains(&SkillId::BlackInk));
    }

    #[test]
    fn targets_already_confused_player_bb2020() {
        // BB2020 does NOT filter out already-distracted players
        let (mut game, _actor_id) = make_game_bi();
        let target_id = "opp1".to_string();
        game.team_away.players.push(make_player(&target_id, None));
        let adj = FieldCoordinate::new(11, 7);
        game.field_model.set_player_coordinate(&target_id, adj);
        // Pre-confused (distracted)
        game.field_model.set_player_state(&target_id, PlayerState::new(PS_STANDING).change_confused(true));

        let mut step = StepBlackInk::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should still find the target (no distracted filter in BB2020)
        assert_eq!(out.action, StepAction::NextStep);
        assert!(step.player_id.is_some(), "should have auto-selected the distracted opponent");
    }

    #[test]
    fn set_parameter_wiring() {
        let mut step = StepBlackInk::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("X".into())));
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(!step.set_parameter(&StepParameter::NrOfDice(2)));
    }

    // ── report wiring ─────────────────────────────────────────────────────────

    #[test]
    fn skill_use_report_added_when_eligible_target_found() {
        use ffb_model::report::report_id::ReportId;
        let (mut game, _actor_id) = make_game_bi();
        let target_id = "opp1".to_string();
        game.team_away.players.push(make_player(&target_id, None));
        let adj = FieldCoordinate::new(11, 7);
        game.field_model.set_player_coordinate(&target_id, adj);
        game.field_model.set_player_state(&target_id, PlayerState::new(PS_STANDING).change_active(true));

        let mut step = StepBlackInk::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::SKILL_USE),
            "SKILL_USE(REMOVE_TACKLEZONE) report must be added when eligible target found");
    }

    #[test]
    fn skill_use_report_not_used_when_player_declines() {
        use ffb_model::report::report_id::ReportId;
        let (mut game, _) = make_game_bi();
        let mut step = StepBlackInk::new();
        step.handle_command(
            &Action::SelectPlayer { player_id: "".to_string() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(game.report_list.has_report(ReportId::SKILL_USE),
            "SKILL_USE(used=false) report must be added when player declines");
    }
}
