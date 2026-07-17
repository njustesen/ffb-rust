use ffb_model::model::{Game, Player};
use ffb_model::util::util_cards::UtilCards;
use crate::modifiers::go_for_it_context::GoForItContext;
use crate::modifiers::go_for_it_modifier::GoForItModifier;
use crate::modifiers::interception_modifier::InterceptionModifier;
use crate::modifiers::modifier_type::ModifierType;
use crate::modifiers::pass_context::PassContext;
use crate::modifiers::pass_modifier::PassModifier;

// The card half of `GenerifiedModifierFactory.findModifiers()`: iterates
// `UtilCards.findAllCards(game)` and includes each card's `rollModifiers()` of the
// matching type whose `appliesToContext` predicate holds.
//
// Java dispatches by anonymous subclass; Rust cards are plain data (tracked by name in
// `InducementSet`/`FieldModel`), so dispatch is by card name instead — only the 4 BB2016
// cards whose Java class overrides `rollModifiers()` produce anything here (confirmed by
// reading every card in `bb2016/Cards.java`; BB2020/BB2025 have no card catalogs).

/// Java: `InterceptionModifierFactory` cards — Fawndough's Headband (thrower holds it),
/// Magic Gloves of Jark Longarm (interceptor holds it). Both `InterceptionModifier(-1, REGULAR)`.
pub fn find_interception_card_modifiers(game: &Game, interceptor: &Player) -> Vec<InterceptionModifier> {
    let mut result = Vec::new();
    for card in UtilCards::find_all_cards(game) {
        match card.name.as_str() {
            "Fawndough's Headband" => {
                if let Some(thrower) = game.thrower() {
                    if UtilCards::has_card(game, &thrower.id, &card.name) {
                        result.push(InterceptionModifier::new(card.name.clone(), -1, ModifierType::REGULAR));
                    }
                }
            }
            "Magic Gloves of Jark Longarm" if UtilCards::has_card(game, &interceptor.id, &card.name) => {
                result.push(InterceptionModifier::new(card.name.clone(), -1, ModifierType::REGULAR));
            }
            _ => {}
        }
    }
    result
}

/// Java: `PassModifierFactory` cards — Gromskull's Exploding Runes, `PassModifier(1, REGULAR)`,
/// applies when the passer holds the card. Java's predicate checks
/// `context.getPlayer().getEnhancementSources().contains(cardName)`; the Rust `Player` model has
/// no `enhancement_sources` field, so this uses the same `UtilCards::has_card` player-held check
/// as the other two cards, which is equivalent for this OWN_PLAYER-targeted, non-transferable card.
pub fn find_pass_card_modifiers(context: &PassContext<'_>) -> Vec<PassModifier> {
    let game = context.game;
    let mut result = Vec::new();
    for card in UtilCards::find_all_cards(game) {
        if card.name == "Gromskull's Exploding Runes"
            && UtilCards::has_card(game, &context.player.id, &card.name)
        {
            result.push(PassModifier::new(card.name.clone(), 1, ModifierType::REGULAR));
        }
    }
    result
}

