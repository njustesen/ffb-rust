/// 1:1 translation of BB2020 `IllegalSubstitutionHandler`.
use ffb_model::enums::{CardEffect, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::inducement::card::Card;
use crate::inducements::card_handler::CardHandler;

pub struct IllegalSubstitutionHandler;

impl IllegalSubstitutionHandler {
    pub fn new() -> Self { Self }
}

impl Default for IllegalSubstitutionHandler {
    fn default() -> Self { Self::new() }
}

impl CardHandler for IllegalSubstitutionHandler {
    fn handler_key_name(&self) -> &'static str { "ILLEGAL_SUBSTITUTION" }

    fn get_name(&self) -> &'static str { "IllegalSubstitutionHandler" }

    /// Java: game.setTurnMode(TurnMode.ILLEGAL_SUBSTITUTION); return false (step continues dialog).
    fn activate_on_game(&self, game: &mut Game, _card: &Card, _player_id: &str) -> bool {
        game.turn_mode = TurnMode::IllegalSubstitution;
        false
    }

    /// Java: remove ILLEGALLY_SUBSTITUTED card effect from all players.
    fn deactivate_on_game(&self, game: &mut Game, _card: &Card, _player_id: &str) {
        let affected: Vec<String> = game.field_model
            .find_players_with_card_effect(CardEffect::IllegallySubstituted)
            .into_iter().map(|id| id.to_string()).collect();
        for id in affected {
            game.field_model.remove_card_effect(&id, CardEffect::IllegallySubstituted);
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
        let h = IllegalSubstitutionHandler;
        let card = Card::new("Illegal Substitution", Some("ILLEGAL_SUBSTITUTION"));
        assert!(h.is_responsible(&card));
    }

    #[test]
    fn allows_player_default_returns_true() {
        let h = IllegalSubstitutionHandler;
        let game = make_game();
        let card = Card::new("Illegal Substitution", Some("ILLEGAL_SUBSTITUTION"));
        assert!(h.allows_player(&game, &card, "player1"));
    }

    #[test]
    fn activate_sets_illegal_substitution_turn_mode() {
        let h = IllegalSubstitutionHandler;
        let mut game = make_game();
        let card = Card::new("Illegal Substitution", Some("ILLEGAL_SUBSTITUTION"));
        let result = h.activate_on_game(&mut game, &card, "player1");
        assert!(!result, "activate returns false (dialog needed)");
        assert_eq!(game.turn_mode, TurnMode::IllegalSubstitution);
    }

    #[test]
    fn get_name_returns_struct_name() {
        let h = IllegalSubstitutionHandler;
        assert_eq!(h.get_name(), "IllegalSubstitutionHandler");
    }
}
