/// 1:1 translation of BB2020 `RabbitsFootHandler`.
use ffb_model::model::game::Game;
use ffb_model::inducement::card::Card;
use ffb_model::model::property::named_properties::NamedProperties;
use crate::inducements::card_handler::CardHandler;

pub struct RabbitsFootHandler;

impl RabbitsFootHandler {
    pub fn new() -> Self { Self }
}

impl Default for RabbitsFootHandler {
    fn default() -> Self { Self::new() }
}

impl CardHandler for RabbitsFootHandler {
    fn handler_key_name(&self) -> &'static str { "RABBITS_FOOT" }

    fn get_name(&self) -> &'static str { "RabbitsFootHandler" }

    /// Java: !player.hasSkillProperty(NamedProperties.preventCardRabbitsFoot)
    fn allows_player(&self, game: &Game, _card: &Card, player_id: &str) -> bool {
        game.player(player_id)
            .map(|p| !p.has_skill_property(NamedProperties::PREVENT_CARD_RABBITS_FOOT))
            .unwrap_or(false)
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
        let h = RabbitsFootHandler;
        let card = Card::new("Rabbit's Foot", Some("RABBITS_FOOT"));
        assert!(h.is_responsible(&card));
        let other = Card::new("Other", Some("OTHER_KEY"));
        assert!(!h.is_responsible(&other));
    }

    #[test]
    fn allows_player_false_when_player_not_in_game() {
        let h = RabbitsFootHandler;
        let game = make_game();
        let card = Card::new("Rabbit's Foot", Some("RABBITS_FOOT"));
        assert!(!h.allows_player(&game, &card, "nonexistent"));
    }

    #[test]
    fn allows_player_true_for_player_without_prevent_property() {
        use ffb_model::model::player::Player;
        use ffb_model::model::player_status::PlayerStatus;
        use ffb_model::types::FieldCoordinate;
        let h = RabbitsFootHandler;
        let mut game = make_game();
        let player = Player {
            id: "p1".into(), name: "Player One".into(), nr: 1,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None,
            player_status: PlayerStatus::ACTIVE,
            ..Default::default()
};
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(10, 5));
        let card = Card::new("Rabbit's Foot", Some("RABBITS_FOOT"));
        assert!(h.allows_player(&game, &card, "p1"));
    }

    #[test]
    fn activate_returns_true_by_default() {
        let mut game = make_game();
        let h = RabbitsFootHandler;
        let card = Card::new("Rabbit's Foot", Some("RABBITS_FOOT"));
        assert!(h.activate_on_game(&mut game, &card, "player1", &mut ffb_model::util::rng::GameRng::new(0)));
    }

    #[test]
    fn get_name_returns_struct_name() {
        let h = RabbitsFootHandler;
        assert_eq!(h.get_name(), "RabbitsFootHandler");
    }
}
