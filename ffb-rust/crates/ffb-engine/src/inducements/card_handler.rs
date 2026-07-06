/// 1:1 translation of `com.fumbbl.ffb.server.inducements.CardHandler`.
/// Abstract base for all card handlers — each concrete handler is responsible for one card type.
use ffb_model::inducement::card::Card;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;

pub trait CardHandler: Send + Sync {
    /// Java: handlerKey() — the name of the CardHandlerKey variant this handler is responsible for.
    fn handler_key_name(&self) -> &'static str;

    /// Java: getName() — returns the class simple name.
    fn get_name(&self) -> &'static str;

    /// Java: isResponsible(Card) — true if this handler should process the given card.
    fn is_responsible(&self, card: &Card) -> bool {
        card.handler_key_name() == Some(self.handler_key_name())
    }

    /// Java: allowsPlayer(Game, Card, Player) — true if the player is a valid target.
    /// Default: any player is allowed.
    fn allows_player(&self, _game: &Game, _card: &Card, _player_id: &str) -> bool {
        true
    }

    /// Java: activate(Card, IStep, Player) — apply the card effect. Returns true if activation
    /// completed synchronously (no dialog needed).
    /// Default: true (no effect; override in concrete handlers).
    fn activate_on_game(&self, _game: &mut Game, _card: &Card, _player_id: &str, _rng: &mut GameRng) -> bool {
        true
    }

    /// Java: deactivate(Card, IStep, Player) — remove the card effect.
    fn deactivate_on_game(&self, _game: &mut Game, _card: &Card, _player_id: &str) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    struct TestHandler;
    impl CardHandler for TestHandler {
        fn handler_key_name(&self) -> &'static str { "TEST_KEY" }
        fn get_name(&self) -> &'static str { "TestHandler" }
        fn activate_on_game(&self, _game: &mut Game, _card: &Card, _player_id: &str, _rng: &mut GameRng) -> bool { true }
    }

    #[test]
    fn is_responsible_matches_handler_key() {
        let h = TestHandler;
        let matching = Card::new("Test Card", Some("TEST_KEY"));
        let non_matching = Card::new("Other Card", Some("OTHER_KEY"));
        let no_key = Card::new("No Key Card", None::<&str>);
        assert!(h.is_responsible(&matching));
        assert!(!h.is_responsible(&non_matching));
        assert!(!h.is_responsible(&no_key));
    }

    #[test]
    fn allows_player_default_returns_true() {
        let h = TestHandler;
        let game = make_game();
        let card = Card::new("Test", Some("TEST_KEY"));
        assert!(h.allows_player(&game, &card, "player1"));
    }

    #[test]
    fn get_name_returns_handler_name() {
        let h = TestHandler;
        assert_eq!(h.get_name(), "TestHandler");
    }
}
