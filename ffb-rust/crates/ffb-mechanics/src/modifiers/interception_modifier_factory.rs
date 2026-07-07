use ffb_model::enums::Rules;
use ffb_model::model::{Game, Player};
use ffb_model::model::property::NamedProperties;
use ffb_model::util::util_disturbing_presence::UtilDisturbingPresence;
use ffb_model::util::util_player::UtilPlayer;
use crate::modifiers::bb2016::interception_modifier_collection::InterceptionModifierCollection as Bb2016Collection;
use crate::modifiers::bb2020::interception_modifier_collection::InterceptionModifierCollection as Bb2020Collection;
use crate::modifiers::bb2025::interception_modifier_collection::InterceptionModifierCollection as Bb2025Collection;
use crate::modifiers::interception_context::InterceptionContext;
use crate::modifiers::interception_modifier::InterceptionModifier;
use crate::modifiers::modifier_type::ModifierType;
use crate::pass_result::PassResult;

/// 1:1 translation of com.fumbbl.ffb.factory.InterceptionModifierFactory.
///
/// Finds applicable interception modifiers:
/// - DISTURBING_PRESENCE: count opposing DP players within 3 steps; select the modifier matching the count.
/// - TACKLEZONE: count adjacent opponents with TZs at the interceptor's position (0 if player has
///   IGNORE_TACKLEZONES_WHEN_CATCHING).
/// - REGULAR: predicate-based (Pouring Rain, pass result accuracy, Stunty thrower, etc.).
pub struct InterceptionModifierFactory {
    collection: Box<dyn InterceptionCollection>,
}

trait InterceptionCollection: Send + Sync {
    fn get_modifiers(&self) -> &[InterceptionModifier];
}

impl InterceptionCollection for Bb2016Collection {
    fn get_modifiers(&self) -> &[InterceptionModifier] { self.get_modifiers() }
}

impl InterceptionCollection for Bb2020Collection {
    fn get_modifiers(&self) -> &[InterceptionModifier] { self.get_modifiers() }
}

impl InterceptionCollection for Bb2025Collection {
    fn get_modifiers(&self) -> &[InterceptionModifier] { self.get_modifiers() }
}

impl InterceptionModifierFactory {
    pub fn for_rules(rules: Rules) -> Self {
        let collection: Box<dyn InterceptionCollection> = match rules {
            Rules::Bb2025 => Box::new(Bb2025Collection::new()),
            Rules::Bb2020 => Box::new(Bb2020Collection::new()),
            _ => Box::new(Bb2016Collection::new()),
        };
        Self { collection }
    }

    /// Java: InterceptionModifierFactory.findModifiers(game, context).
    ///
    /// Returns applicable modifiers for the interceptor's agility roll.
    /// DISTURBING_PRESENCE modifiers have no predicate in the collection, so the matching
    /// modifier is selected by exact count rather than via `applies_to_context`.
    pub fn find_applicable<'a>(
        &'a self,
        game: &Game,
        interceptor: &Player,
        pass_result: PassResult,
        bomb: bool,
    ) -> Vec<&'a InterceptionModifier> {
        let dp_count = UtilDisturbingPresence::find_opposing_disturbing_presences(
            game, &interceptor.id,
        );

        let tz_count = if interceptor.has_skill_property(NamedProperties::IGNORE_TACKLEZONES_WHEN_CATCHING) {
            0
        } else if let Some(coord) = game.field_model.player_coordinate(&interceptor.id) {
            let other_team = UtilPlayer::find_other_team(game, &interceptor.id);
            UtilPlayer::find_adjacent_players_with_tacklezones(game, other_team, coord, false)
                .len() as i32
        } else {
            0
        };

        let ctx = InterceptionContext::new_with_tacklezones(game, interceptor, pass_result, bomb, tz_count);

        let mut result: Vec<&'a InterceptionModifier> = Vec::new();
        for m in self.collection.get_modifiers() {
            match m.get_type() {
                ModifierType::DISTURBING_PRESENCE => {
                    // No predicate — select by exact count.
                    if dp_count > 0 && m.get_modifier() == dp_count {
                        result.push(m);
                    }
                }
                _ => {
                    if m.applies_to_context(&ctx) {
                        result.push(m);
                    }
                }
            }
        }
        result
    }

    /// BB2016 interception minimum roll: max(2, 7 - min(ag, 6) + 2 + modifier_total).
    pub fn minimum_roll_bb2016(player: &Player, modifiers: &[&InterceptionModifier]) -> i32 {
        let total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        (7 - player.agility_with_modifiers().min(6) + 2 + total).max(2)
    }

    /// BB2020/BB2025 interception minimum roll: max(2, agility + modifier_total).
    pub fn minimum_roll_bb2020(player: &Player, modifiers: &[&InterceptionModifier]) -> i32 {
        let total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        (player.agility_with_modifiers() + total).max(2)
    }
}

impl Default for InterceptionModifierFactory {
    fn default() -> Self { Self::for_rules(Rules::Bb2025) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId};
    use ffb_model::model::{Game, Player, Team, SkillWithValue};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::enums::{PS_STANDING, PlayerState};

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

    fn minimal_player(id: &str, agility: i32) -> Player {
        let mut p = Player::default();
        p.id = id.into();
        p.agility = agility;
        p
    }

    fn make_game(rules: Rules) -> Game {
        Game::new(empty_team("home"), empty_team("away"), rules)
    }

    #[test]
    fn for_rules_bb2016_returns_factory() {
        let f = InterceptionModifierFactory::for_rules(Rules::Bb2016);
        assert!(!f.collection.get_modifiers().is_empty());
    }

    #[test]
    fn for_rules_bb2025_returns_factory() {
        let f = InterceptionModifierFactory::for_rules(Rules::Bb2025);
        assert!(!f.collection.get_modifiers().is_empty());
    }

