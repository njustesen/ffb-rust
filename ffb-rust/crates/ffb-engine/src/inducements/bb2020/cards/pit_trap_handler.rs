/// 1:1 translation of BB2020 `PitTrapHandler`.
use ffb_model::model::game::Game;
use ffb_model::inducement::card::Card;
use crate::inducements::card_handler::CardHandler;
use crate::step::framework::StepParameter;
use crate::step::util_server_injury::drop_player;

pub struct PitTrapHandler;

impl PitTrapHandler {
    pub fn new() -> Self { Self }
}

impl Default for PitTrapHandler {
    fn default() -> Self { Self::new() }
}

impl CardHandler for PitTrapHandler {
    fn handler_key_name(&self) -> &'static str { "PIT_TRAP" }

    fn get_name(&self) -> &'static str { "PitTrapHandler" }

    fn activate_on_game(&self, _game: &mut Game, _card: &Card, _player_id: &str, _rng: &mut ffb_model::util::rng::GameRng) -> bool {
        true
    }

    /// Java: `activate(Card, IStep, Player)`:
    /// `step.publishParameters(UtilServerInjury.dropPlayer(step, player, ApothecaryMode.DEFENDER))`.
    fn activation_parameters(
        &self, game: &mut Game, _card: &Card, player_id: &str, _rng: &mut ffb_model::util::rng::GameRng,
    ) -> Vec<StepParameter> {
        drop_player(game, player_id, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn is_responsible_for_correct_key() {
        let h = PitTrapHandler;
        let card = Card::new("Pit Trap", Some("PIT_TRAP"));
        assert!(h.is_responsible(&card));
        let other = Card::new("Other", Some("OTHER_KEY"));
        assert!(!h.is_responsible(&other));
    }

    #[test]
    fn allows_player_default_returns_true() {
        let h = PitTrapHandler;
        let game = make_game();
        let card = Card::new("Pit Trap", Some("PIT_TRAP"));
        assert!(h.allows_player(&game, &card, "player1"));
    }

    #[test]
    fn get_name_returns_struct_name() {
        let h = PitTrapHandler;
        assert_eq!(h.get_name(), "PitTrapHandler");
    }

    #[test]
    fn handler_key_name_is_pit_trap() {
        let h = PitTrapHandler::new();
        assert_eq!(h.handler_key_name(), "PIT_TRAP");
    }

    #[test]
    fn activate_on_game_returns_true() {
        let mut game = make_game();
        let h = PitTrapHandler::new();
        let card = Card::new("Pit Trap", Some("PIT_TRAP"));
        let result = h.activate_on_game(&mut game, &card, "player1", &mut ffb_model::util::rng::GameRng::new(0));
        assert!(result);
    }

    fn make_game_with_player(id: &str) -> Game {
        use ffb_model::enums::{PlayerGender, PlayerType, PlayerState, PS_STANDING};
        use ffb_model::model::player::Player;
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
        game
    }

    #[test]
    fn activation_parameters_drops_the_player_prone() {
        use ffb_model::enums::PS_PRONE;
        let mut game = make_game_with_player("player1");
        let h = PitTrapHandler::new();
        let card = Card::new("Pit Trap", Some("PIT_TRAP"));
        h.activation_parameters(&mut game, &card, "player1", &mut ffb_model::util::rng::GameRng::new(0));
        assert_eq!(game.field_model.player_state("player1").unwrap().base(), PS_PRONE);
    }

    #[test]
    fn activation_parameters_scatters_the_ball_when_player_is_carrying_it() {
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game_with_player("player1");
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        let h = PitTrapHandler::new();
        let card = Card::new("Pit Trap", Some("PIT_TRAP"));
        let params = h.activation_parameters(&mut game, &card, "player1", &mut ffb_model::util::rng::GameRng::new(0));
        assert!(params.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(_))));
    }
}
