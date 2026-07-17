use ffb_model::enums::{Rules, SkillId};
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

    /// Returns skill-based jump modifiers for the player.
    ///
    /// `accumulated_modifier` is the sum of modifier values from `find_applicable()` (TACKLEZONE +
    /// PREHENSILE_TAIL). Required for DEPENDS_ON_SUM_OF_OTHERS modifiers (Leap, BB2020 VeryLongLegs).
    ///
    /// Java: skills iterate their registered JumpModifiers; applicable ones are included.
    ///
    /// - bb2025/bb2016.VeryLongLegs: JumpModifier("Very Long Legs", -1, REGULAR).
    /// - bb2020.VeryLongLegs: JumpModifier(-1, DEPENDS_ON_SUM_OF_OTHERS), applies when accumulated > 1.
    /// - bb2020.Leap: JumpModifier(-1, DEPENDS_ON_SUM_OF_OTHERS), applies when accumulated > 1.
    /// - bb2025.Leap: JumpModifier(-1, DEPENDS_ON_SUM_OF_OTHERS), applies when accumulated > 1
    ///   OR (accumulated > 0 AND modifier_count > 1).
    /// - bb2016.Leap: no JumpModifier (just the canLeap property).
    pub fn find_skill_modifiers(
        &self,
        ctx: &JumpContext<'_>,
        accumulated_modifier: i32,
        modifier_count: i32,
    ) -> Vec<JumpModifier> {
        let rules = ctx.game.rules;
        let player = &ctx.player;
        let mut result = Vec::new();
        for skill_id in player.all_skill_ids() {
            match skill_id {
                SkillId::VeryLongLegs => {
                    match rules {
                        Rules::Bb2020 => {
                            // DEPENDS_ON_SUM_OF_OTHERS: applies only when accumulated modifier > 1.
                            if accumulated_modifier > 1 {
                                result.push(JumpModifier::new(
                                    "Very Long Legs", -1, ModifierType::DEPENDS_ON_SUM_OF_OTHERS,
                                ));
                            }
                        }
                        _ => {
                            // BB2025 and BB2016: REGULAR — always applies.
                            result.push(JumpModifier::new("Very Long Legs", -1, ModifierType::REGULAR));
                        }
                    }
                }
                SkillId::Leap => {
                    match rules {
                        Rules::Bb2020 => {
                            if accumulated_modifier > 1 {
                                result.push(JumpModifier::new("Leap", -1, ModifierType::DEPENDS_ON_SUM_OF_OTHERS));
                            }
                        }
                        Rules::Bb2025 | Rules::Common => {
                            if accumulated_modifier > 1
                                || (accumulated_modifier > 0 && modifier_count > 1)
                            {
                                result.push(JumpModifier::new("Leap", -1, ModifierType::DEPENDS_ON_SUM_OF_OTHERS));
                            }
                        }
                        _ => {} // BB2016: no Leap jump modifier
                    }
                }
                _ => {}
            }
        }
        result
    }

    /// Java: `ModifierAggregator.getJumpModifiers()`'s skill half. Unlike `find_skill_modifiers`
    /// (context-scoped: the `accumulated_modifier > 1`/`modifier_count > 1` guards are runtime
    /// predicates evaluated by `GenerifiedModifierFactory.findModifiers`, not conditions on
    /// whether the skill registers the modifier at all), this returns every edition-applicable
    /// skill's raw registered `JumpModifier`, matching Java's `Skill.getJumpModifiers()`. Only
    /// `VeryLongLegs` (all editions) and `Leap` (bb2020/bb2025 only) register one.
    pub fn find_registered_modifiers(rules: Rules) -> Vec<JumpModifier> {
        let mut result = Vec::new();
        for skill_id in ffb_model::factory::skill_factory::SkillFactory::new().get_skills() {
            match skill_id {
                SkillId::VeryLongLegs => {
                    let modifier_type = if rules == Rules::Bb2020 {
                        ModifierType::DEPENDS_ON_SUM_OF_OTHERS
                    } else {
                        ModifierType::REGULAR
                    };
                    result.push(JumpModifier::new("Very Long Legs", -1, modifier_type));
                }
                SkillId::Leap if matches!(rules, Rules::Bb2020 | Rules::Bb2025 | Rules::Common) => {
                    result.push(JumpModifier::new("Leap", -1, ModifierType::DEPENDS_ON_SUM_OF_OTHERS));
                }
                _ => {}
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
    fn find_registered_modifiers_bb2016_has_only_very_long_legs() {
        let mods = JumpModifierFactory::find_registered_modifiers(Rules::Bb2016);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_name(), "Very Long Legs");
    }

    #[test]
    fn find_registered_modifiers_bb2025_has_both() {
        let mods = JumpModifierFactory::find_registered_modifiers(Rules::Bb2025);
        let names: Vec<&str> = mods.iter().map(|m| m.get_name()).collect();
        assert!(names.contains(&"Very Long Legs"));
        assert!(names.contains(&"Leap"));
        assert_eq!(mods.len(), 2);
    }

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

    fn make_game_with_jumping_player(rules: Rules) -> (ffb_model::model::Game, ffb_model::model::Player) {
        use ffb_model::model::{Game, Player, Team};
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING, PlayerState};
        let mut p = Player {
            id: "j1".into(), name: "Jumper".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, is_big_guy: false,
            ..Default::default()
        };
        let home = Team {
            id: "home".into(), name: "home".into(), race: "human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
            fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![p.clone()],
            vampire_lord: false, necromancer: false,
        };
        let away = Team {
            id: "away".into(), name: "away".into(), race: "human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
            fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
            vampire_lord: false, necromancer: false,
        };
        let mut game = Game::new(home, away, rules);
        game.field_model.set_player_coordinate("j1", ffb_model::types::FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("j1", PlayerState(PS_STANDING));
        let player = game.player("j1").unwrap().clone();
        (game, player)
    }

    #[test]
    fn find_skill_modifiers_very_long_legs_bb2025_regular() {
        use ffb_model::model::SkillWithValue;
        use ffb_model::types::FieldCoordinate;
        let (mut game, mut player) = make_game_with_jumping_player(Rules::Bb2025);
        player.starting_skills.push(SkillWithValue::new(SkillId::VeryLongLegs));
        game.team_home.players[0] = player.clone();
        let factory = JumpModifierFactory::for_rules(Rules::Bb2025);
        let ctx = crate::modifiers::jump_context::JumpContext::new(&game, &player, FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 5));
        let mods = factory.find_skill_modifiers(&ctx, 0, 0);
        assert!(mods.iter().any(|m| m.get_name() == "Very Long Legs" && m.get_type() == ModifierType::REGULAR));
    }

    #[test]
    fn find_skill_modifiers_very_long_legs_bb2020_depends_on_sum_applies_when_accumulated_gt_1() {
        use ffb_model::model::SkillWithValue;
        use ffb_model::types::FieldCoordinate;
        let (mut game, mut player) = make_game_with_jumping_player(Rules::Bb2020);
        player.starting_skills.push(SkillWithValue::new(SkillId::VeryLongLegs));
        game.team_home.players[0] = player.clone();
        let factory = JumpModifierFactory::for_rules(Rules::Bb2020);
        let ctx = crate::modifiers::jump_context::JumpContext::new(&game, &player, FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 5));
        // No accumulated modifier → not applied
        assert!(factory.find_skill_modifiers(&ctx, 0, 0).is_empty());
        // Accumulated > 1 → applied
        let mods = factory.find_skill_modifiers(&ctx, 2, 1);
        assert!(mods.iter().any(|m| m.get_name() == "Very Long Legs"));
    }

    #[test]
    fn find_skill_modifiers_leap_bb2025_applies_when_accumulated_gt_1() {
        use ffb_model::model::SkillWithValue;
        use ffb_model::types::FieldCoordinate;
        let (mut game, mut player) = make_game_with_jumping_player(Rules::Bb2025);
        player.starting_skills.push(SkillWithValue::new(SkillId::Leap));
        game.team_home.players[0] = player.clone();
        let factory = JumpModifierFactory::for_rules(Rules::Bb2025);
        let ctx = crate::modifiers::jump_context::JumpContext::new(&game, &player, FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 5));
        // accumulated > 1 → applies
        let mods = factory.find_skill_modifiers(&ctx, 2, 1);
        assert!(mods.iter().any(|m| m.get_name() == "Leap"));
        // accumulated == 0, count == 0 → does not apply
        assert!(factory.find_skill_modifiers(&ctx, 0, 0).is_empty());
    }

    #[test]
    fn find_skill_modifiers_leap_bb2016_no_modifier() {
        use ffb_model::model::SkillWithValue;
        use ffb_model::types::FieldCoordinate;
        let (mut game, mut player) = make_game_with_jumping_player(Rules::Bb2016);
        player.starting_skills.push(SkillWithValue::new(SkillId::Leap));
        game.team_home.players[0] = player.clone();
        let factory = JumpModifierFactory::for_rules(Rules::Bb2016);
        let ctx = crate::modifiers::jump_context::JumpContext::new(&game, &player, FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 5));
        // BB2016 Leap has no JumpModifier — always empty
        assert!(factory.find_skill_modifiers(&ctx, 5, 3).is_empty());
    }
}
