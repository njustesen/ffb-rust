use ffb_model::enums::Rules;
use ffb_model::model::SpecialEffect;
use ffb_model::model::{Game, Player};
use crate::modifiers::armor_modifier::ArmorModifier;
use crate::modifiers::armor_modifier_context::ArmorModifierContext;
use crate::modifiers::armor_modifiers::ArmorModifiers;
use crate::modifiers::bb2016::armor_modifiers::Bb2016ArmorModifiers;
use crate::modifiers::bb2020::armor_modifiers::Bb2020ArmorModifiers;
use crate::modifiers::bb2025::armor_modifiers::Bb2025ArmorModifiers;
use crate::modifiers::modifier_aggregator::ModifierAggregator;

/// 1:1 translation of com.fumbbl.ffb.factory.ArmorModifierFactory.
pub struct ArmorModifierFactory {
    armor_modifiers: Box<dyn ArmorModifiers>,
    modifier_aggregator: ModifierAggregator,
}

impl ArmorModifierFactory {
    pub fn new(rules: Rules) -> Self {
        Self {
            armor_modifiers: make_armor_modifiers(rules),
            modifier_aggregator: ModifierAggregator::new(),
        }
    }

    /// Java: forName(String) → returns the first modifier matching name across all_values + aggregator.
    pub fn for_name(&self, name: &str) -> Option<Box<dyn ArmorModifier>> {
        let from_collection = self.armor_modifiers.all_values()
            .into_iter()
            .find(|m| m.get_name() == name);
        if from_collection.is_some() { return from_collection; }

        // DEFERRED: modifier_aggregator.get_armour_modifiers() always returns empty (SkillFactory not ported)
        self.modifier_aggregator.get_armour_modifiers()
            .into_iter()
            .find(|m| m.get_name() == name)
    }

    /// Java: findArmorModifiers — scans attacker skills for applicable modifiers.
    /// Skill-based modifiers DEFERRED (SkillFactory not ported); collection-only modifiers returned.
    pub fn find_armor_modifiers(
        &self,
        game: &Game,
        attacker: Option<&Player>,
        defender: &Player,
        is_stab: bool,
        is_foul: bool,
    ) -> Vec<Box<dyn ArmorModifier>> {
        // DEFERRED: UtilCards.findAllSkills(attacker).flatMap(skill.getArmorModifiers) check
        // requires SkillFactory — returns empty for now.
        let _ = (game, attacker, defender, is_stab, is_foul);
        vec![]
    }

    /// Java: specialEffectArmourModifiers(SpecialEffect, Player) — returns modifiers for given effect.
    /// Checks ignoresArmourModifiersFromSkills on defender; DEFERRED (skill property lookup).
    pub fn special_effect_armour_modifiers(
        &self,
        special_effect: SpecialEffect,
        _defender: &Player,
    ) -> Vec<Box<dyn ArmorModifier>> {
        // DEFERRED: ignoresArmourModifiersFromSkills check (SkillFactory not ported)
        self.armor_modifiers.values()
            .into_iter()
            .filter(|m| m.get_special_effect() == Some(special_effect))
            .collect()
    }

    /// Java: getFoulAssist(ArmorModifierContext) — returns foul-assist modifiers matching context.
    pub fn get_foul_assist(&self, context: &ArmorModifierContext<'_>) -> Vec<Box<dyn ArmorModifier>> {
        // DEFERRED: ignoresArmourModifiersFromSkills check on defender (SkillFactory not ported)
        self.armor_modifiers.values()
            .into_iter()
            .filter(|m| m.get_special_effect().is_none() && m.applies_to_context(context))
            .collect()
    }

    /// Java: toArray(Set<ArmorModifier>) — sorted by name.
    pub fn to_array(mut modifiers: Vec<Box<dyn ArmorModifier>>) -> Vec<Box<dyn ArmorModifier>> {
        modifiers.sort_by(|a, b| a.get_name().cmp(b.get_name()));
        modifiers
    }

    /// Initialize with use_all flag (controls legacy Bomb modifier inclusion).
    pub fn set_use_all(&mut self, use_all: bool) {
        self.armor_modifiers.set_use_all(use_all);
    }
}

fn make_armor_modifiers(rules: Rules) -> Box<dyn ArmorModifiers> {
    match rules {
        Rules::Bb2016 => Box::new(Bb2016ArmorModifiers),
        Rules::Bb2020 => Box::new(Bb2020ArmorModifiers::new()),
        Rules::Bb2025 | Rules::Common => Box::new(Bb2025ArmorModifiers),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::team::Team;
    use ffb_model::enums::{PlayerType, PlayerGender};

    fn dummy_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "human".into(), roster_id: "human".into(),
            coach: "c".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players,
        }
    }

    fn make_game(rules: Rules) -> Game {
        let home = make_team("home", vec![dummy_player("a"), dummy_player("d")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, rules)
    }

    #[test]
    fn for_name_finds_foul_assist() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        assert!(f.for_name("1 Offensive Assist").is_some());
    }

    #[test]
    fn for_name_returns_none_for_unknown() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        assert!(f.for_name("Unknown Modifier").is_none());
    }

    #[test]
    fn special_effect_returns_fireball_modifier() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let defender = dummy_player("d");
        let mods = f.special_effect_armour_modifiers(SpecialEffect::FIREBALL, &defender);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "Fireball");
    }

    #[test]
    fn special_effect_bomb_not_in_bb2025() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let defender = dummy_player("d");
        let mods = f.special_effect_armour_modifiers(SpecialEffect::BOMB, &defender);
        assert!(mods.is_empty());
    }

    #[test]
    fn get_foul_assist_returns_matching_modifier() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = dummy_player("a");
        let defender = dummy_player("d");
        let ctx = ArmorModifierContext::new_with_foul_assists(&game, Some(&attacker), &defender, false, true, 3);
        let mods = f.get_foul_assist(&ctx);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "3 Offensive Assists");
    }

    #[test]
    fn get_foul_assist_defensive_assist() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = dummy_player("a");
        let defender = dummy_player("d");
        let ctx = ArmorModifierContext::new_with_foul_assists(&game, Some(&attacker), &defender, false, true, -2);
        let mods = f.get_foul_assist(&ctx);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "2 Defensive Assists");
    }

    #[test]
    fn to_array_sorts_by_name() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = dummy_player("a");
        let defender = dummy_player("d");
        // Get a few modifiers
        let ctx1 = ArmorModifierContext::new_with_foul_assists(&game, Some(&attacker), &defender, false, true, 3);
        let ctx2 = ArmorModifierContext::new_with_foul_assists(&game, Some(&attacker), &defender, false, true, 1);
        let mut mods = vec![];
        mods.extend(f.get_foul_assist(&ctx1));
        mods.extend(f.get_foul_assist(&ctx2));
        let sorted = ArmorModifierFactory::to_array(mods);
        assert!(sorted[0].get_name() <= sorted[1].get_name());
    }
}
