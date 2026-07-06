/// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerCards.
///
/// Static utility methods for playing and deactivating inducement cards.
use ffb_model::enums::InducementDuration;
use ffb_model::events::GameEvent;
use ffb_model::inducement::card::Card;
use ffb_model::model::game::Game;

pub struct UtilServerCards;

impl UtilServerCards {
    pub fn new() -> Self { Self }

    /// Java: UtilServerCards.deactivateCard (for a single card by name, in home or away set).
    /// Deactivates the card, emits GameEvent::CardDeactivated, calls the handler's deactivate_on_game.
    /// Returns the emitted event if the card was found and deactivated.
    pub fn deactivate_card(game: &mut Game, card_name: &str) -> Option<GameEvent> {
        use crate::factory::card_handler_factory::CardHandlerFactory;

        let card_clone: Option<Card>;
        let player_id: Option<String>;
        let home;

        if game.turn_data_home.inducement_set.is_active(card_name) {
            card_clone = game.turn_data_home.inducement_set.get_active_card(card_name).cloned();
            home = true;
        } else if game.turn_data_away.inducement_set.is_active(card_name) {
            card_clone = game.turn_data_away.inducement_set.get_active_card(card_name).cloned();
            home = false;
        } else {
            return None;
        }

        let card = card_clone?;

        player_id = if card.handler_key_name().map_or(false, |_| true) {
            game.field_model.find_player_with_card(card_name).map(|s| s.to_string())
        } else {
            None
        };

        // Deactivate in inducement set
        if home {
            game.turn_data_home.inducement_set.deactivate_card(card_name);
        } else {
            game.turn_data_away.inducement_set.deactivate_card(card_name);
        }

        // Update field model: remove or keep card based on remains_in_play
        if card.is_remains_in_play() {
            if let Some(pid) = &player_id {
                game.field_model.keep_deactivated_card(pid, card_name);
            }
        } else if let Some(pid) = &player_id {
            game.field_model.remove_card(pid, card_name);
        }

        // Call handler cleanup
        let mut factory = CardHandlerFactory::new();
        factory.initialize(game.rules);
        if let Some(handler) = factory.for_card(&card) {
            handler.deactivate_on_game(game, &card, player_id.as_deref().unwrap_or(""));
        }

        Some(GameEvent::CardDeactivated { card_id: card_name.to_string() })
    }

    /// Java: StepEndTurn.deactivateCards(InducementDuration, isHomeTurnEnding).
    /// Deactivates all active cards with the given duration. For UNTIL_END_OF_OPPONENTS_TURN,
    /// only deactivates the opponent's cards.
    pub fn deactivate_cards(
        game: &mut Game,
        duration: InducementDuration,
        is_home_turn_ending: bool,
    ) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // Collect names to deactivate from home set
        let home_names: Vec<String> = game.turn_data_home.inducement_set
            .get_active_card_objects()
            .into_iter()
            .filter(|card| card.get_duration() == Some(duration))
            .filter(|_| {
                duration != InducementDuration::UntilEndOfOpponentsTurn || !is_home_turn_ending
            })
            .map(|card| card.name.clone())
            .collect();

        // Collect names to deactivate from away set
        let away_names: Vec<String> = game.turn_data_away.inducement_set
            .get_active_card_objects()
            .into_iter()
            .filter(|card| card.get_duration() == Some(duration))
            .filter(|_| {
                duration != InducementDuration::UntilEndOfOpponentsTurn || is_home_turn_ending
            })
            .map(|card| card.name.clone())
            .collect();

        for name in home_names.iter().chain(away_names.iter()) {
            if let Some(ev) = Self::deactivate_card(game, name) {
                events.push(ev);
            }
        }

