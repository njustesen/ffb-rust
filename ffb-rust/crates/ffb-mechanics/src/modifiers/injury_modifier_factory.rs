use ffb_model::enums::{Rules, SkillId};
use ffb_model::model::SpecialEffect;
use ffb_model::model::{Game, Player};
use crate::modifiers::injury_modifier::InjuryModifier;
use crate::modifiers::injury_modifier_context::InjuryModifierContext;
use crate::modifiers::injury_modifiers::InjuryModifiers;
use crate::modifiers::bb2016::injury_modifiers::Bb2016InjuryModifiers;
use crate::modifiers::bb2020::injury_modifiers::Bb2020InjuryModifiers;
use crate::modifiers::bb2025::injury_modifiers::Bb2025InjuryModifiers;
use crate::modifiers::modifier_aggregator::ModifierAggregator;
use crate::modifiers::static_injury_modifier_attacker::StaticInjuryModifierAttacker;

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

        // ModifierAggregator is intentionally empty — per-skill injury modifier lookup uses direct matching.
        self.modifier_aggregator.get_injury_modifiers()
            .into_iter()
            .find(|m| m.get_name() == name)
    }

    /// Java: findInjuryModifiersWithoutNiggling — scans attacker then defender skills.
    pub fn find_injury_modifiers_without_niggling(
        &self,
        game: &Game,
        attacker: Option<&Player>,
        defender: &Player,
        is_stab: bool,
        is_foul: bool,
        is_vomit_like: bool,
        is_chainsaw: bool,
    ) -> Vec<Box<dyn InjuryModifier>> {
        let mut context = InjuryModifierContext::new(game, attacker, defender, is_stab, is_foul, is_vomit_like, is_chainsaw);
        let mut modifiers = get_injury_modifiers_from_skills(attacker, &context);
        context.set_defender_mode();
        modifiers.extend(get_injury_modifiers_from_skills(Some(defender), &context));
        modifiers
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

/// Java: InjuryModifierFactory.getInjuryModifiers — iterates player skills in current mode.
fn get_injury_modifiers_from_skills(
    player: Option<&Player>,
    context: &InjuryModifierContext,
) -> Vec<Box<dyn InjuryModifier>> {
    let Some(player) = player else { return vec![]; };
    player
        .all_skill_ids()
        .filter_map(|skill_id| skill_to_injury_modifier(skill_id, context))
        .collect()
}

/// Maps a SkillId to its injury modifier for the given context.
/// Translates Java `Skill.getInjuryModifiers()` + `appliesToContext` per-skill logic.
/// Note: the Java "either/or" check (InjuryContext.getArmorModifiers) is not yet ported;
/// both armor and injury modifiers are offered independently.
fn skill_to_injury_modifier(
    skill_id: SkillId,
    context: &InjuryModifierContext,
) -> Option<Box<dyn InjuryModifier>> {
    if context.is_defender_mode() { return None; }
    match skill_id {
        SkillId::MightyBlow => {
            if context.is_foul || context.is_stab || context.is_vomit_like || context.is_chainsaw {
                return None;
            }
            // BB2025: attacker must not be distracted.
            if context.game.rules == Rules::Bb2025 {
                if let Some(a) = context.attacker {
                    if context.game.field_model.player_state(&a.id)
                        .map_or(false, |s| s.is_distracted()) { return None; }
                }
            }
            Some(Box::new(StaticInjuryModifierAttacker::new("Mighty Blow", 1, false)))
        }
        SkillId::DirtyPlayer => {
            if context.is_foul {
                Some(Box::new(StaticInjuryModifierAttacker::new("Dirty Player", 1, false)))
            } else {
                None
            }
        }
        SkillId::LethalFlight => {
            if !context.is_ttm { return None; }
            let attacker = context.attacker?;
            let attacker_team = context.game.player_team_id(&attacker.id);
            let defender_team = context.game.player_team_id(&context.defender.id);
            if attacker_team.is_none() || attacker_team == defender_team { return None; }
            Some(Box::new(StaticInjuryModifierAttacker::new("Lethal Flight", 1, false)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, PlayerGender, SkillId};

    fn dummy_player_with_nigglings(id: &str, count: i32) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: count, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
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

    fn bare_player(id: &str) -> Player {
        dummy_player_with_nigglings(id, 0)
    }

    fn player_with_skill(id: &str, skill_id: SkillId) -> Player {
        use ffb_model::model::skill_def::SkillWithValue;
        let mut p = bare_player(id);
        p.starting_skills = vec![SkillWithValue { skill_id, value: None }];
        p
    }

    fn make_game(rules: Rules) -> Game {
        use ffb_model::model::team::Team;
        let home = Team {
            id: "home".into(), name: "home".into(), race: "human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        };
        let away = home.clone();
        Game::new(home, away, rules)
    }

    #[test]
    fn find_injury_modifiers_mighty_blow_applies_on_block() {
        let f = InjuryModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::MightyBlow);
        let defender = bare_player("d");
        let mods = f.find_injury_modifiers_without_niggling(&game, Some(&attacker), &defender, false, false, false, false);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "Mighty Blow");
    }

    #[test]
    fn find_injury_modifiers_mighty_blow_ignores_foul() {
        let f = InjuryModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::MightyBlow);
        let defender = bare_player("d");
        let mods = f.find_injury_modifiers_without_niggling(&game, Some(&attacker), &defender, false, true, false, false);
        assert!(mods.is_empty());
    }

    #[test]
    fn find_injury_modifiers_mighty_blow_ignores_stab() {
        let f = InjuryModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::MightyBlow);
        let defender = bare_player("d");
        let mods = f.find_injury_modifiers_without_niggling(&game, Some(&attacker), &defender, true, false, false, false);
        assert!(mods.is_empty());
    }

    #[test]
    fn find_injury_modifiers_dirty_player_applies_on_foul() {
        let f = InjuryModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::DirtyPlayer);
        let defender = bare_player("d");
        let mods = f.find_injury_modifiers_without_niggling(&game, Some(&attacker), &defender, false, true, false, false);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "Dirty Player");
    }

    #[test]
    fn find_injury_modifiers_dirty_player_ignores_block() {
        let f = InjuryModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::DirtyPlayer);
        let defender = bare_player("d");
        let mods = f.find_injury_modifiers_without_niggling(&game, Some(&attacker), &defender, false, false, false, false);
        assert!(mods.is_empty());
    }

    #[test]
    fn find_injury_modifiers_no_attacker_returns_empty() {
        let f = InjuryModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let defender = bare_player("d");
        let mods = f.find_injury_modifiers_without_niggling(&game, None, &defender, false, false, false, false);
        assert!(mods.is_empty());
    }

    #[test]
    fn find_injury_modifiers_chainsaw_skips_mighty_blow() {
        let f = InjuryModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::MightyBlow);
        let defender = bare_player("d");
        let mods = f.find_injury_modifiers_without_niggling(&game, Some(&attacker), &defender, false, false, false, true);
        assert!(mods.is_empty());
    }
}
