use ffb_model::enums::{Rules, SkillId};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_player::UtilPlayer;
use crate::modifiers::dodge_context::DodgeContext;
use crate::modifiers::dodge_modifier::DodgeModifier;
use crate::modifiers::dodge_modifier_collection::DodgeModifierCollection;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.factory.DodgeModifierFactory.
///
/// BB2025 uses the base DodgeModifierCollection (tackle zones only — no prehensile tail).
/// TACKLEZONE selection counts opposing players with tackle zones at the target coordinate.
pub struct DodgeModifierFactory {
    collection: DodgeModifierCollection,
}

impl DodgeModifierFactory {
    /// Construct a factory for the given rules edition.
    /// BB2025 and all editions without an edition-specific subclass use the base collection.
    pub fn for_rules(_rules: Rules) -> Self {
        Self { collection: DodgeModifierCollection::new() }
    }

    /// 1:1 translation of DodgeModifierFactory.forName.
    pub fn for_name(&self, name: &str) -> Option<&DodgeModifier> {
        self.collection.get_modifiers().iter().find(|m| m.get_name() == name)
    }

    /// Returns the modifiers applicable to the given context.
    /// 1:1 translation of GenerifiedModifierFactory.findModifiers + DodgeModifierFactory overrides.
    ///
    /// REGULAR modifiers filtered by predicate (base BB2025 collection has none).
    /// TACKLEZONE: one modifier selected by counting opponents with TZs at target coordinate.
    /// PREHENSILE_TAIL: 0 for BB2025 (no such modifiers in base collection).
    pub fn find_applicable<'a>(&'a self, context: &DodgeContext<'_>) -> Vec<&'a DodgeModifier> {
        let mut result: Vec<&'a DodgeModifier> = self.collection.get_modifiers().iter()
            .filter(|m| m.get_type() == ModifierType::REGULAR && m.applies_to_context(context))
            .collect();

        // Java: isAffectedByTackleZones → !player.hasSkillProperty(ignoreTacklezonesWhenDodging)
        let player = context.acting_player.player_id.as_deref()
            .and_then(|id| context.game.player(id));
        let affected_by_tackle_zones = player
            .map(|p| !p.has_skill_property(NamedProperties::IGNORE_TACKLEZONES_WHEN_DODGING))
            .unwrap_or(true);

        if affected_by_tackle_zones {
            // Java: count opponents with TZs at target coordinate, excluding hasNoTacklezoneForDodging
            let count = self.count_tacklezones(context) as i32;
            if count > 0 {
                if let Some(tz_mod) = self.collection.get_modifiers().iter()
                    .find(|m| m.get_type() == ModifierType::TACKLEZONE && m.get_modifier() == count)
                {
                    result.push(tz_mod);
                }
            }
        }

        result
    }

    /// Count opposing players with tackle zones at the target coordinate.
    /// Java: GenerifiedModifierFactory.numberOfTacklezones with DodgeModifierFactory override.
    fn count_tacklezones(&self, context: &DodgeContext<'_>) -> usize {
        let acting_player_id = match context.acting_player.player_id.as_deref() {
            Some(id) => id,
            None => return 0,
        };
        let other_team = UtilPlayer::find_other_team(context.game, acting_player_id);
        let adjacent = UtilPlayer::find_adjacent_players_with_tacklezones(
            context.game,
            other_team,
            context.target_coordinate,
            false,
        );
        adjacent.iter()
            .filter(|id| {
                context.game.player(id).map(|p| !p.has_skill_property(NamedProperties::HAS_NO_TACKLEZONE_FOR_DODGING)).unwrap_or(false)
            })
            .count()
    }