        events
    }

    /// Java: UtilServerCards.deactivateCard(GameState, Player) — deactivate the card for a player.
    /// Finds the first active card (home or away) whose handler's allows_player returns true for player_id.
    pub fn deactivate_card_for_player(game: &mut Game, player_id: &str) -> Option<GameEvent> {
        use crate::factory::card_handler_factory::CardHandlerFactory;

        let home_names: Vec<String> = game.turn_data_home.inducement_set.get_active_cards()
            .iter().map(|s| s.to_string()).collect();
        let away_names: Vec<String> = game.turn_data_away.inducement_set.get_active_cards()
            .iter().map(|s| s.to_string()).collect();

        let mut factory = CardHandlerFactory::new();
        factory.initialize(game.rules);

        for card_name in home_names.iter().chain(away_names.iter()) {
            let card = game.turn_data_home.inducement_set.get_active_card(card_name)
                .or_else(|| game.turn_data_away.inducement_set.get_active_card(card_name))
                .cloned();
            if let Some(card) = card {
                if let Some(handler) = factory.for_card(&card) {
                    if handler.allows_player(game, &card, player_id) {
                        return Self::deactivate_card(game, card_name);
                    }
                }
            }
        }
        None
    }

    /// Java: StepCatchScatterThrowIn.deactivateCards() — deactivate WHILE_HOLDING_THE_BALL
    /// cards from players who no longer hold the ball.
    pub fn deactivate_while_holding_ball(game: &mut Game) -> Vec<GameEvent> {
        use ffb_model::util::util_player::UtilPlayer;

        let mut events = Vec::new();

        // Collect all (player_id, card_name) pairs where the player no longer holds the ball
        // and the card has WHILE_HOLDING_THE_BALL duration.
        let to_deactivate: Vec<(String, String)> = game.field_model.player_cards.iter()
            .flat_map(|(player_id, cards)| {
                cards.iter()
                    .filter(|card| card.get_duration() == Some(InducementDuration::WhileHoldingTheBall))
                    .map(|card| (player_id.clone(), card.name.clone()))
                    .collect::<Vec<_>>()
            })
            .filter(|(player_id, _)| !UtilPlayer::has_ball(game, player_id))
            .collect();

        for (_, card_name) in &to_deactivate {
            if let Some(ev) = Self::deactivate_card(game, card_name) {
                events.push(ev);
            }
        }

        events
    }
}

impl Default for UtilServerCards {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn make_card(name: &str, duration: InducementDuration) -> Card {
        Card::new(name, None::<&str>).with_duration(duration)
    }

    #[test]
    fn struct_can_be_created() {
        let _ = UtilServerCards::new();
    }

    #[test]
    fn default_creates_instance() {
        let _ = UtilServerCards::default();
    }

    #[test]
    fn deactivate_card_unknown_returns_none() {
        let mut game = make_game();
        assert!(UtilServerCards::deactivate_card(&mut game, "No Such Card").is_none());
    }

    #[test]
    fn deactivate_card_home_active_emits_event() {
        let mut game = make_game();
        let card = make_card("Chop Block", InducementDuration::UntilEndOfTurn);
        game.turn_data_home.inducement_set.add_available_card("Chop Block");
        game.turn_data_home.inducement_set.activate_card_full(card);
        let ev = UtilServerCards::deactivate_card(&mut game, "Chop Block");
        assert!(ev.is_some());
        assert_eq!(ev.unwrap(), GameEvent::CardDeactivated { card_id: "Chop Block".to_string() });
        assert!(!game.turn_data_home.inducement_set.is_active("Chop Block"));
        assert!(game.turn_data_home.inducement_set.is_deactivated("Chop Block"));
    }

    #[test]
    fn deactivate_card_away_active_emits_event() {
        let mut game = make_game();
        let card = make_card("Distract", InducementDuration::UntilEndOfOpponentsTurn);
        game.turn_data_away.inducement_set.add_available_card("Distract");
        game.turn_data_away.inducement_set.activate_card_full(card);
        let ev = UtilServerCards::deactivate_card(&mut game, "Distract");
        assert!(ev.is_some());
        assert!(!game.turn_data_away.inducement_set.is_active("Distract"));
    }

