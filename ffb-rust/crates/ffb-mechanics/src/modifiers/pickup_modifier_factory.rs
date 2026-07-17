use ffb_model::enums::{Rules, SkillId};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_player::UtilPlayer;
use crate::modifiers::modifier_type::ModifierType;
use crate::modifiers::pickup_context::PickupContext;
use crate::modifiers::pickup_modifier::PickupModifier;
use crate::modifiers::pickup_modifier_collection::PickupModifierCollection;

/// 1:1 translation of com.fumbbl.ffb.factory.PickupModifierFactory.
///
/// Edition-agnostic: BB2025 and all editions use the same base PickupModifierCollection.
/// Modifier lookup and minimum-roll calculation mirror the Java GenerifiedModifierFactory.
pub struct PickupModifierFactory {
    collection: PickupModifierCollection,
}

impl PickupModifierFactory {
    /// Construct a factory for the given rules edition.
    pub fn for_rules(_rules: Rules) -> Self {
        Self { collection: PickupModifierCollection::new() }
    }

    /// 1:1 translation of PickupModifierFactory.forName.
    pub fn for_name(&self, name: &str) -> Option<&PickupModifier> {
        self.collection.get_modifiers().iter().find(|m| m.get_name() == name)
    }

    /// Returns the modifiers applicable to the given context.
    /// 1:1 translation of GenerifiedModifierFactory.findModifiers.
    ///
    /// REGULAR modifiers are filtered by their predicates.
    /// TACKLEZONE: one modifier is selected by counting actual tackle zones; applied unless
    ///   the player has `ignoreTacklezonesWhenPickingUp`.
    pub fn find_applicable<'a>(&'a self, context: &PickupContext<'_>) -> Vec<&'a PickupModifier> {
        let mut result: Vec<&'a PickupModifier> = self.collection.get_modifiers().iter()
            .filter(|m| m.get_type() == ModifierType::REGULAR && m.applies_to_context(context))
            .collect();

        // Java: isAffectedByTackleZones → !player.hasSkillProperty(ignoreTacklezonesWhenPickingUp)
        let affected_by_tackle_zones = !context.player.has_skill_property(NamedProperties::IGNORE_TACKLEZONES_WHEN_PICKING_UP);
        if affected_by_tackle_zones {
            let count = UtilPlayer::find_tacklezones(context.game, &context.player.id) as i32;
            if count > 0 {
                // Find the TACKLEZONE modifier matching this count (multiplier == count)
                if let Some(tz_mod) = self.collection.get_modifiers().iter()
                    .find(|m| m.get_type() == ModifierType::TACKLEZONE && m.get_modifier() == count)
                {
                    result.push(tz_mod);
                }
            }
        }

        result
    }

    /// Returns skill-based pickup modifiers for the player.
    /// Java: common.ExtraArms registers PickupModifier("Extra Arms", -1, REGULAR).
    pub fn find_skill_modifiers(&self, context: &PickupContext<'_>) -> Vec<PickupModifier> {
        let player = context.player;
        let mut result = Vec::new();
        for skill_id in player.all_skill_ids() {
            if skill_id == SkillId::ExtraArms {
                result.push(PickupModifier::new("Extra Arms", -1, ModifierType::REGULAR));
            }
        }
        result
    }

    /// Java: `ModifierAggregator.getPickupModifiers()`'s skill half. Only `common.ExtraArms`
    /// registers a `PickupModifier` in the Java source, in every edition.
    pub fn find_registered_modifiers(_rules: Rules) -> Vec<PickupModifier> {
        let mut result = Vec::new();
        for skill_id in ffb_model::factory::skill_factory::SkillFactory::new().get_skills() {
            if skill_id == SkillId::ExtraArms {
                result.push(PickupModifier::new("Extra Arms", -1, ModifierType::REGULAR));
            }
        }
        result
    }

