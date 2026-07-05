/// 1:1 translation of BB2020 `ChopBlockHandler`.
use ffb_model::model::game::Game;
use ffb_model::inducement::card::Card;
use ffb_model::util::util_player::UtilPlayer;
use crate::inducements::card_handler::CardHandler;

pub struct ChopBlockHandler;

impl ChopBlockHandler {
    pub fn new() -> Self { Self }
}

impl Default for ChopBlockHandler {
    fn default() -> Self { Self::new() }
}

impl CardHandler for ChopBlockHandler {
    fn handler_key_name(&self) -> &'static str { "CHOP_BLOCK" }

    fn get_name(&self) -> &'static str { "ChopBlockHandler" }

    /// Java: player is active, not prone/stunned, has adjacent blockable opponents.
    fn allows_player(&self, game: &Game, _card: &Card, player_id: &str) -> bool {
        let state = match game.field_model.player_state(player_id) {
            Some(s) => s,
            None => return false,
        };
        if !state.is_active() || state.is_prone_or_stunned() {
            return false;
        }
        let coord = match game.field_model.player_coordinate(player_id) {
            Some(c) => c,
            None => return false,
        };
        let is_home = game.team_home.players.iter().any(|p| p.id == player_id);
        let other_team = if is_home { &game.team_away } else { &game.team_home };
        !UtilPlayer::find_adjacent_blockable_players(game, other_team, coord).is_empty()
    }

    // Java: no activate override → default (return true, no effect)
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
        let h = ChopBlockHandler;
        let card = Card::new("Chop Block", Some("CHOP_BLOCK"));
        assert!(h.is_responsible(&card));
        let other = Card::new("Other", Some("OTHER_KEY"));
        assert!(!h.is_responsible(&other));
    }

    #[test]
    fn allows_player_returns_false_for_unknown_player() {
        let h = ChopBlockHandler;
        let game = make_game();
        let card = Card::new("Chop Block", Some("CHOP_BLOCK"));
        assert!(!h.allows_player(&game, &card, "nonexistent"));
    }

    #[test]
    fn get_name_returns_struct_name() {
        let h = ChopBlockHandler;
        assert_eq!(h.get_name(), "ChopBlockHandler");
    }

    #[test]
    fn activate_returns_true_by_default() {
        let mut game = make_game();
        let h = ChopBlockHandler;
        let card = Card::new("Chop Block", Some("CHOP_BLOCK"));
        assert!(h.activate_on_game(&mut game, &card, "player1"));
    }
}