    #[test]
    fn deactivate_cards_until_end_of_turn_deactivates_matching() {
        let mut game = make_game();
        let card_a = make_card("Witch Brew", InducementDuration::UntilEndOfTurn);
        let card_b = make_card("Force Shield", InducementDuration::UntilEndOfOpponentsTurn);
        game.turn_data_home.inducement_set.add_available_card("Witch Brew");
        game.turn_data_home.inducement_set.activate_card_full(card_a);
        game.turn_data_home.inducement_set.add_available_card("Force Shield");
        game.turn_data_home.inducement_set.activate_card_full(card_b);
        let events = UtilServerCards::deactivate_cards(&mut game, InducementDuration::UntilEndOfTurn, true);
        assert_eq!(events.len(), 1);
        assert!(!game.turn_data_home.inducement_set.is_active("Witch Brew"));
        assert!(game.turn_data_home.inducement_set.is_active("Force Shield"));
    }

    #[test]
    fn deactivate_cards_until_end_of_opponents_turn_home_skips_home_cards_when_home_ending() {
        let mut game = make_game();
        // Home card with UNTIL_END_OF_OPPONENTS_TURN: skip when home turn is ending (it's home's card)
        let card = make_card("Chop Block", InducementDuration::UntilEndOfOpponentsTurn);
        game.turn_data_home.inducement_set.add_available_card("Chop Block");
        game.turn_data_home.inducement_set.activate_card_full(card);
        let events = UtilServerCards::deactivate_cards(&mut game, InducementDuration::UntilEndOfOpponentsTurn, true);
        assert_eq!(events.len(), 0, "Home card should not be deactivated when home turn is ending");
    }

    #[test]
    fn deactivate_cards_until_end_of_opponents_turn_away_deactivates_when_home_ending() {
        let mut game = make_game();
        // Away card with UNTIL_END_OF_OPPONENTS_TURN: deactivate when home turn is ending
        let card = make_card("Distract", InducementDuration::UntilEndOfOpponentsTurn);
        game.turn_data_away.inducement_set.add_available_card("Distract");
        game.turn_data_away.inducement_set.activate_card_full(card);
        let events = UtilServerCards::deactivate_cards(&mut game, InducementDuration::UntilEndOfOpponentsTurn, true);
        assert_eq!(events.len(), 1);
        assert!(!game.turn_data_away.inducement_set.is_active("Distract"));
    }

    #[test]
    fn deactivate_cards_empty_returns_no_events() {
        let mut game = make_game();
        let events = UtilServerCards::deactivate_cards(&mut game, InducementDuration::UntilEndOfTurn, true);
        assert!(events.is_empty());
    }

    #[test]
    fn deactivate_while_holding_ball_deactivates_non_carrier() {
        let mut game = make_game();
        // Player p1 is on the field without the ball; card assigned to them
        let card = make_card("Illegal Substitution", InducementDuration::WhileHoldingTheBall);
        game.field_model.add_card("p1", card.clone());
        game.turn_data_home.inducement_set.add_available_card("Illegal Substitution");
        game.turn_data_home.inducement_set.activate_card_full(card);
        // No ball in play → p1 doesn't have ball
        let events = UtilServerCards::deactivate_while_holding_ball(&mut game);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn deactivate_while_holding_ball_keeps_carrier_card() {
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        // Set ball at coord and player p1 at same coord (p1 "has" the ball)
        let coord = FieldCoordinate::new(10, 5);
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_in_play = true;
        let card = make_card("Illegal Substitution", InducementDuration::WhileHoldingTheBall);
        game.field_model.add_card("p1", card.clone());
        game.turn_data_home.inducement_set.add_available_card("Illegal Substitution");
        game.turn_data_home.inducement_set.activate_card_full(card);
        let events = UtilServerCards::deactivate_while_holding_ball(&mut game);
        // p1 has ball, so card should NOT be deactivated
        assert_eq!(events.len(), 0);
    }
}
