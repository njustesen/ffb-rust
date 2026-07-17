/// 1:1 translation of com.fumbbl.ffb.util.UtilCards (selected methods).
///
/// Java uses a rich Skill object with getRerollSource() etc.; Rust uses SkillId + properties().
/// Only the methods needed for BB2025 step translations are implemented here.
use crate::enums::SkillId;
use crate::inducement::card::Card;
use crate::model::game::Game;
use crate::model::player::Player;

pub struct UtilCards;

impl UtilCards {
    pub fn new() -> Self { Self }

    /// Java: `UtilCards.findAllCards(Game)` — every card active or deactivated on
    /// either team's `InducementSet`. Deactivated cards are omitted here: unlike Java's
    /// `InducementSet`, the Rust port only retains deactivated cards by name (no `Card`
    /// object), a pre-existing model gap outside this method's scope.
    pub fn find_all_cards(game: &Game) -> Vec<&Card> {
        game.turn_data_home.inducement_set.get_active_card_objects()
            .into_iter()
            .chain(game.turn_data_away.inducement_set.get_active_card_objects())
            .collect()
    }

    /// Java: `UtilCards.hasCard(Game, Player, Card)` — true if `pCard` is assigned to
    /// `pPlayer` via `FieldModel.getCards(Player)`. Compares by card name (Rust cards
    /// are tracked by name, not object identity).
    pub fn has_card(game: &Game, player_id: &str, card_name: &str) -> bool {
        game.field_model.get_cards(player_id).iter().any(|c| c.name == card_name)
    }

    /// Java: `hasUnusedSkillWithProperty(Player<?> player, ISkillProperty property)`.
    ///
    /// Returns true if the player has at least one skill with the given named property
    /// that has not yet been marked as used this turn.
    pub fn has_unused_skill_with_property(player: &Player, property: &str) -> bool {
        player.all_skill_ids()
            .any(|id| id.properties().contains(&property) && !player.used_skills.contains(&id))
    }

    /// Java: `hasSkillWithProperty(Player<?> player, ISkillProperty property)` — ignores used state.
    pub fn has_skill_with_property(player: &Player, property: &str) -> bool {
        player.has_skill_property(property)
    }

    /// Java: `getUnusedSkillWithProperty(ActingPlayer<?> actingPlayer, ISkillProperty property)`.
    ///
    /// Returns the first unused SkillId with the given property, or None.
    pub fn get_unused_skill_with_property(player: &Player, property: &str) -> Option<SkillId> {
        player.all_skill_ids()
            .find(|id| id.properties().contains(&property) && !player.used_skills.contains(id))
    }

    /// Java: `hasSkillToCancelProperty(Player<?> player, ISkillProperty property)`.
    ///
    /// Returns true if the player has any skill with a CancelSkillProperty that cancels
    /// the given named property. In the Rust skill system, CancelSkillProperty(X) is
    /// registered as the string `"cancels" + X` (e.g. `"cancelsDodge"` cancels `"canDodge"`).
    pub fn has_skill_to_cancel_property(player: &Player, property: &str) -> bool {
        // Build the cancel-property name: "cancels" + property with first char uppercased.
        let cancel_prop = if property.is_empty() {
            "cancels".to_string()
        } else {
            let mut chars = property.chars();
            let first = chars.next().unwrap().to_uppercase().to_string();
            format!("cancels{}{}", first, chars.as_str())
        };
        player.has_skill_property(&cancel_prop)
    }

    /// Java: `getSkillCancelling(Player<?> player, Skill skill)` (by property).
    ///
    /// Returns the first SkillId that has a CancelSkillProperty for the given named property,
    /// matching `hasSkillToCancelProperty` but returning the ID for use in reports.
    pub fn get_skill_cancelling_property(player: &Player, property: &str) -> Option<SkillId> {
        let cancel_prop = if property.is_empty() {
            "cancels".to_string()
        } else {
            let mut chars = property.chars();
            let first = chars.next().unwrap().to_uppercase().to_string();
            format!("cancels{}{}", first, chars.as_str())
        };
        player.all_skill_ids().find(|id| id.properties().contains(&cancel_prop.as_str()))
    }
}

impl Default for UtilCards {
    fn default() -> Self { Self::new() }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{PlayerGender, PlayerType, SkillId};
    use crate::model::skill_def::SkillWithValue;
    use std::collections::HashSet;

    fn player_with_skill(skill_id: SkillId) -> Player {
        Player {
            id: "p1".into(), name: "Test".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    #[test]
    fn has_unused_skill_with_property_true_when_skill_present_and_unused() {
        let p = player_with_skill(SkillId::Dodge);
        // Dodge has property "canRerollOnFail"/"canRerollDodge" — check any known property
        // Use a property we know Dodge has (from skill_id properties table)
        let props = SkillId::Dodge.properties();
        if let Some(prop) = props.first() {
            assert!(UtilCards::has_unused_skill_with_property(&p, prop));
        }
    }

    #[test]
    fn has_unused_skill_with_property_false_when_skill_used() {
        let mut p = player_with_skill(SkillId::Dodge);
        p.used_skills.insert(SkillId::Dodge);
        let props = SkillId::Dodge.properties();
        if let Some(prop) = props.first() {
            assert!(!UtilCards::has_unused_skill_with_property(&p, prop));
        }
    }

    #[test]
    fn has_unused_skill_with_property_false_when_no_skill() {
        let p = player_with_skill(SkillId::Block);
        // Dodge property is not on a Block player
        let dodge_props = SkillId::Dodge.properties();
        if let Some(prop) = dodge_props.first() {
            if !SkillId::Block.properties().contains(prop) {
                assert!(!UtilCards::has_unused_skill_with_property(&p, prop));
            }
        }
    }

    #[test]
    fn get_unused_skill_with_property_returns_skill_id_when_present_and_none_when_absent() {
        // Wrestle has "canTakeDownPlayersWithHimOnBothDown"
        let property = "canTakeDownPlayersWithHimOnBothDown";
        let p = player_with_skill(SkillId::Wrestle);
        assert_eq!(
            UtilCards::get_unused_skill_with_property(&p, property),
            Some(SkillId::Wrestle),
        );

        // A player without Wrestle should return None for that property
        let p2 = player_with_skill(SkillId::Block);
        assert_eq!(
            UtilCards::get_unused_skill_with_property(&p2, property),
            None,
        );
    }

    #[test]
    fn has_skill_to_cancel_property_true_for_tackle_cancelling_dodge_reroll() {
        // Tackle has "cancelsCanRerollDodge" which cancels the property "canRerollDodge".
        // has_skill_to_cancel_property("canRerollDodge") should build "cancelsCanRerollDodge" and find Tackle.
        let p = player_with_skill(SkillId::Tackle);
        assert!(UtilCards::has_skill_to_cancel_property(&p, "canRerollDodge"));

        // A player without Tackle should not cancel that property
        let p2 = player_with_skill(SkillId::Block);
        assert!(!UtilCards::has_skill_to_cancel_property(&p2, "canRerollDodge"));
    }
}
