/// 1:1 translation of BB2020 `WitchBrewHandler`.
use ffb_model::enums::CardEffect;
use ffb_model::model::game::Game;
use ffb_model::inducement::card::Card;
use crate::inducements::card_handler::CardHandler;

pub struct WitchBrewHandler;

impl WitchBrewHandler {
    pub fn new() -> Self { Self }
}

impl Default for WitchBrewHandler {
    fn default() -> Self { Self::new() }
}

impl CardHandler for WitchBrewHandler {
    fn handler_key_name(&self) -> &'static str { "WITCH_BREW" }

    fn get_name(&self) -> &'static str { "WitchBrewHandler" }

    /// Java: rollCardEffect() → interpretWitchBrewRoll(roll) → addCardEffect(player, effect).
    /// DEFERRED(card-activate-witch-brew-dice): CardHandler::activate_on_game has no RNG parameter;
    /// dice rolling requires IStep context. StepPlayCard must wire this when processing card effects.
    fn activate_on_game(&self, _game: &mut Game, _card: &Card, _player_id: &str) -> bool {
        true
    }

    /// Java: remove SEDATIVE and MAD_CAP_MUSHROOM_POTION if present.
    fn deactivate_on_game(&self, game: &mut Game, _card: &Card, player_id: &str) {
        if game.field_model.has_card_effect(player_id, CardEffect::Sedative) {
            game.field_model.remove_card_effect(player_id, CardEffect::Sedative);
        }
        if game.field_model.has_card_effect(player_id, CardEffect::MadCapMushroomPotion) {
            game.field_model.remove_card_effect(player_id, CardEffect::MadCapMushroomPotion);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn is_responsible_for_correct_key() {
        let h = WitchBrewHandler;
        let card = Card::new("Witch Brew", Some("WITCH_BREW"));
        assert!(h.is_responsible(&card));
        let other = Card::new("Other", Some("OTHER_KEY"));
        assert!(!h.is_responsible(&other));
    }

    #[test]
    fn allows_player_default_returns_true() {
        let h = WitchBrewHandler;
        let game = make_game();
        let card = Card::new("Witch Brew", Some("WITCH_BREW"));
        assert!(h.allows_player(&game, &card, "player1"));
    }

    #[test]
    fn activate_returns_true() {
        let mut game = make_game();
        let h = WitchBrewHandler;
        let card = Card::new("Witch Brew", Some("WITCH_BREW"));
        assert!(h.activate_on_game(&mut game, &card, "player1"));
    }

    #[test]
    fn deactivate_removes_sedative() {
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(10, 5));
        game.field_model.add_card_effect("p1", CardEffect::Sedative);
        assert!(game.field_model.has_card_effect("p1", CardEffect::Sedative));
        let h = WitchBrewHandler;
        let card = Card::new("Witch Brew", Some("WITCH_BREW"));
        h.deactivate_on_game(&mut game, &card, "p1");
        assert!(!game.field_model.has_card_effect("p1", CardEffect::Sedative));
    }

    #[test]
    fn deactivate_removes_mad_cap_mushroom_potion() {
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(10, 5));
        game.field_model.add_card_effect("p1", CardEffect::MadCapMushroomPotion);
        assert!(game.field_model.has_card_effect("p1", CardEffect::MadCapMushroomPotion));
        let h = WitchBrewHandler;
        let card = Card::new("Witch Brew", Some("WITCH_BREW"));
        h.deactivate_on_game(&mut game, &card, "p1");
        assert!(!game.field_model.has_card_effect("p1", CardEffect::MadCapMushroomPotion));
    }

    #[test]
    fn deactivate_is_noop_when_no_effects() {
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(10, 5));
        let h = WitchBrewHandler;
        let card = Card::new("Witch Brew", Some("WITCH_BREW"));
        h.deactivate_on_game(&mut game, &card, "p1");
        assert!(!game.field_model.has_card_effect("p1", CardEffect::Sedative));
        assert!(!game.field_model.has_card_effect("p1", CardEffect::MadCapMushroomPotion));
    }

    #[test]
    fn get_name_returns_struct_name() {
        let h = WitchBrewHandler;
        assert_eq!(h.get_name(), "WitchBrewHandler");
    }
}
