/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepRaidingParty (BB2025).
///
/// Resolves the Raiding Party skill: move an open teammate to an adjacent square beside an opponent.
///
/// Init params: GOTO_LABEL_ON_FAILURE, GOTO_LABEL_ON_SUCCESS.
/// Runtime params: END_TURN, END_PLAYER_ACTION.
/// Commands: CLIENT_PLAYER_CHOICE (teammate selection), CLIENT_FIELD_COORDINATE (target square), CLIENT_END_TURN.
///
/// Stub: Sequence generation (PICK_UP + CATCH_SCATTER_THROW_IN push) not translated.
/// Behavior: NEXT_STEP immediately when no eligible players are found.
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepRaidingParty {
    /// Java: endPlayerAction — set by END_PLAYER_ACTION parameter.
    pub end_player_action: bool,
    /// Java: endTurn — set by END_TURN parameter.
    pub end_turn: bool,
    /// Java: goToLabelOnFailure — GOTO_LABEL_ON_FAILURE init parameter.
    pub goto_label_on_failure: String,
    /// Java: gotoLabelOnSuccess — GOTO_LABEL_ON_SUCCESS init parameter.
    pub goto_label_on_success: String,
    /// Java: playerId — selected teammate.
    pub player_id: Option<String>,
}

impl StepRaidingParty {
    pub fn new() -> Self {
        Self {
            end_player_action: false,
            end_turn: false,
            goto_label_on_failure: String::new(),
            goto_label_on_success: String::new(),
            player_id: None,
        }
    }
}

impl Default for StepRaidingParty {
    fn default() -> Self { Self::new() }
}

impl Step for StepRaidingParty {
    fn id(&self) -> StepId { StepId::RaidingParty }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { player_id } => {
                if player_id.is_empty() {
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
            StepParameter::GotoLabelOnSuccess(v)    => { self.goto_label_on_success = v.clone(); true }
            _ => false,
        }
    }
}

impl StepRaidingParty {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: skill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canMoveOpenTeamMate)
        let has_skill = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::RaidingParty) && !p.used_skills.contains(&SkillId::RaidingParty))
            .unwrap_or(false);

        if !has_skill {
            return StepOutcome::next();
        }

        // Java: if (endTurn || endPlayerAction) → GOTO_LABEL
        if self.end_turn || self.end_player_action {
            return StepOutcome::goto(&self.goto_label_on_failure);
        }

        // Stub: sequence generation (PICK_UP + CATCH_SCATTER_THROW_IN) not translated → NEXT_STEP
        StepOutcome::next()
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

    fn make_game_rp() -> (Game, String) {
        let pid = "actor".to_string();
        let mut home = test_team("home", 0);
        home.players.push(make_player(&pid, Some(SkillId::RaidingParty)));
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
        let (mut game, _) = make_game_rp();
        game.team_home.players[0].starting_skills.clear();
        let mut step = StepRaidingParty::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_goes_to_label() {
        let (mut game, _) = make_game_rp();
        let mut step = StepRaidingParty::new();
        step.goto_label_on_failure = "FAIL".into();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn with_skill_returns_next_step_stub() {
        let (mut game, _) = make_game_rp();
        let mut step = StepRaidingParty::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_wiring() {
        let mut step = StepRaidingParty::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("F".into())));
        assert_eq!(step.goto_label_on_failure, "F");
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("S".into())));
        assert_eq!(step.goto_label_on_success, "S");
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }
}
