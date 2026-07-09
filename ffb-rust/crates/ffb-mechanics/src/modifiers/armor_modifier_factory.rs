use ffb_model::enums::{Rules, SkillId};
use ffb_model::model::SpecialEffect;
use ffb_model::model::{Game, Player};
use ffb_model::util::util_player::UtilPlayer;
use ffb_model::model::property::named_properties::NamedProperties;
use crate::modifiers::armor_modifier::ArmorModifier;
use crate::modifiers::armor_modifier_context::ArmorModifierContext;
use crate::modifiers::armor_modifiers::ArmorModifiers;
use crate::modifiers::bb2016::armor_modifiers::Bb2016ArmorModifiers;
use crate::modifiers::bb2020::armor_modifiers::Bb2020ArmorModifiers;
use crate::modifiers::bb2025::armor_modifiers::Bb2025ArmorModifiers;
use crate::modifiers::modifier_aggregator::ModifierAggregator;
use crate::modifiers::static_armour_modifier::StaticArmourModifier;

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

        // ModifierAggregator is intentionally empty — per-skill armor modifier lookup uses direct matching.
        self.modifier_aggregator.get_armour_modifiers()
            .into_iter()
            .find(|m| m.get_name() == name)
    }

    /// Java: findArmorModifiers — scans attacker skills for applicable modifiers.
    pub fn find_armor_modifiers(
        &self,
        game: &Game,
        attacker: Option<&Player>,
        defender: &Player,
        is_stab: bool,
        is_foul: bool,
    ) -> Vec<Box<dyn ArmorModifier>> {
        if defender.has_skill_property(NamedProperties::IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS) {
            return vec![];
        }
        let context = ArmorModifierContext::new(game, attacker, defender, is_stab, is_foul);
        get_armor_modifiers_from_skills(attacker, &context)
    }

    /// Java: specialEffectArmourModifiers(SpecialEffect, Player) — returns modifiers for given effect.
    pub fn special_effect_armour_modifiers(
        &self,
        special_effect: SpecialEffect,
        defender: &Player,
    ) -> Vec<Box<dyn ArmorModifier>> {
        if defender.has_skill_property(NamedProperties::IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS) {
            return vec![];
        }
        self.armor_modifiers.values()
            .into_iter()
            .filter(|m| m.get_special_effect() == Some(special_effect))
            .collect()
    }

    /// Java: getFoulAssist(ArmorModifierContext) — returns foul-assist modifiers matching context.
    pub fn get_foul_assist(&self, context: &ArmorModifierContext<'_>) -> Vec<Box<dyn ArmorModifier>> {
        if context.defender.has_skill_property(NamedProperties::IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS) {
            return vec![];
        }
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

/// Java: ArmorModifierFactory.getArmorModifiers(Player, ArmorModifierContext) —
/// iterates attacker skills and returns their applicable armor modifiers.
fn get_armor_modifiers_from_skills(
    attacker: Option<&Player>,
    context: &ArmorModifierContext,
) -> Vec<Box<dyn ArmorModifier>> {
    let Some(attacker) = attacker else { return vec![]; };
    attacker
        .all_skill_ids()
        .filter_map(|skill_id| skill_to_armor_modifier(skill_id, context))
        .collect()
}

/// Maps a SkillId to its armor modifier for the given context, returning None if not applicable.
/// Translates the Java `Skill.getArmorModifiers()` + `appliesToContext` per-skill logic.
fn skill_to_armor_modifier(
    skill_id: SkillId,
    context: &ArmorModifierContext,
) -> Option<Box<dyn ArmorModifier>> {
    match skill_id {
        SkillId::MightyBlow => {
            if context.is_stab || context.is_foul { return None; }
            // BB2025 added: attacker must not be distracted.
            if context.game.rules == Rules::Bb2025 {
                if let Some(a) = context.attacker {
                    if context.game.field_model.player_state(&a.id)
                        .map_or(false, |s| s.is_distracted()) { return None; }
                }
            }
            Some(Box::new(StaticArmourModifier::new("Mighty Blow", 1, false)))
        }
        SkillId::DirtyPlayer => {
            if context.is_foul {
                Some(Box::new(StaticArmourModifier::new("Dirty Player", 1, false)))
            } else {
                None
            }
        }
        SkillId::Claw => {
            if context.is_stab || context.is_foul { return None; }
            if context.attacker.map_or(false, |a| a.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW)) {
                return None;
            }
            // Java: context.getDefender().getArmourWithModifiers() > 8
            if context.defender.armour_with_modifiers() > 8 {
                Some(Box::new(StaticArmourModifier::new("Claws", 0, false)))
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
            Some(Box::new(StaticArmourModifier::new("Lethal Flight", 1, false)))
        }
        SkillId::Stakes => {
            // BB2016 only: bonus on Stab against undead teams/positions.
            // Java: Stakes.postConstruct registers StaticArmourModifier(+1, is_stab AND target is undead).
            if context.game.rules != Rules::Bb2016 { return None; }
            if !context.is_stab { return None; }
            let _attacker = context.attacker?;
            // Determine which team the defender belongs to.
            let defender_team = if context.game.team_home.has_player(&context.defender.id) {
                &context.game.team_home
            } else {
                &context.game.team_away
            };
            // Java: otherTeam.getRoster().isUndead() — covers Necromantic, Undead, Vampire teams.
            // NOTE: Khemri is not captured by necromancer/vampire_lord flags (position-level check needed).
            let team_is_undead = defender_team.necromancer || defender_team.vampire_lord;
            if team_is_undead {
                Some(Box::new(StaticArmourModifier::new("Stakes", 1, false)))
            } else {
                None
            }
        }
        SkillId::ASneakyPair => {
            // BB2025 only: bonus on foul/stab when a partner is adjacent to the defender.
            // Java: ASneakyPair.postConstruct (BB2025) registers StaticArmourModifier(+1, foul OR stab, partnerMarksDefender).
            if context.game.rules != Rules::Bb2025 { return None; }
            if !context.is_foul && !context.is_stab { return None; }
            if !UtilPlayer::partner_marks_defender(context.game, &context.defender.id, SkillId::ASneakyPair) {
                return None;
            }
            Some(Box::new(StaticArmourModifier::new("A Sneaky Pair", 1, false)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::team::Team;
    use ffb_model::enums::{PlayerType, PlayerGender, SkillId};

    fn dummy_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
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
            vampire_lord: false,
            necromancer: false,
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

    fn player_with_skill(id: &str, skill_id: SkillId) -> Player {
        use ffb_model::model::skill_def::SkillWithValue;
        let mut p = dummy_player(id);
        p.starting_skills = vec![SkillWithValue { skill_id, value: None }];
        p
    }

    fn player_with_armour(id: &str, armour: i32) -> Player {
        let mut p = dummy_player(id);
        p.armour = armour;
        p
    }

    #[test]
    fn find_armor_modifiers_mighty_blow_applies_on_block() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::MightyBlow);
        let defender = dummy_player("d");
        let mods = f.find_armor_modifiers(&game, Some(&attacker), &defender, false, false);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "Mighty Blow");
        assert_eq!(mods[0].get_modifier(Some(&attacker), &defender), 1);
    }

    #[test]
    fn find_armor_modifiers_mighty_blow_ignores_stab() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::MightyBlow);
        let defender = dummy_player("d");
        let mods = f.find_armor_modifiers(&game, Some(&attacker), &defender, true, false);
        assert!(mods.is_empty());
    }

    #[test]
    fn find_armor_modifiers_mighty_blow_ignores_foul() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::MightyBlow);
        let defender = dummy_player("d");
        let mods = f.find_armor_modifiers(&game, Some(&attacker), &defender, false, true);
        assert!(mods.is_empty());
    }

    #[test]
    fn find_armor_modifiers_dirty_player_applies_on_foul() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::DirtyPlayer);
        let defender = dummy_player("d");
        let mods = f.find_armor_modifiers(&game, Some(&attacker), &defender, false, true);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "Dirty Player");
    }

    #[test]
    fn find_armor_modifiers_dirty_player_ignores_block() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::DirtyPlayer);
        let defender = dummy_player("d");
        let mods = f.find_armor_modifiers(&game, Some(&attacker), &defender, false, false);
        assert!(mods.is_empty());
    }

    #[test]
    fn find_armor_modifiers_claws_applies_when_armour_high() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::Claw);
        let defender = player_with_armour("d", 9);
        let mods = f.find_armor_modifiers(&game, Some(&attacker), &defender, false, false);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "Claws");
    }

    #[test]
    fn find_armor_modifiers_claws_ignores_low_armour() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::Claw);
        let defender = player_with_armour("d", 8);
        let mods = f.find_armor_modifiers(&game, Some(&attacker), &defender, false, false);
        assert!(mods.is_empty());
    }

    #[test]
    fn find_armor_modifiers_iron_hard_skin_blocks_all() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = player_with_skill("a", SkillId::MightyBlow);
        let defender = player_with_skill("d", SkillId::IronHardSkin);
        let mods = f.find_armor_modifiers(&game, Some(&attacker), &defender, false, false);
        assert!(mods.is_empty());
    }

    #[test]
    fn special_effect_iron_hard_skin_blocks_fireball() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let defender = player_with_skill("d", SkillId::IronHardSkin);
        let mods = f.special_effect_armour_modifiers(SpecialEffect::FIREBALL, &defender);
        assert!(mods.is_empty());
    }

    #[test]
    fn get_foul_assist_iron_hard_skin_blocks_assists() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let attacker = dummy_player("a");
        let defender = player_with_skill("d", SkillId::IronHardSkin);
        let ctx = ArmorModifierContext::new_with_foul_assists(&game, Some(&attacker), &defender, false, true, 3);
        let mods = f.get_foul_assist(&ctx);
        assert!(mods.is_empty());
    }

    #[test]
    fn find_armor_modifiers_no_attacker_returns_empty() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_game(Rules::Bb2025);
        let defender = dummy_player("d");
        let mods = f.find_armor_modifiers(&game, None, &defender, false, false);
        assert!(mods.is_empty());
    }

    // ── ASneakyPair ───────────────────────────────────────────────────────────

    fn make_sneaky_pair_game() -> Game {
        use ffb_model::types::FieldCoordinate;
        use ffb_model::enums::PlayerState;
        // home: "def" (the fouled player)
        // away: "a1" (attacker/main), "a2" (partner) both with ASneakyPair
        let def = dummy_player("def");
        let a1 = player_with_skill("a1", SkillId::ASneakyPair);
        let a2 = player_with_skill("a2", SkillId::ASneakyPair);
        let home = make_team("home", vec![def]);
        let away = make_team("away", vec![a1.clone(), a2.clone()]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        // ACTIVE_STANDING = 0x101
        let standing = PlayerState(0x101);
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def", standing);
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("a1", standing);
        game.field_model.set_player_coordinate("a2", FieldCoordinate::new(5, 6));
        game.field_model.set_player_state("a2", standing);
        game
    }

    #[test]
    fn a_sneaky_pair_applies_on_foul_with_two_partners() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_sneaky_pair_game();
        let attacker = player_with_skill("a1", SkillId::ASneakyPair);
        let defender = dummy_player("def");
        let mods = f.find_armor_modifiers(&game, Some(&attacker), &defender, false, true);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "A Sneaky Pair");
        assert_eq!(mods[0].get_modifier(Some(&attacker), &defender), 1);
    }

    #[test]
    fn a_sneaky_pair_applies_on_stab_with_two_partners() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_sneaky_pair_game();
        let attacker = player_with_skill("a1", SkillId::ASneakyPair);
        let defender = dummy_player("def");
        let mods = f.find_armor_modifiers(&game, Some(&attacker), &defender, true, false);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "A Sneaky Pair");
    }

    #[test]
    fn a_sneaky_pair_requires_foul_or_stab() {
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        let game = make_sneaky_pair_game();
        let attacker = player_with_skill("a1", SkillId::ASneakyPair);
        let defender = dummy_player("def");
        // block action (neither foul nor stab) → no modifier
        let mods = f.find_armor_modifiers(&game, Some(&attacker), &defender, false, false);
        assert!(mods.is_empty());
    }

    #[test]
    fn a_sneaky_pair_requires_partner_adjacent() {
        use ffb_model::types::FieldCoordinate;
        use ffb_model::enums::PlayerState;
        let f = ArmorModifierFactory::new(Rules::Bb2025);
        // only one attacker with the skill — no partner → no modifier
        let home = make_team("home", vec![dummy_player("def")]);
        let away = make_team("away", vec![player_with_skill("a1", SkillId::ASneakyPair)]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        let standing = PlayerState(0x101);
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def", standing);
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("a1", standing);
        let attacker = player_with_skill("a1", SkillId::ASneakyPair);
        let defender = dummy_player("def");
        let mods = f.find_armor_modifiers(&game, Some(&attacker), &defender, false, true);
        assert!(mods.is_empty());
    }

    #[test]
    fn a_sneaky_pair_requires_bb2025() {
        let f = ArmorModifierFactory::new(Rules::Bb2016);
        let game = make_sneaky_pair_game();
        let attacker = player_with_skill("a1", SkillId::ASneakyPair);
        let defender = dummy_player("def");
        // BB2016 game → skill not active regardless of foul
        let mods = f.find_armor_modifiers(&game, Some(&attacker), &defender, false, true);
        // BB2016 factory may not even have ASneakyPair, but either way the context check fires first
        // Important: ArmorModifierFactory uses Rules from its constructor for the modifiers collection,
        // but skill_to_armor_modifier checks context.game.rules — here game.rules == Bb2025 so we need
        // the factory to be Bb2016 AND the game rules to be Bb2016.
        drop(mods); // just verify it compiles and doesn't panic
        let mut game2 = make_sneaky_pair_game();
        game2.rules = Rules::Bb2016;
        let mods2 = f.find_armor_modifiers(&game2, Some(&attacker), &defender, false, true);
        assert!(mods2.is_empty());
    }
}
