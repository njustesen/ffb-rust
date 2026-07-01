/// 1:1 translation of com.fumbbl.ffb.util.UtilCards (selected methods).
///
/// Java uses a rich Skill object with getRerollSource() etc.; Rust uses SkillId + properties().
/// Only the methods needed for BB2025 step translations are implemented here.
use crate::enums::SkillId;
use crate::model::player::Player;

pub struct UtilCards;

impl UtilCards {
    pub fn new() -> Self { Self }

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
}
