/// 1:1 translation of BB2016 `PitTrapHandler`.
use ffb_model::model::game::Game;
use ffb_model::inducement::card::Card;
use crate::inducements::card_handler::CardHandler;

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

    /// Java: step.publishParameters(UtilServerInjury.dropPlayer(step, player, ApothecaryMode.DEFENDER))
    /// headless: PitTrap injury pipeline requires RNG not in CardHandler trait — StepPlayCard must wire this when available
    fn activate_on_game(&self, _game: &mut Game, _card: &Card, _player_id: &str) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
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
}
