/// 1:1 translation of com.fumbbl.ffb.server.step.action.pass.StepBombardier (COMMON).
///
/// Handles the Bombardier skill setup: if the acting player has Bombardier and is performing
/// a bomb throw action (ThrowBomb or HailMaryBomb), sets the appropriate bomb TurnMode.
///
/// No init or set_parameter needed. No dice rolls. Always returns NEXT_STEP.
use ffb_model::enums::{PlayerAction, SkillId, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepBombardier;

impl StepBombardier {
    pub fn new() -> Self { Self }
}

impl Default for StepBombardier {
    fn default() -> Self { Self::new() }
}

impl Step for StepBombardier {
    fn id(&self) -> StepId { StepId::Bombardier }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepBombardier {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // Java: if (!game.getTurnMode().isBombTurn() && (action == THROW_BOMB || action == HAIL_MARY_BOMB))
        if !game.turn_mode.is_bomb_turn() {
            let is_bomb_action = matches!(
                game.acting_player.player_action,
                Some(PlayerAction::ThrowBomb) | Some(PlayerAction::HailMaryBomb)
            );
            if is_bomb_action {
                // Java: actingPlayer.markSkillUsed(Bombardier)
                if let Some(ref pid) = game.acting_player.player_id.clone() {
                    if let Some(p) = game.team_home.players.iter_mut().find(|p| p.id == *pid) {
                        p.used_skills.insert(SkillId::Bombardier);
                    } else if let Some(p) = game.team_away.players.iter_mut().find(|p| p.id == *pid) {
                        p.used_skills.insert(SkillId::Bombardier);
                    }
                }

                // Java: set turn mode based on team and current turn mode
                let is_home = game.acting_player.player_id.as_deref()
                    .map(|id| game.team_home.players.iter().any(|p| p.id == id))
                    .unwrap_or(false);

                game.turn_mode = if is_home {
                    if game.turn_mode == TurnMode::Blitz { TurnMode::BombHomeBlitz } else { TurnMode::BombHome }
                } else {
                    if game.turn_mode == TurnMode::Blitz { TurnMode::BombAwayBlitz } else { TurnMode::BombAway }
                };
            }
        }

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::model::skill_def::SkillWithValue;

    fn make_game_with_bombardier() -> (Game, String) {
        let pid = "thrower".to_string();
        let mut home = test_team("home", 0);
        home.players.push(ffb_model::model::player::Player {
            id: pid.clone(), name: pid.clone(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Bombardier, value: None }],
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(PlayerAction::ThrowBomb);
        (game, pid)
    }

    #[test]
    fn throw_bomb_action_sets_bomb_home_turn_mode() {
        let (mut game, _) = make_game_with_bombardier();
        StepBombardier::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::BombHome);
    }

    #[test]
    fn blitz_turn_mode_becomes_bomb_home_blitz() {
        let (mut game, _) = make_game_with_bombardier();
        game.turn_mode = TurnMode::Blitz;
        StepBombardier::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::BombHomeBlitz);
    }

    #[test]
    fn bomb_turn_mode_is_not_changed() {
        let (mut game, _) = make_game_with_bombardier();
        game.turn_mode = TurnMode::BombHome;
        StepBombardier::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::BombHome);
    }

    #[test]
    fn non_bomb_action_does_not_change_turn_mode() {
        let (mut game, _) = make_game_with_bombardier();
        game.acting_player.player_action = Some(PlayerAction::Pass);
        game.turn_mode = TurnMode::Regular;
        StepBombardier::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Regular);
    }

    #[test]
    fn away_player_sets_bomb_away() {
        let home = test_team("home", 0);
        let mut away = test_team("away", 0);
        let pid = "away_thrower".to_string();
        away.players.push(ffb_model::model::player::Player {
            id: pid.clone(), name: pid.clone(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![],
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
                    ..Default::default()
});
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = false;
        game.acting_player.player_id = Some(pid);
        game.acting_player.player_action = Some(PlayerAction::ThrowBomb);
        StepBombardier::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::BombAway);
    }

    #[test]
    fn marks_bombardier_skill_used() {
        let (mut game, pid) = make_game_with_bombardier();
        StepBombardier::new().start(&mut game, &mut GameRng::new(0));
        let player = game.team_home.players.iter().find(|p| p.id == pid).unwrap();
        assert!(player.used_skills.contains(&SkillId::Bombardier));
    }

    #[test]
    fn always_returns_next_step() {
        let (mut game, _) = make_game_with_bombardier();
        let out = StepBombardier::new().start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
