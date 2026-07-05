use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Records the old acting player state into TargetSelectionState (if present),
/// then recovers the player's tackle zones.
///
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.shared.StepInitActivation.
pub struct StepInitActivation;

impl StepInitActivation {
    pub fn new() -> Self { Self }
}

impl Default for StepInitActivation {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitActivation {
    fn id(&self) -> StepId { StepId::InitActivation }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepInitActivation {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let player_state = match game.field_model.player_state(&player_id) {
            Some(s) => s,
            None => return StepOutcome::next(),
        };

        // Java: if (targetSelectionState != null) { targetSelectionState.setOldActingPlayerState(playerState); }
        if let Some(tss) = game.field_model.target_selection_state.as_mut() {
            tss.set_old_acting_player_state(Some(player_state));
        }

        // Java: game.getFieldModel().setPlayerState(player, playerState.recoverTacklezones())
        game.field_model.set_player_state(&player_id, player_state.recover_tacklezones());

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_STANDING, PlayerState};
    use ffb_model::model::player::Player;
    use ffb_model::model::target_selection_state::TargetSelectionState;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_acting_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
        game.acting_player.player_id = Some(id.into());
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_game();
        let mut step = StepInitActivation::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn recovers_tacklezones_on_player() {
        let mut game = make_game();
        add_acting_player(&mut game, "p1");
        let mut step = StepInitActivation::new();
        step.start(&mut game, &mut GameRng::new(0));
        let state = game.field_model.player_state("p1").unwrap();
        assert!(state.has_tacklezones());
    }

    #[test]
    fn stores_old_state_in_target_selection_state_when_present() {
        let mut game = make_game();
        add_acting_player(&mut game, "p1");
        let initial_state = game.field_model.player_state("p1").unwrap();
        game.field_model.target_selection_state = Some(TargetSelectionState::default());
        let mut step = StepInitActivation::new();
        step.start(&mut game, &mut GameRng::new(0));
        let tss = game.field_model.target_selection_state.as_ref().unwrap();
        assert_eq!(tss.get_old_acting_player_state(), Some(initial_state));
    }

    #[test]
    fn no_target_selection_state_no_panic() {
        let mut game = make_game();
        add_acting_player(&mut game, "p1");
        game.field_model.target_selection_state = None;
        let mut step = StepInitActivation::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn returns_next_step_action() {
        let mut game = make_game();
        add_acting_player(&mut game, "p1");
        let mut step = StepInitActivation::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_always_returns_false() {
        let mut step = StepInitActivation::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
