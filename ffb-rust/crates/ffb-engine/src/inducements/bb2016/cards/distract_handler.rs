/// 1:1 translation of BB2016 `DistractHandler`.
use ffb_model::enums::CardEffect;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::inducement::card::Card;
use crate::inducements::card_handler::CardHandler;

pub struct DistractHandler;

impl DistractHandler {
    pub fn new() -> Self { Self }
}

impl Default for DistractHandler {
    fn default() -> Self { Self::new() }
}

impl CardHandler for DistractHandler {
    fn handler_key_name(&self) -> &'static str { "DISTRACT" }

    fn get_name(&self) -> &'static str { "DistractHandler" }

    /// Java: add DISTRACTED to opponent players within 3 squares (Chebyshev) of the card user.
    /// Java: `findAdjacentCoordinates(playerCoordinate, FieldCoordinateBounds.FIELD, 3, false)`
    /// — note the "3" here is a step count, not adjacency: this covers a 7x7 area, not just the
    /// 8 immediately-adjacent squares.
    fn activate_on_game(&self, game: &mut Game, _card: &Card, player_id: &str, _rng: &mut ffb_model::util::rng::GameRng) -> bool {
        let coord = match game.field_model.player_coordinate(player_id) {
            Some(c) => c,
            None => return true,
        };
        let is_home = game.team_home.players.iter().any(|p| p.id == player_id);
        let other_team_player_ids: Vec<String> = if is_home {
            game.team_away.players.iter().map(|p| p.id.clone()).collect()
        } else {
            game.team_home.players.iter().map(|p| p.id.clone()).collect()
        };

        const STEPS: i32 = 3;
        for dx in -STEPS..=STEPS {
            for dy in -STEPS..=STEPS {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let adj_coord = ffb_model::types::FieldCoordinate::new(coord.x + dx, coord.y + dy);
                if !adj_coord.is_on_pitch() {
                    continue;
                }
                if let Some(other_id) = game.field_model.player_at(adj_coord).cloned() {
                    if other_team_player_ids.contains(&other_id) {
                        game.field_model.add_card_effect(&other_id, CardEffect::Distracted);
                    }
                }
            }
        }
        true
    }

    /// Java: remove DISTRACTED from all affected players; also clear confused state if not skill-granted.
    fn deactivate_on_game(&self, game: &mut Game, _card: &Card, _player_id: &str) {
        let distracted: Vec<String> = game.field_model
            .find_players_with_card_effect(CardEffect::Distracted)
            .into_iter().map(|id| id.to_string()).collect();
        for id in distracted {
            game.field_model.remove_card_effect(&id, CardEffect::Distracted);
            // Java: if !player.hasSkillProperty(appliesConfusion) && playerState.isConfused()
            //       → fieldModel.setPlayerState(player, playerState.changeConfused(false))
            let applies_confusion = game.team_home.player(&id)
                .or_else(|| game.team_away.player(&id))
                .map(|p| p.has_skill_property(NamedProperties::APPLIES_CONFUSION))
                .unwrap_or(false);
            if !applies_confusion {
                if let Some(state) = game.field_model.player_state(&id) {
                    if state.is_confused() {
                        game.field_model.set_player_state(&id, state.change_confused(false));
                    }
                }
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn is_responsible_for_correct_key() {
        let h = DistractHandler;
        let card = Card::new("Distract", Some("DISTRACT"));
        assert!(h.is_responsible(&card));
        let other = Card::new("Other", Some("OTHER_KEY"));
        assert!(!h.is_responsible(&other));
    }

    #[test]
    fn activate_on_empty_field_returns_true() {
        let mut game = make_game();
        let h = DistractHandler;
        let card = Card::new("Distract", Some("DISTRACT"));
        assert!(h.activate_on_game(&mut game, &card, "nonexistent", &mut ffb_model::util::rng::GameRng::new(0)));
    }

    /// Regression: Java's DistractHandler.activate uses
    /// `findAdjacentCoordinates(playerCoordinate, FieldCoordinateBounds.FIELD, 3, false)` — a
    /// step count of 3, covering a 7x7 area, not just the 8 immediately-adjacent squares.
    /// An opponent 3 squares away (but not adjacent) must still be distracted.
    #[test]
    fn activate_distracts_opponent_three_squares_away() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PS_STANDING, PlayerState};
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        let user_coord = FieldCoordinate::new(10, 5);
        let far_coord = FieldCoordinate::new(13, 5); // 3 squares away in x, 0 in y
        game.team_home.players.push(Player { id: "home1".into(), ..Default::default() });
        game.team_away.players.push(Player { id: "away1".into(), ..Default::default() });
        game.field_model.set_player_coordinate("home1", user_coord);
        game.field_model.set_player_coordinate("away1", far_coord);
        game.field_model.set_player_state("away1", PlayerState::new(PS_STANDING));

        let h = DistractHandler;
        let card = Card::new("Distract", Some("DISTRACT"));
        h.activate_on_game(&mut game, &card, "home1", &mut ffb_model::util::rng::GameRng::new(0));

        assert!(game.field_model.has_card_effect("away1", CardEffect::Distracted));
    }

    /// Regression: an opponent further than 3 squares away must NOT be distracted.
    #[test]
    fn activate_does_not_distract_opponent_beyond_three_squares() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PS_STANDING, PlayerState};
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        let user_coord = FieldCoordinate::new(5, 5);
        let too_far_coord = FieldCoordinate::new(9, 5); // 4 squares away
        game.team_home.players.push(Player { id: "home1".into(), ..Default::default() });
        game.team_away.players.push(Player { id: "away1".into(), ..Default::default() });
        game.field_model.set_player_coordinate("home1", user_coord);
        game.field_model.set_player_coordinate("away1", too_far_coord);
        game.field_model.set_player_state("away1", PlayerState::new(PS_STANDING));

        let h = DistractHandler;
        let card = Card::new("Distract", Some("DISTRACT"));
        h.activate_on_game(&mut game, &card, "home1", &mut ffb_model::util::rng::GameRng::new(0));

        assert!(!game.field_model.has_card_effect("away1", CardEffect::Distracted));
    }

    #[test]
    fn deactivate_clears_confused_on_distracted_player() {
        use ffb_model::enums::PlayerState;
        use ffb_model::enums::PS_STANDING;
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 5);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.add_card_effect("p1", CardEffect::Distracted);
        let confused_state = PlayerState::new(PS_STANDING).change_confused(true);
        game.field_model.set_player_state("p1", confused_state);
        let h = DistractHandler;
        let card = Card::new("Distract", Some("DISTRACT"));
        h.deactivate_on_game(&mut game, &card, "p1");
        assert!(!game.field_model.has_card_effect("p1", CardEffect::Distracted));
        if let Some(state) = game.field_model.player_state("p1") {
            assert!(!state.is_confused());
        }
    }

    #[test]
    fn deactivate_removes_distracted_effect() {
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(10, 5));
        game.field_model.add_card_effect("p1", CardEffect::Distracted);
        let h = DistractHandler;
        let card = Card::new("Distract", Some("DISTRACT"));
        h.deactivate_on_game(&mut game, &card, "p1");
        assert!(!game.field_model.has_card_effect("p1", CardEffect::Distracted));
    }

    #[test]
    fn get_name_returns_struct_name() {
        let h = DistractHandler;
        assert_eq!(h.get_name(), "DistractHandler");
    }
}