/// Java: `GoForItModifierFactory` cards — Greased Shoes, `GoForItModifier(3)`, applies when the
/// side NOT currently on turn holds the card active (Java: `context.getGame().isHomePlaying() ?
/// turnDataAway : turnDataHome`). Not player-scoped: a `CardTarget::TURN` card.
pub fn find_go_for_it_card_modifiers(context: &GoForItContext<'_>) -> Vec<GoForItModifier> {
    let game = context.game;
    let mut result = Vec::new();
    for card in UtilCards::find_all_cards(game) {
        if card.name == "Greased Shoes" {
            let opposing_turn_data = if game.home_playing { &game.turn_data_away } else { &game.turn_data_home };
            if opposing_turn_data.inducement_set.is_active(&card.name) {
                result.push(GoForItModifier::new(card.name.clone(), 3));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::Team;
    use ffb_model::types::FieldCoordinate;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2016)
    }

    fn minimal_player(id: &str) -> Player {
        Player { id: id.into(), ..Default::default() }
    }

    #[test]
    fn fawndough_headband_applies_when_thrower_holds_card() {
        let mut game = make_game();
        let thrower = minimal_player("thrower");
        game.team_home.players.push(thrower.clone());
        game.thrower_id = Some("thrower".into());
        game.field_model.add_card("thrower", ffb_model::inducement::card::Card::new("Fawndough's Headband", None::<&str>));
        game.turn_data_home.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Fawndough's Headband", None::<&str>));

        let interceptor = minimal_player("interceptor");
        let mods = find_interception_card_modifiers(&game, &interceptor);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_modifier(), -1);
        assert_eq!(mods[0].get_name(), "Fawndough's Headband");
    }

    #[test]
    fn fawndough_headband_absent_when_thrower_lacks_card() {
        let mut game = make_game();
        let thrower = minimal_player("thrower");
        game.team_home.players.push(thrower.clone());
        game.thrower_id = Some("thrower".into());
        game.turn_data_home.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Fawndough's Headband", None::<&str>));

        let interceptor = minimal_player("interceptor");
        let mods = find_interception_card_modifiers(&game, &interceptor);
        assert!(mods.is_empty());
    }

    #[test]
    fn magic_gloves_applies_when_interceptor_holds_card() {
        let mut game = make_game();
        let interceptor = minimal_player("interceptor");
        game.field_model.add_card("interceptor", ffb_model::inducement::card::Card::new("Magic Gloves of Jark Longarm", None::<&str>));
        game.turn_data_home.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Magic Gloves of Jark Longarm", None::<&str>));

        let mods = find_interception_card_modifiers(&game, &interceptor);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_modifier(), -1);
    }

    #[test]
    fn no_interception_card_modifiers_when_no_cards_active() {
        let game = make_game();
        let interceptor = minimal_player("interceptor");
        let mods = find_interception_card_modifiers(&game, &interceptor);
        assert!(mods.is_empty());
    }

    #[test]
    fn gromskull_applies_when_passer_holds_card() {
        use ffb_model::enums::PassingDistance;
        let mut game = make_game();
        let passer = minimal_player("passer");
        game.field_model.add_card("passer", ffb_model::inducement::card::Card::new("Gromskull's Exploding Runes", None::<&str>));
        game.turn_data_home.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Gromskull's Exploding Runes", None::<&str>));
        game.team_home.players.push(passer.clone());
        game.field_model.set_player_coordinate("passer", FieldCoordinate::new(5, 5));

        let passer_ref = game.player("passer").unwrap();
        let ctx = PassContext::new(&game, passer_ref, PassingDistance::ShortPass, false);
        let mods = find_pass_card_modifiers(&ctx);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_modifier(), 1);
        assert_eq!(mods[0].get_name(), "Gromskull's Exploding Runes");
    }

    #[test]
    fn gromskull_absent_when_passer_lacks_card() {
        use ffb_model::enums::PassingDistance;
        let mut game = make_game();
        let passer = minimal_player("passer");
        game.turn_data_home.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Gromskull's Exploding Runes", None::<&str>));
        let ctx = PassContext::new(&game, &passer, PassingDistance::ShortPass, false);
        let mods = find_pass_card_modifiers(&ctx);
        assert!(mods.is_empty());
    }

    #[test]
    fn greased_shoes_applies_when_opposing_side_holds_it_home_playing() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_away.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Greased Shoes", None::<&str>));

        let player = minimal_player("p1");
        let ctx = GoForItContext::new(&game, &player);
        let mods = find_go_for_it_card_modifiers(&ctx);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_modifier(), 3);
    }

    #[test]
    fn greased_shoes_absent_when_own_side_holds_it() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Greased Shoes", None::<&str>));

        let player = minimal_player("p1");
        let ctx = GoForItContext::new(&game, &player);
        let mods = find_go_for_it_card_modifiers(&ctx);
        assert!(mods.is_empty());
    }

    #[test]
    fn greased_shoes_applies_when_opposing_side_holds_it_away_playing() {
        let mut game = make_game();
        game.home_playing = false;
        game.turn_data_home.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Greased Shoes", None::<&str>));

        let player = minimal_player("p1");
        let ctx = GoForItContext::new(&game, &player);
        let mods = find_go_for_it_card_modifiers(&ctx);
        assert_eq!(mods.len(), 1);
    }
}
