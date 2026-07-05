/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepBlackInk (BB2025).
///
/// Applies the Black Ink skill: gaze at an adjacent opponent to distract them (confuse).
///
/// Init params: GOTO_LABEL_ON_FAILURE.
/// Runtime params: END_TURN, END_PLAYER_ACTION.
/// Commands: CLIENT_PLAYER_CHOICE (player selection or decline), CLIENT_END_TURN.
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepBlackInk {
    /// Java: endPlayerAction — set by END_PLAYER_ACTION parameter.
    pub end_player_action: bool,
    /// Java: endTurn — set by END_TURN parameter.
    pub end_turn: bool,
    /// Java: goToLabelOnFailure — GOTO_LABEL_ON_FAILURE init parameter.
    pub goto_label_on_failure: String,
    /// Java: playerId — set from CLIENT_PLAYER_CHOICE command.
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
        // Java: CLIENT_PLAYER_CHOICE — player selected or declined (empty id → SKIP_STEP)
        // Java: CLIENT_END_TURN → endTurn = true, EXECUTE_STEP
        match action {
            Action::SelectPlayer { player_id } => {
                if player_id.is_empty() {
                    // Declined: restore old player state (not translated), report not used, NEXT_STEP
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
        // Java: skill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canGazeAutomatically)
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

        // Java: if (endTurn || endPlayerAction || actingPlayer.isStandingUp()) → GOTO_LABEL
        if self.end_turn || self.end_player_action || game.acting_player.standing_up {
            return StepOutcome::goto(&self.goto_label_on_failure);
        }

        // Java: if (!StringTool.isProvided(playerId)) → find eligible opponents
        if self.player_id.is_none() {
            let eligibles = Self::find_adjacent_opponents(game, &player_id);
            if eligibles.is_empty() {
                return StepOutcome::next();
            }
            if eligibles.len() == 1 {
                self.player_id = Some(eligibles[0].clone());
            } else {
                // More than one target: show dialog; random agent will decline next command
                return StepOutcome::cont();
            }
        }

        // Java: player = game.getPlayerById(playerId); confuse + mark skill used
        if let Some(ref target_id) = self.player_id.clone() {
            if let Some(state) = game.field_model.player_state(target_id) {
                game.field_model.set_player_state(target_id, state.change_confused(true));
            }
            // Java: actingPlayer.markSkillUsed(skill)
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

    /// Java: findPlayers — adjacent standing-or-prone non-distracted opponents.
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
                        (s.is_standing() || s.is_prone())
                            && !s.is_distracted()
                            && c.is_adjacent(actor_coord)
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
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "pos".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skill.map(|s| vec![SkillWithValue { skill_id: s, value: None }])
                .unwrap_or_default(),
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        }
    }

    fn make_game_with_black_ink() -> (Game, String) {
        let pid = "actor".to_string();
        let mut home = test_team("home", 0);
        home.players.push(make_player(&pid, Some(SkillId::BlackInk)));
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(10, 7));
        (game, pid)
    }

    #[test]
    fn no_skill_returns_next_step() {
        let (mut game, _) = make_game_with_black_ink();
        game.team_home.players[0].starting_skills.clear();
        let mut step = StepBlackInk::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_goes_to_label() {
        let (mut game, _) = make_game_with_black_ink();
        let mut step = StepBlackInk::new();
        step.goto_label_on_failure = "FAIL".into();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("FAIL"));
    }

    #[test]
    fn end_player_action_goes_to_label() {
        let (mut game, _) = make_game_with_black_ink();
        let mut step = StepBlackInk::new();
        step.goto_label_on_failure = "FAIL".into();
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn no_eligible_opponents_returns_next_step() {
        let (mut game, _) = make_game_with_black_ink();
        let mut step = StepBlackInk::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn confuses_auto_selected_target() {
        let (mut game, actor_id) = make_game_with_black_ink();
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
    fn set_parameter_wiring() {
        let mut step = StepBlackInk::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("X".into())));
        assert_eq!(step.goto_label_on_failure, "X");
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }
}