    /// 1:1 translation of AgilityMechanic.minimumRollPickup.
    /// `max(2, agility + sum(modifier))` for BB2025.
    pub fn minimum_roll(agility: i32, modifiers: &[&PickupModifier]) -> i32 {
        let total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        (agility + total).max(2)
    }
}

impl Default for PickupModifierFactory {
    fn default() -> Self {
        Self::for_rules(Rules::Bb2025)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, Weather};
    use ffb_model::model::{Game, Team, Player};
    use ffb_model::enums::{PlayerType, PlayerGender};
    use crate::modifiers::pickup_context::PickupContext;

    #[test]
    fn find_registered_modifiers_includes_extra_arms() {
        let mods = PickupModifierFactory::find_registered_modifiers(Rules::Bb2025);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "Extra Arms");
    }

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

    fn minimal_player() -> Player {
        Player {
            id: "p1".into(), name: "Joe".into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_game(weather: Weather) -> Game {
        let mut game = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025);
        game.field_model.weather = weather;
        game
    }

    #[test]
    fn nice_weather_no_modifiers() {
        let game = make_game(Weather::Nice);
        let player = minimal_player();
        let factory = PickupModifierFactory::for_rules(Rules::Bb2025);
        let ctx = PickupContext::new(&game, &player);
        let mods = factory.find_applicable(&ctx);
        assert!(mods.is_empty(), "Nice weather should yield no pickup modifiers");
    }

    #[test]
    fn pouring_rain_adds_one_modifier() {
        let game = make_game(Weather::PouringRain);
        let player = minimal_player();
        let factory = PickupModifierFactory::for_rules(Rules::Bb2025);
        let ctx = PickupContext::new(&game, &player);
        let mods = factory.find_applicable(&ctx);
        assert_eq!(mods.len(), 1, "Pouring Rain should add one pickup modifier");
        assert_eq!(mods[0].get_modifier(), 1);
    }

    #[test]
    fn minimum_roll_no_modifiers() {
        // agility 3, no modifiers → max(2, 3) = 3
        assert_eq!(PickupModifierFactory::minimum_roll(3, &[]), 3);
    }

    #[test]
    fn minimum_roll_with_rain_modifier() {
        // Pouring Rain adds +1 → max(2, agility=3 + 1) = 4
        let m = PickupModifier::new("Pouring Rain", 1, crate::modifiers::modifier_type::ModifierType::REGULAR);
        assert_eq!(PickupModifierFactory::minimum_roll(3, &[&m]), 4);
    }

    #[test]
    fn minimum_roll_never_below_two() {
        assert_eq!(PickupModifierFactory::minimum_roll(1, &[]), 2);
    }

    #[test]
    fn for_name_returns_rain_modifier() {
        let factory = PickupModifierFactory::for_rules(Rules::Bb2025);
        assert!(factory.for_name("Pouring Rain").is_some());
        assert!(factory.for_name("NonExistent").is_none());
    }

    #[test]
    fn find_skill_modifiers_extra_arms_applies() {
        use ffb_model::enums::SkillId;
        use ffb_model::model::SkillWithValue;
        let game = make_game(Weather::Nice);
        let mut player = minimal_player();
        player.starting_skills.push(SkillWithValue::new(SkillId::ExtraArms));
        let factory = PickupModifierFactory::for_rules(Rules::Bb2025);
        let ctx = PickupContext::new(&game, &player);
        let mods = factory.find_skill_modifiers(&ctx);
        assert!(mods.iter().any(|m| m.get_name() == "Extra Arms"));
        assert_eq!(mods.iter().find(|m| m.get_name() == "Extra Arms").unwrap().get_modifier(), -1);
    }

    #[test]
    fn find_skill_modifiers_no_extra_arms_returns_empty() {
        let game = make_game(Weather::Nice);
        let player = minimal_player();
        let factory = PickupModifierFactory::for_rules(Rules::Bb2025);
        let ctx = PickupContext::new(&game, &player);
        let mods = factory.find_skill_modifiers(&ctx);
        assert!(mods.is_empty());
    }
}
