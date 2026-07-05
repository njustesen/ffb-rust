use ffb_model::enums::Rules;
use ffb_model::model::SpecialEffect;
use ffb_model::model::{Game, Player};
use crate::modifiers::injury_modifier::InjuryModifier;
use crate::modifiers::injury_modifier_context::InjuryModifierContext;
use crate::modifiers::injury_modifiers::InjuryModifiers;
use crate::modifiers::bb2016::injury_modifiers::Bb2016InjuryModifiers;
use crate::modifiers::bb2020::injury_modifiers::Bb2020InjuryModifiers;
use crate::modifiers::bb2025::injury_modifiers::Bb2025InjuryModifiers;
use crate::modifiers::modifier_aggregator::ModifierAggregator;

/// 1:1 translation of com.fumbbl.ffb.factory.InjuryModifierFactory.
pub struct InjuryModifierFactory {
    injury_modifiers: Box<dyn InjuryModifiers>,
    modifier_aggregator: ModifierAggregator,
}

impl InjuryModifierFactory {
    pub fn new(rules: Rules) -> Self {
        Self {
            injury_modifiers: make_injury_modifiers(rules),
            modifier_aggregator: ModifierAggregator::new(),
        }
    }

    /// Java: forName(String) → first modifier matching name across all_values + aggregator.
    pub fn for_name(&self, name: &str) -> Option<Box<dyn InjuryModifier>> {
        let from_collection = self.injury_modifiers.all_values()
            .into_iter()
            .find(|m| m.get_name() == name);
        if from_collection.is_some() { return from_collection; }

        // DEFERRED: modifier_aggregator.get_injury_modifiers() always empty (SkillFactory not ported)
        self.modifier_aggregator.get_injury_modifiers()
            .into_iter()
            .find(|m| m.get_name() == name)
    }

    /// Java: findInjuryModifiersWithoutNiggling — scans attacker then defender skills.
    /// DEFERRED: skill.getInjuryModifiers() requires SkillFactory (not ported).
    pub fn find_injury_modifiers_without_niggling(
        &self,
        _game: &Game,
        _attacker: Option<&Player>,
        _defender: &Player,
        _is_stab: bool,
        _is_foul: bool,
        _is_vomit_like: bool,
        _is_chainsaw: bool,
    ) -> Vec<Box<dyn InjuryModifier>> {
        // DEFERRED: UtilCards.findAllSkills(attacker/defender).flatMap(skill.getInjuryModifiers)
        // requires SkillFactory — returns empty for now.
        vec![]
    }

    /// Java: findInjuryModifiers — without-niggling + getNigglingInjuryModifier.
    pub fn find_injury_modifiers(
        &self,
        game: &Game,
        attacker: Option<&Player>,
        defender: &Player,
        is_stab: bool,
        is_foul: bool,
        is_vomit_like: bool,
    ) -> Vec<Box<dyn InjuryModifier>> {
        self.find_injury_modifiers_chainsaw(game, attacker, defender, is_stab, is_foul, is_vomit_like, false)
    }

    /// Java: findInjuryModifiers (with isChainsaw parameter).
    pub fn find_injury_modifiers_chainsaw(
        &self,
        game: &Game,
        attacker: Option<&Player>,
        defender: &Player,
        is_stab: bool,
        is_foul: bool,
        is_vomit_like: bool,
        is_chainsaw: bool,
    ) -> Vec<Box<dyn InjuryModifier>> {
        let mut modifiers = self.find_injury_modifiers_without_niggling(
            game, attacker, defender, is_stab, is_foul, is_vomit_like, is_chainsaw,
        );
        if let Some(niggling) = self.get_niggling_injury_modifier(defender) {
            modifiers.push(niggling);
        }
        modifiers
    }

    /// Java: getNigglingInjuryModifier(Player) — finds niggling modifier matching player's NI count.
    pub fn get_niggling_injury_modifier(&self, player: &Player) -> Option<Box<dyn InjuryModifier>> {
        let count = player.niggling_injuries;
        if count <= 0 { return None; }
        self.injury_modifiers.values()
            .into_iter()
            .find(|m| m.is_niggling_injury_modifier() && m.get_modifier(None, player) == count)
    }

    /// Java: specialEffectInjuryModifiers(SpecialEffect) — returns modifiers for given effect.
    pub fn special_effect_injury_modifiers(&self, special_effect: SpecialEffect) -> Vec<Box<dyn InjuryModifier>> {
        self.injury_modifiers.values()
            .into_iter()
            .filter(|m| m.get_special_effect() == Some(special_effect))
            .collect()
    }

    /// Initialize with use_all flag (controls legacy Bomb modifier inclusion).
    pub fn set_use_all(&mut self, use_all: bool) {
        self.injury_modifiers.set_use_all(use_all);
    }
}

fn make_injury_modifiers(rules: Rules) -> Box<dyn InjuryModifiers> {
    match rules {
        Rules::Bb2016 => Box::new(Bb2016InjuryModifiers),
        Rules::Bb2020 => Box::new(Bb2020InjuryModifiers::new()),
        Rules::Bb2025 | Rules::Common => Box::new(Bb2025InjuryModifiers),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, PlayerGender};

    fn dummy_player_with_nigglings(id: &str, count: i32) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: count, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        }
    }

    #[test]
    fn for_name_finds_niggling() {
        let f = InjuryModifierFactory::new(Rules::Bb2016);
        assert!(f.for_name("1 Niggling Injury").is_some());
    }

    #[test]
    fn for_name_returns_none_in_bb2025() {
        let f = InjuryModifierFactory::new(Rules::Bb2025);
        assert!(f.for_name("1 Niggling Injury").is_none());
    }

    #[test]
    fn get_niggling_modifier_matches_count() {
        let f = InjuryModifierFactory::new(Rules::Bb2016);
        let p = dummy_player_with_nigglings("p", 2);
        let m = f.get_niggling_injury_modifier(&p);
        assert!(m.is_some());
        assert_eq!(m.unwrap().get_name(), "2 Niggling Injuries");
    }

    #[test]
    fn get_niggling_modifier_zero_returns_none() {
        let f = InjuryModifierFactory::new(Rules::Bb2016);
        let p = dummy_player_with_nigglings("p", 0);
        assert!(f.get_niggling_injury_modifier(&p).is_none());
    }

    #[test]
    fn special_effect_fireball_bb2025() {
        let f = InjuryModifierFactory::new(Rules::Bb2025);
        let mods = f.special_effect_injury_modifiers(SpecialEffect::FIREBALL);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "Fireball");
    }

    #[test]
    fn special_effect_bomb_not_in_bb2025() {
        let f = InjuryModifierFactory::new(Rules::Bb2025);
        assert!(f.special_effect_injury_modifiers(SpecialEffect::BOMB).is_empty());
    }

    #[test]
    fn special_effect_bomb_in_bb2020_use_all() {
        let mut f = InjuryModifierFactory::new(Rules::Bb2020);
        f.set_use_all(true);
        assert!(!f.special_effect_injury_modifiers(SpecialEffect::BOMB).is_empty());
    }
}
