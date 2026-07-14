/// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerCards.
///
/// Static utility methods for playing and deactivating inducement cards.
use ffb_model::enums::InducementDuration;
use ffb_model::events::GameEvent;
use ffb_model::inducement::card::Card;
use ffb_model::model::game::Game;
use ffb_model::report::report_play_card::ReportPlayCard;
use ffb_model::util::rng::GameRng;
use crate::step::framework::StepParameter;

pub struct UtilServerCards;

impl UtilServerCards {
    pub fn new() -> Self { Self }

    /// Java: `UtilServerCards.findAllowedPlayersForCard(Game, Card)`.
    pub fn find_allowed_players_for_card(game: &Game, card: &Card) -> Vec<String> {
        use ffb_model::inducement::card_target::CardTarget;
        use crate::factory::card_handler_factory::CardHandlerFactory;

        if !card.get_target().is_played_on_player() {
            return vec![];
        }

        let own_is_home = game.turn_data_home.inducement_set.is_available(&card.name);
        let (own_team, other_team) = if own_is_home {
            (&game.team_home, &game.team_away)
        } else {
            (&game.team_away, &game.team_home)
        };

        let mut factory = CardHandlerFactory::new();
        factory.initialize(game.rules);
        let handler = factory.for_card(card);

        game.team_home.players.iter().chain(game.team_away.players.iter())
            .filter(|player| {
                let Some(state) = game.field_model.player_state(&player.id) else { return false };
                let mut allowed = !state.is_casualty()
                    && state.base() != ffb_model::enums::PS_BANNED
                    && state.base() != ffb_model::enums::PS_MISSING;
                if card.get_target() == CardTarget::OWN_PLAYER {
                    allowed &= own_team.has_player(&player.id);
                }
                if card.get_target() == CardTarget::OPPOSING_PLAYER {
                    allowed &= other_team.has_player(&player.id);
                }
                if let Some(handler) = handler {
                    allowed &= handler.allows_player(game, card, &player.id);
                }
                allowed
            })
            .map(|player| player.id.clone())
            .collect()
    }

    /// Java: `UtilServerCards.activateCard(IStep, Card, boolean homeTeam, String playerId)`.
    ///
    /// Client-only concerns skipped (headless engine, no rendering/network layer here):
    /// `pStep.getResult().setAnimation(...)` and `UtilServerGame.syncGameModel(pStep)`.
    ///
    /// Returns `(do_next_step, params)`: `do_next_step` mirrors the card handler's `activate()`
    /// boolean return (only `IllegalSubstitutionHandler` returns `false`, to hold the step open
    /// for a follow-up dialog); `params` are collected from the handler's `activation_parameters`
    /// (e.g. `PitTrapHandler`'s `dropPlayer`-derived `StepParameter`s, which Java pushes via
    /// `step.publishParameters(...)` inside `activate()` itself).
    pub fn activate_card(
        game: &mut Game,
        rng: &mut GameRng,
        card: &Card,
        home_team: bool,
        player_id: Option<&str>,
    ) -> (bool, Vec<StepParameter>) {
        use crate::factory::card_handler_factory::CardHandlerFactory;

        let team_id = if home_team { game.team_home.id.clone() } else { game.team_away.id.clone() };
        let has_player_id = player_id.map(|p| !p.is_empty()).unwrap_or(false);
        game.report_list.add(if has_player_id {
            ReportPlayCard::new_with_player(team_id, card.name.clone(), player_id.map(str::to_string))
        } else {
            ReportPlayCard::new(team_id, card.name.clone())
        });

        // Java: `inducementSet.activateCard(pCard)` — moves the full Card (preserving
        // handler_key/target/duration) from available to active.
        if home_team {
            game.turn_data_home.inducement_set.activate_card_full(card.clone());
        } else {
            game.turn_data_away.inducement_set.activate_card_full(card.clone());
        }

        if has_player_id {
            if let Some(pid) = player_id {
                game.field_model.add_card(pid, card.clone());
            }
        }

        let mut factory = CardHandlerFactory::new();
        factory.initialize(game.rules);
        let Some(handler) = factory.for_card(card) else { return (true, vec![]) };

        let pid = player_id.unwrap_or("");
        let do_next_step = handler.activate_on_game(game, card, pid, rng);
        let params = handler.activation_parameters(game, card, pid, rng);
        (do_next_step, params)
    }

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

    fn add_player(game: &mut Game, home: bool, id: &str, coord: ffb_model::types::FieldCoordinate) {
        use ffb_model::enums::{PlayerGender, PlayerState, PlayerType, PS_STANDING};
        use ffb_model::model::player::Player;
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if home { game.team_home.players.push(player); } else { game.team_away.players.push(player); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn activate_card_reports_and_activates() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let card = Card::new("Blackmail", None::<&str>);
        let mut rng = GameRng::new(0);
        let (do_next_step, params) = UtilServerCards::activate_card(&mut game, &mut rng, &card, true, None);
        assert!(do_next_step);
        assert!(params.is_empty());
        assert!(game.report_list.has_report(ReportId::PLAY_CARD));
        assert!(game.turn_data_home.inducement_set.is_active("Blackmail"));
    }

    #[test]
    fn activate_card_with_pit_trap_handler_drops_the_player() {
        use ffb_model::enums::PS_PRONE;
        use ffb_model::types::FieldCoordinate;
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        add_player(&mut game, true, "p1", FieldCoordinate::new(5, 5));
        let card = Card::new("Pit Trap", Some("PIT_TRAP"));
        let mut rng = GameRng::new(0);
        UtilServerCards::activate_card(&mut game, &mut rng, &card, true, Some("p1"));
        assert_eq!(game.field_model.player_state("p1").unwrap().base(), PS_PRONE);
        assert!(game.field_model.find_player_with_card("Pit Trap").is_some());
    }

    #[test]
    fn find_allowed_players_for_card_excludes_non_player_targeted_cards() {
        let game = make_game();
        let card = Card::new("Blackmail", None::<&str>); // default target: TURN
        assert!(UtilServerCards::find_allowed_players_for_card(&game, &card).is_empty());
    }

    #[test]
    fn find_allowed_players_for_card_filters_by_own_player_target() {
        use ffb_model::inducement::card_target::CardTarget;
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        add_player(&mut game, true, "home1", FieldCoordinate::new(5, 5));
        add_player(&mut game, false, "away1", FieldCoordinate::new(6, 5));
        game.turn_data_home.inducement_set.add_available_card("Pit Trap");
        let card = Card::new("Pit Trap", Some("PIT_TRAP")).with_target(CardTarget::OWN_PLAYER);
        let allowed = UtilServerCards::find_allowed_players_for_card(&game, &card);
        assert_eq!(allowed, vec!["home1".to_string()]);
    }

    #[test]
    fn find_allowed_players_for_card_filters_by_opposing_player_target() {
        use ffb_model::inducement::card_target::CardTarget;
        use ffb_model::types::FieldCoordinate;
        let mut game = make_game();
        add_player(&mut game, true, "home1", FieldCoordinate::new(5, 5));
        add_player(&mut game, false, "away1", FieldCoordinate::new(6, 5));
        game.turn_data_home.inducement_set.add_available_card("Opponent Card");
        // No registered handler for this key -> the handler-gate in
        // find_allowed_players_for_card is skipped, isolating the OPPOSING_PLAYER filter itself.
        let card = Card::new("Opponent Card", Some("UNKNOWN_KEY")).with_target(CardTarget::OPPOSING_PLAYER);
        let allowed = UtilServerCards::find_allowed_players_for_card(&game, &card);
        assert_eq!(allowed, vec!["away1".to_string()]);
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
