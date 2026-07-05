/// 1:1 translation of BB2020 `CustardPieHandler`.
use ffb_model::model::game::Game;
use ffb_model::inducement::card::Card;
use ffb_model::util::util_player::UtilPlayer;
use crate::inducements::card_handler::CardHandler;

pub struct CustardPieHandler;

impl CustardPieHandler {
    pub fn new() -> Self { Self }
}

impl Default for CustardPieHandler {
    fn default() -> Self { Self::new() }
}

impl CardHandler for CustardPieHandler {
    fn handler_key_name(&self) -> &'static str { "CUSTARD_PIE" }

    fn get_name(&self) -> &'static str { "CustardPieHandler" }

    /// Java: has adjacent standing or prone players of own team.
    fn allows_player(&self, game: &Game, _card: &Card, player_id: &str) -> bool {
        let coord = match game.field_model.player_coordinate(player_id) {
            Some(c) => c,
            None => return false,
        };
        let is_home = game.team_home.players.iter().any(|p| p.id == player_id);
        let own_team = if is_home { &game.team_home } else { &game.team_away };
        !UtilPlayer::find_adjacent_blockable_players(game, own_team, coord).is_empty()
    }

    /// Java: game.getFieldModel().setPlayerState(player, playerState.changeHypnotized(true))
    fn activate_on_game(&self, game: &mut Game, _card: &Card, player_id: &str) -> bool {
        if let Some(state) = game.field_model.player_state(player_id) {
            game.field_model.set_player_state(player_id, state.change_hypnotized(true));
        }
        true
    }

    /// Java: if playerState.isHypnotized() → changeHypnotized(false)
    fn deactivate_on_game(&self, game: &mut Game, _card: &Card, player_id: &str) {
        if let Some(state) = game.field_model.player_state(player_id) {
            if state.is_hypnotized() {
                game.field_model.set_player_state(player_id, state.change_hypnotized(false));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn is_responsible_for_correct_key() {
        let h = CustardPieHandler;
        let card = Card::new("Custard Pie", Some("CUSTARD_PIE"));
        assert!(h.is_responsible(&card));
    }

    #[test]
    fn allows_player_false_for_unknown_player() {
        let h = CustardPieHandler;
        let game = make_game();
        let card = Card::new("Custard Pie", Some("CUSTARD_PIE"));
        assert!(!h.allows_player(&game, &card, "nonexistent"));
    }

    #[test]
    fn get_name_returns_struct_name() {
        let h = CustardPieHandler;
        assert_eq!(h.get_name(), "CustardPieHandler");
    }

    #[test]
    fn activate_sets_hypnotized_state() {
        use ffb_model::enums::{PS_STANDING, PlayerState};
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(13, 7));
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));
        let h = CustardPieHandler;
        let card = Card::new("Custard Pie", Some("CUSTARD_PIE"));
        h.activate_on_game(&mut game, &card, "p1");
        let state = game.field_model.player_state("p1").unwrap();
        assert!(state.is_hypnotized());
    }

    #[test]
    fn deactivate_clears_hypnotized_state() {
        use ffb_model::enums::{PS_STANDING, PlayerState};
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(13, 7));
        let hypno = PlayerState::new(PS_STANDING).change_hypnotized(true);
        game.field_model.set_player_state("p1", hypno);
        let h = CustardPieHandler;
        let card = Card::new("Custard Pie", Some("CUSTARD_PIE"));
        h.deactivate_on_game(&mut game, &card, "p1");
        let state = game.field_model.player_state("p1").unwrap();
        assert!(!state.is_hypnotized());
    }
}
