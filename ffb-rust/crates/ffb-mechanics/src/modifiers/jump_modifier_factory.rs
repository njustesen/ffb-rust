use ffb_model::enums::Rules;
use ffb_model::model::property::NamedProperties;
use ffb_model::util::util_player::UtilPlayer;
use crate::modifiers::mixed::jump_modifier_collection::JumpModifierCollection as MixedCollection;
use crate::modifiers::bb2016::jump_modifier_collection::JumpModifierCollection as Bb2016Collection;
use crate::modifiers::jump_context::JumpContext;
use crate::modifiers::jump_modifier::JumpModifier;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.factory.mixed.JumpModifierFactory (BB2020/BB2025).
///
/// Finds modifiers for an agility (Jump/Leap) roll:
/// - TACKLEZONE: max(fromZones, toZones) tackle zones from adjacent opponents.
/// - PREHENSILE_TAIL: opponents adjacent to `from` with makesJumpingHarder property.
/// - BB2016: no modifiers (empty collection).
pub struct JumpModifierFactory {
    collection: Box<dyn JumpCollection>,
}

trait JumpCollection: Send + Sync {
    fn get_modifiers(&self) -> &[JumpModifier];
}

impl JumpCollection for MixedCollection {
    fn get_modifiers(&self) -> &[JumpModifier] { self.get_modifiers() }
}

impl JumpCollection for Bb2016Collection {
    fn get_modifiers(&self) -> &[JumpModifier] { self.get_modifiers() }
}

impl JumpModifierFactory {
    pub fn for_rules(rules: Rules) -> Self {
        let collection: Box<dyn JumpCollection> = match rules {
            Rules::Bb2025 | Rules::Bb2020 | Rules::Common => Box::new(MixedCollection::new()),
            _ => Box::new(Bb2016Collection::new()),
        };
        Self { collection }
    }

    /// Java: JumpModifierFactory.findModifiers(context).
    /// Returns TACKLEZONE and PREHENSILE_TAIL modifiers based on opponent adjacency at from/to.
    pub fn find_applicable<'a>(&'a self, ctx: &JumpContext<'_>) -> Vec<&'a JumpModifier> {
        let mut result: Vec<&'a JumpModifier> = Vec::new();

        // REGULAR skill modifiers (none in current collection — skill infra not ported).
        // The mixed collection only has TZ and PT modifiers.

        // Determine opposing team.
        let other_team = if ctx.game.team_home.has_player(&ctx.player.id) {
            &ctx.game.team_away
        } else {
            &ctx.game.team_home
        };

        // Count tackle zones at `from` and `to` — take max.
        let from_ids = UtilPlayer::find_adjacent_players_with_tacklezones(
            ctx.game, other_team, ctx.from, false,
        );
        let to_ids = UtilPlayer::find_adjacent_players_with_tacklezones(
            ctx.game, other_team, ctx.to, false,
        );
        let tz_count = (from_ids.len() as i32).max(to_ids.len() as i32);

        if tz_count > 0 {
            if let Some(m) = self.collection.get_modifiers().iter()
                .find(|m| m.get_type() == ModifierType::TACKLEZONE && m.get_multiplier() == tz_count)
            {
                result.push(m);
            }
        }

        // Count prehensile tails: adjacent opponents at `from` with makesJumpingHarder.
        let pt_count = from_ids.iter()
            .filter(|id| ctx.game.player(id.as_str())
                .map(|p| p.has_skill_property(NamedProperties::MAKES_JUMPING_HARDER))
                .unwrap_or(false))
            .count() as i32;

        if pt_count > 0 {
            if let Some(m) = self.collection.get_modifiers().iter()
                .find(|m| m.get_type() == ModifierType::PREHENSILE_TAIL && m.get_multiplier() == pt_count)
            {
                result.push(m);
            }
        }

        result
    }

    /// Java: AgilityMechanic.minimumRollJump: agility + sum(modifiers), min 2.
    pub fn minimum_roll(agility: i32, modifiers: &[&JumpModifier]) -> i32 {
        let total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        (agility + total).max(2)
    }
}

impl Default for JumpModifierFactory {
    fn default() -> Self { Self::for_rules(Rules::Bb2025) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    #[test]
    fn for_rules_bb2025_has_tacklezone_modifiers() {
        let f = JumpModifierFactory::for_rules(Rules::Bb2025);
        assert!(f.collection.get_modifiers().iter().any(|m| m.get_type() == ModifierType::TACKLEZONE));
    }

    #[test]
    fn for_rules_bb2016_has_no_modifiers() {
        let f = JumpModifierFactory::for_rules(Rules::Bb2016);
        assert_eq!(f.collection.get_modifiers().len(), 0);
    }

    #[test]
    fn minimum_roll_base_is_agility() {
        assert_eq!(JumpModifierFactory::minimum_roll(3, &[]), 3);
    }

    #[test]
    fn minimum_roll_never_below_two() {
        assert_eq!(JumpModifierFactory::minimum_roll(1, &[]), 2);
    }

    #[test]
    fn minimum_roll_adds_modifier_total() {
        let m = JumpModifier::new("1 Tacklezone", 1, ModifierType::TACKLEZONE);
        assert_eq!(JumpModifierFactory::minimum_roll(3, &[&m]), 4);
    }
}