    /// Returns skill-based dodge modifiers for the acting player.
    /// 1:1 translation of GenerifiedModifierFactory skill iteration for DodgeModifierFactory.
    ///
    /// Java: skills iterate their registered DodgeModifiers; applicable ones are included.
    pub fn find_skill_modifiers(&self, context: &DodgeContext<'_>) -> Vec<DodgeModifier> {
        let rules = context.game.rules;
        let Some(player_id) = context.acting_player.player_id.as_deref() else { return vec![]; };
        let Some(player) = context.game.player(player_id) else { return vec![]; };
        let mut result = Vec::new();
        for skill_id in player.all_skill_ids() {
            match skill_id {
                SkillId::TwoHeads => {
                    // Java: common.TwoHeads registers DodgeModifier("Two Heads", -1, REGULAR).
                    result.push(DodgeModifier::new("Two Heads", -1, ModifierType::REGULAR));
                }
                SkillId::Titchy if rules == Rules::Bb2016 => {
                    // Java: bb2016.Titchy registers DodgeModifier("Titchy", -1, REGULAR).
                    result.push(DodgeModifier::new("Titchy", -1, ModifierType::REGULAR));
                }
                SkillId::Stunty if rules == Rules::Bb2016 => {
                    // Java: bb2016.Stunty registers DodgeModifier("Stunty", 0, REGULAR).
                    // Value 0 — no numerical change; acts as a named marker in modifier reporting.
                    result.push(DodgeModifier::new("Stunty", 0, ModifierType::REGULAR));
                }
                SkillId::BreakTackle if rules == Rules::Bb2016 => {
                    // Java: bb2016.BreakTackle registers DodgeModifier("Break Tackle", 0, REGULAR, useStrength=true)
                    // with predicate: context.isUseBreakTackle() || hasUnusedSkill(actingPlayer, skill).
                    let use_break_tackle = context.use_break_tackle
                        || (player.has_skill(SkillId::BreakTackle) && !player.used_skills.contains(&SkillId::BreakTackle));
                    if use_break_tackle {
                        result.push(DodgeModifier::new_with_use_strength(
                            "Break Tackle", 0, ModifierType::REGULAR, true,
                        ));
                    }
                }
                _ => {}
            }
        }
        result
    }

    /// 1:1 translation of AgilityMechanic.minimumRollDodge (BB2025).
    /// `max(2, agility + sum(modifier))`.
    pub fn minimum_roll(agility: i32, modifiers: &[&DodgeModifier]) -> i32 {
        let total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        (agility + total).max(2)
    }
}

