use ffb_model::enums::Rules;
use ffb_model::util::util_player::UtilPlayer;
use crate::modifiers::bb2025::right_stuff_modifier_collection::RightStuffModifierCollection as Bb2025Collection;
use crate::modifiers::bb2016::right_stuff_modifier_collection::RightStuffModifierCollection as Bb2016Collection;
use crate::modifiers::bb2020::right_stuff_modifier_collection::RightStuffModifierCollection as Bb2020Collection;
use crate::modifiers::right_stuff_context::RightStuffContext;
use crate::modifiers::right_stuff_modifier::RightStuffModifier;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.factory.RightStuffModifierFactory.
///
/// Java: isAffectedByTackleZones=true, isAffectedByDisturbingPresence=false.
pub struct RightStuffModifierFactory {
    collection: Box<dyn RightStuffCollection>,
}

trait RightStuffCollection: Send + Sync {
    fn find_applicable<'a>(&'a self, ctx: &RightStuffContext<'_>) -> Vec<&'a RightStuffModifier>;
    fn get_modifiers(&self) -> &[RightStuffModifier];
}

impl RightStuffCollection for Bb2025Collection {
    fn find_applicable<'a>(&'a self, ctx: &RightStuffContext<'_>) -> Vec<&'a RightStuffModifier> { self.find_applicable(ctx) }
    fn get_modifiers(&self) -> &[RightStuffModifier] { self.get_modifiers() }
}
impl RightStuffCollection for Bb2020Collection {
    fn find_applicable<'a>(&'a self, ctx: &RightStuffContext<'_>) -> Vec<&'a RightStuffModifier> { self.find_applicable(ctx) }
    fn get_modifiers(&self) -> &[RightStuffModifier] { self.get_modifiers() }
}
impl RightStuffCollection for Bb2016Collection {
    fn find_applicable<'a>(&'a self, ctx: &RightStuffContext<'_>) -> Vec<&'a RightStuffModifier> { self.find_applicable(ctx) }
    fn get_modifiers(&self) -> &[RightStuffModifier] { self.get_modifiers() }
}

impl RightStuffModifierFactory {
    pub fn for_rules(rules: Rules) -> Self {
        let collection: Box<dyn RightStuffCollection> = match rules {
            Rules::Bb2025 | Rules::Common => Box::new(Bb2025Collection::new()),
            Rules::Bb2020 => Box::new(Bb2020Collection::new()),
            _ => Box::new(Bb2016Collection::new()),
        };
        Self { collection }
    }

    pub fn for_name(&self, name: &str) -> Option<&RightStuffModifier> {
        self.collection.get_modifiers().iter().find(|m| m.get_name() == name)
    }

    /// Java: GenerifiedModifierFactory.findModifiers.
    /// REGULAR modifiers filtered by predicate; TACKLEZONE modifier selected by tackle-zone count.
    /// isAffectedByTackleZones=true for RightStuffModifierFactory.
    pub fn find_applicable<'a>(&'a self, ctx: &RightStuffContext<'_>) -> Vec<&'a RightStuffModifier> {
        let mut result: Vec<&'a RightStuffModifier> = self.collection.get_modifiers().iter()
            .filter(|m| m.get_type() == ModifierType::REGULAR && m.applies_to_context(ctx))
            .collect();

        let count = UtilPlayer::find_tacklezones(ctx.game, &ctx.player.id) as i32;
        if count > 0 {
            if let Some(tz_mod) = self.collection.get_modifiers().iter()
                .find(|m| m.get_type() == ModifierType::TACKLEZONE && m.get_modifier() == count)
            {
                result.push(tz_mod);
            }
        }

        result
    }

    /// Java: AgilityMechanic.minimumRollRightStuff: agility + sum(modifiers), min 2.
    pub fn minimum_roll(agility: i32, modifiers: &[&RightStuffModifier]) -> i32 {
        let total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        (agility + total).max(2)
    }
}

impl Default for RightStuffModifierFactory {
    fn default() -> Self {
        Self::for_rules(Rules::Bb2025)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    #[test]
    fn minimum_roll_base_is_agility() {
        assert_eq!(RightStuffModifierFactory::minimum_roll(3, &[]), 3);
    }

    #[test]
    fn minimum_roll_never_below_two() {
        assert_eq!(RightStuffModifierFactory::minimum_roll(1, &[]), 2);
    }

    #[test]
    fn minimum_roll_adds_modifier_total() {
        let m = RightStuffModifier::new("Subpar Throw", 1, ModifierType::REGULAR);
        assert_eq!(RightStuffModifierFactory::minimum_roll(3, &[&m]), 4);
    }

    #[test]
    fn for_rules_bb2025_has_pass_result_modifiers() {
        let factory = RightStuffModifierFactory::for_rules(Rules::Bb2025);
        assert!(factory.for_name("Subpar Throw").is_some());
        assert!(factory.for_name("Fumbled Throw").is_some());
    }

    #[test]
    fn for_rules_bb2016_has_kick_range_modifiers() {
        let factory = RightStuffModifierFactory::for_rules(Rules::Bb2016);
        assert!(factory.for_name("Medium Kick").is_some());
        assert!(factory.for_name("Long Kick").is_some());
    }
}
