/// 1:1 translation of BB2016 `ForceShieldHandler`.
use ffb_model::model::game::Game;
use ffb_model::inducement::card::Card;
use ffb_model::util::util_player::UtilPlayer;
use crate::inducements::card_handler::CardHandler;

pub struct ForceShieldHandler;

impl ForceShieldHandler {
    pub fn new() -> Self { Self }
}

impl Default for ForceShieldHandler {
    fn default() -> Self { Self::new() }
}

impl CardHandler for ForceShieldHandler {
    fn handler_key_name(&self) -> &'static str { "FORCE_SHIELD" }

    fn get_name(&self) -> &'static str { "ForceShieldHandler" }

    /// Java: UtilPlayer.hasBall(game, player)
    fn allows_player(&self, game: &Game, _card: &Card, player_id: &str) -> bool {
        UtilPlayer::has_ball(game, player_id)
    }

    // Java: no activate override → default (return true, no effect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn is_responsible_for_correct_key() {
        let h = ForceShieldHandler;
        let card = Card::new("Force Shield", Some("FORCE_SHIELD"));
        assert!(h.is_responsible(&card));
        let other = Card::new("Other", Some("OTHER_KEY"));
        assert!(!h.is_responsible(&other));
    }

    #[test]
    fn allows_player_false_when_no_ball() {
        let h = ForceShieldHandler;
        let game = make_game();
        let card = Card::new("Force Shield", Some("FORCE_SHIELD"));
        assert!(!h.allows_player(&game, &card, "player1"));
    }

    #[test]
    fn allows_player_true_when_has_ball() {
        use ffb_model::types::FieldCoordinate;
        let h = ForceShieldHandler;
        let mut game = make_game();
        let coord = FieldCoordinate::new(13, 7);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;
        let card = Card::new("Force Shield", Some("FORCE_SHIELD"));
        assert!(h.allows_player(&game, &card, "p1"));
    }

    #[test]
    fn get_name_returns_struct_name() {
        let h = ForceShieldHandler;
        assert_eq!(h.get_name(), "ForceShieldHandler");
    }
}