impl Default for DodgeModifierFactory {
    fn default() -> Self {
        Self::for_rules(Rules::Bb2025)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::{ActingPlayer, Game, Team};
    use ffb_model::types::FieldCoordinate;
    use crate::modifiers::dodge_context::DodgeContext;

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

    fn make_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025)
    }

    #[test]
    fn no_opponents_no_modifiers() {
        let game = make_game();
        let factory = DodgeModifierFactory::for_rules(Rules::Bb2025);
        let acting = ActingPlayer::default();
        let src = FieldCoordinate::new(5, 5);
        let tgt = FieldCoordinate::new(6, 5);
        let ctx = DodgeContext::new(&game, &acting, src, tgt);
        let mods = factory.find_applicable(&ctx);
        assert!(mods.is_empty(), "No opponents should yield no dodge modifiers");
    }

    #[test]
    fn minimum_roll_no_modifiers() {
        // agility 3, no modifiers → max(2, 3) = 3
        assert_eq!(DodgeModifierFactory::minimum_roll(3, &[]), 3);
    }

    #[test]
    fn minimum_roll_with_one_tackle_zone() {
        let m = DodgeModifier::new("1 Tacklezone", 1, ModifierType::TACKLEZONE);
        // agility 3 + 1 = 4
        assert_eq!(DodgeModifierFactory::minimum_roll(3, &[&m]), 4);
    }

    #[test]
    fn minimum_roll_never_below_two() {
        assert_eq!(DodgeModifierFactory::minimum_roll(1, &[]), 2);
    }

    #[test]
    fn for_name_returns_tackle_zone_modifier() {
        let factory = DodgeModifierFactory::for_rules(Rules::Bb2025);
        assert!(factory.for_name("1 Tacklezone").is_some());
        assert!(factory.for_name("8 Tacklezones").is_some());
        assert!(factory.for_name("NonExistent").is_none());
    }

    fn make_game_with_player(rules: Rules) -> (ffb_model::model::Game, String) {
        use ffb_model::model::{Game, Player, Team};
        use ffb_model::enums::{PlayerType, PlayerGender, PS_STANDING, PlayerState};
        let pid = "p1".to_string();
        let p = Player {
            id: pid.clone(), name: "Test".into(), nr: 1, position_id: "pos".into(),
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
            special_rules: vec![], players: vec![p],
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
        game.field_model.set_player_coordinate(&pid, ffb_model::types::FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(&pid, PlayerState(PS_STANDING));
        (game, pid)
    }

    #[test]
    fn find_skill_modifiers_two_heads_applies_in_bb2025() {
        use ffb_model::model::{ActingPlayer, SkillWithValue};
        let (mut game, pid) = make_game_with_player(Rules::Bb2025);
        game.team_home.players[0].starting_skills.push(SkillWithValue::new(SkillId::TwoHeads));
        let factory = DodgeModifierFactory::for_rules(Rules::Bb2025);
        let mut acting = ActingPlayer::default();
        acting.player_id = Some(pid.clone());
        let ctx = DodgeContext::new(&game, &acting, ffb_model::types::FieldCoordinate::new(5, 5), ffb_model::types::FieldCoordinate::new(6, 5));
        let mods = factory.find_skill_modifiers(&ctx);
        assert!(mods.iter().any(|m| m.get_name() == "Two Heads"));
        assert_eq!(mods.iter().find(|m| m.get_name() == "Two Heads").unwrap().get_modifier(), -1);
    }

    #[test]
    fn find_skill_modifiers_titchy_only_in_bb2016() {
        use ffb_model::model::{ActingPlayer, SkillWithValue};
        let (mut game, pid) = make_game_with_player(Rules::Bb2016);
        game.team_home.players[0].starting_skills.push(SkillWithValue::new(SkillId::Titchy));
        let factory = DodgeModifierFactory::for_rules(Rules::Bb2016);
        let mut acting = ActingPlayer::default();
        acting.player_id = Some(pid.clone());
        let ctx = DodgeContext::new(&game, &acting, ffb_model::types::FieldCoordinate::new(5, 5), ffb_model::types::FieldCoordinate::new(6, 5));
        let mods = factory.find_skill_modifiers(&ctx);
        assert!(mods.iter().any(|m| m.get_name() == "Titchy"), "Titchy should appear in BB2016");
    }

    #[test]
    fn find_skill_modifiers_titchy_not_in_bb2025() {
        use ffb_model::model::{ActingPlayer, SkillWithValue};
        let (mut game, pid) = make_game_with_player(Rules::Bb2025);
        game.team_home.players[0].starting_skills.push(SkillWithValue::new(SkillId::Titchy));
        let factory = DodgeModifierFactory::for_rules(Rules::Bb2025);
        let mut acting = ActingPlayer::default();
        acting.player_id = Some(pid.clone());
        let ctx = DodgeContext::new(&game, &acting, ffb_model::types::FieldCoordinate::new(5, 5), ffb_model::types::FieldCoordinate::new(6, 5));
        let mods = factory.find_skill_modifiers(&ctx);
        assert!(!mods.iter().any(|m| m.get_name() == "Titchy"), "Titchy should not appear in BB2025");
    }

    #[test]
    fn find_skill_modifiers_break_tackle_applies_when_use_break_tackle_flag_set() {
        use ffb_model::model::{ActingPlayer, SkillWithValue};
        let (mut game, pid) = make_game_with_player(Rules::Bb2016);
        game.team_home.players[0].starting_skills.push(SkillWithValue::new(SkillId::BreakTackle));
        let factory = DodgeModifierFactory::for_rules(Rules::Bb2016);
        let mut acting = ActingPlayer::default();
        acting.player_id = Some(pid.clone());
        let ctx = DodgeContext::new_with_break_tackle(&game, &acting, ffb_model::types::FieldCoordinate::new(5, 5), ffb_model::types::FieldCoordinate::new(6, 5), true);
        let mods = factory.find_skill_modifiers(&ctx);
        let bt = mods.iter().find(|m| m.get_name() == "Break Tackle");
        assert!(bt.is_some(), "Break Tackle modifier should appear when use_break_tackle=true");
        assert!(bt.unwrap().is_use_strength(), "Break Tackle modifier should have use_strength=true");
    }

    #[test]
    fn find_skill_modifiers_no_skill_mods_without_skills() {
        use ffb_model::model::ActingPlayer;
        let (game, pid) = make_game_with_player(Rules::Bb2025);
        let factory = DodgeModifierFactory::for_rules(Rules::Bb2025);
        let mut acting = ActingPlayer::default();
        acting.player_id = Some(pid.clone());
        let ctx = DodgeContext::new(&game, &acting, ffb_model::types::FieldCoordinate::new(5, 5), ffb_model::types::FieldCoordinate::new(6, 5));
        let mods = factory.find_skill_modifiers(&ctx);
        assert!(mods.is_empty(), "No skill modifiers without skills");
    }
}