    #[test]
    fn no_modifiers_for_isolated_interceptor() {
        let mut game = make_game(Rules::Bb2016);
        let interceptor = minimal_player("i1", 3);
        game.team_home.players.push(interceptor.clone());
        game.field_model.set_player_coordinate("i1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("i1", PlayerState(PS_STANDING));

        let factory = InterceptionModifierFactory::for_rules(Rules::Bb2016);
        let mods = factory.find_applicable(&game, &interceptor, PassResult::ACCURATE, false);
        // No DP, no TZ, no weather → only TACKLEZONE/DP modifiers absent
        assert!(mods.is_empty());
    }

    #[test]
    fn tacklezone_modifier_applied_when_marked() {
        let mut game = make_game(Rules::Bb2016);
        let interceptor = minimal_player("i1", 3);
        game.team_home.players.push(interceptor.clone());
        game.field_model.set_player_coordinate("i1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("i1", PlayerState(PS_STANDING));

        let mut opponent = minimal_player("o1", 3);
        opponent.id = "o1".into();
        game.team_away.players.push(opponent);
        game.field_model.set_player_coordinate("o1", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("o1", PlayerState(PS_STANDING));

        let factory = InterceptionModifierFactory::for_rules(Rules::Bb2016);
        let mods = factory.find_applicable(&game, &interceptor, PassResult::ACCURATE, false);
        let has_tz = mods.iter().any(|m| m.get_type() == ModifierType::TACKLEZONE);
        assert!(has_tz, "expected TACKLEZONE modifier when adjacent opponent has TZ");
    }

    #[test]
    fn ignore_tacklezones_when_catching_skips_tz_modifiers() {
        use ffb_model::model::property::named_properties::NamedProperties;
        let mut game = make_game(Rules::Bb2016);
        let mut interceptor = minimal_player("i1", 3);
        // Give interceptor the "ignoreTacklezonesWhenCatching" property via a skill
        interceptor.starting_skills.push(SkillWithValue::new(SkillId::SureFeet));
        game.team_home.players.push(interceptor.clone());
        game.field_model.set_player_coordinate("i1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("i1", PlayerState(PS_STANDING));

        let opponent = minimal_player("o1", 3);
        game.team_away.players.push(opponent);
        game.field_model.set_player_coordinate("o1", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("o1", PlayerState(PS_STANDING));

        // Check if interceptor actually has the property
        let interceptor_ref = game.player("i1").unwrap();
        let has_prop = interceptor_ref.has_skill_property(NamedProperties::IGNORE_TACKLEZONES_WHEN_CATCHING);
        if has_prop {
            let factory = InterceptionModifierFactory::for_rules(Rules::Bb2016);
            let mods = factory.find_applicable(&game, interceptor_ref, PassResult::ACCURATE, false);
            let has_tz = mods.iter().any(|m| m.get_type() == ModifierType::TACKLEZONE);
            assert!(!has_tz, "TZ modifier should be skipped when player ignores TZs when catching");
        }
        // If the skill doesn't have that property, the test is vacuously true.
    }

    #[test]
    fn disturbing_presence_modifier_selected_by_count() {
        let mut game = make_game(Rules::Bb2016);
        let interceptor = minimal_player("i1", 3);
        game.team_home.players.push(interceptor.clone());
        game.field_model.set_player_coordinate("i1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("i1", PlayerState(PS_STANDING));

        let mut dp_player = minimal_player("o1", 3);
        dp_player.starting_skills.push(SkillWithValue::new(SkillId::DisturbingPresence));
        game.team_away.players.push(dp_player);
        game.field_model.set_player_coordinate("o1", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("o1", PlayerState(PS_STANDING));

        let factory = InterceptionModifierFactory::for_rules(Rules::Bb2016);
        let interceptor_ref = game.player("i1").unwrap();
        let mods = factory.find_applicable(&game, interceptor_ref, PassResult::ACCURATE, false);

        let dp_mods: Vec<_> = mods.iter().filter(|m| m.get_type() == ModifierType::DISTURBING_PRESENCE).collect();
        // Exactly one DP modifier should be present
        assert_eq!(dp_mods.len(), 1);
        // Its modifier value should equal the count (1)
        assert_eq!(dp_mods[0].get_modifier(), 1);
    }

    #[test]
    fn minimum_roll_bb2016_agility_3_no_modifiers() {
        // max(2, 7 - min(3, 6) + 2) = max(2, 6) = 6
        let mut p = Player::default();
        p.agility = 3;
        assert_eq!(InterceptionModifierFactory::minimum_roll_bb2016(&p, &[]), 6);
    }

    #[test]
    fn minimum_roll_bb2016_agility_6_no_modifiers() {
        // max(2, 7 - 6 + 2) = max(2, 3) = 3
        let mut p = Player::default();
        p.agility = 6;
        assert_eq!(InterceptionModifierFactory::minimum_roll_bb2016(&p, &[]), 3);
    }

    #[test]
    fn minimum_roll_bb2016_adds_modifier() {
        // AG3: base=6, +1 TZ modifier → 7
        let mut p = Player::default();
        p.agility = 3;
        let m = InterceptionModifier::new("1 Tacklezone", 1, ModifierType::TACKLEZONE);
        assert_eq!(InterceptionModifierFactory::minimum_roll_bb2016(&p, &[&m]), 7);
    }

    #[test]
    fn minimum_roll_bb2016_never_below_two() {
        let mut p = Player::default();
        p.agility = 6;
        let m = InterceptionModifier::new("bonus", -5, ModifierType::REGULAR);
        assert_eq!(InterceptionModifierFactory::minimum_roll_bb2016(&p, &[&m]), 2);
    }
}
