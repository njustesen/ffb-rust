use ffb_model::enums::{Rules, TurnMode};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_disturbing_presence::UtilDisturbingPresence;
use ffb_model::util::util_player::UtilPlayer;
use crate::modifiers::bb2016::pass_modifier_collection::PassModifierCollection as Bb2016Collection;
use crate::modifiers::mixed::pass_modifier_collection::PassModifierCollection as MixedCollection;
use crate::modifiers::modifier_type::ModifierType;
use crate::modifiers::pass_context::PassContext;
use crate::modifiers::pass_modifier::PassModifier;
use crate::modifiers::pass_modifier_collection::PassModifierCollection;

/// Edition-agnostic trait for a pass modifier collection.
trait PassCollection: Send + Sync {
    fn find_applicable<'a>(&'a self, ctx: &PassContext<'_>) -> Vec<&'a PassModifier>;
    fn get_modifiers(&self) -> &[PassModifier];
}

impl PassCollection for PassModifierCollection {
    fn find_applicable<'a>(&'a self, ctx: &PassContext<'_>) -> Vec<&'a PassModifier> {
        self.find_applicable(ctx)
    }
    fn get_modifiers(&self) -> &[PassModifier] { self.get_modifiers() }
}

impl PassCollection for Bb2016Collection {
    fn find_applicable<'a>(&'a self, ctx: &PassContext<'_>) -> Vec<&'a PassModifier> {
        self.find_applicable(ctx)
    }
    fn get_modifiers(&self) -> &[PassModifier] { self.get_modifiers() }
}

impl PassCollection for MixedCollection {
    fn find_applicable<'a>(&'a self, ctx: &PassContext<'_>) -> Vec<&'a PassModifier> {
        self.find_applicable(ctx)
    }
    fn get_modifiers(&self) -> &[PassModifier] { self.get_modifiers() }
}

/// 1:1 translation of com.fumbbl.ffb.factory.PassModifierFactory.
///
/// Holds the edition-specific PassModifierCollection and provides modifier
/// lookup. Unlike Java's generified factory, this Rust version holds the
/// collection directly rather than via a Scanner / DI framework.
pub struct PassModifierFactory {
    collection: Box<dyn PassCollection>,
}

impl PassModifierFactory {
    pub fn for_rules(rules: Rules) -> Self {
        let collection: Box<dyn PassCollection> = match rules {
            Rules::Bb2016 => Box::new(Bb2016Collection::new()),
            Rules::Bb2020 | Rules::Bb2025 | Rules::Common => Box::new(MixedCollection::new()),
        };
        Self { collection }
    }

    /// 1:1 translation of PassModifierFactory.forName.
    pub fn for_name(&self, name: &str) -> Option<&PassModifier> {
        self.collection.get_modifiers().iter().find(|m| m.get_name() == name)
    }

    /// 1:1 translation of GenerifiedModifierFactory.findModifiers for PassModifierFactory.
    ///
    /// Collects: REGULAR (predicate-filtered), TACKLEZONE (count-based),
    /// DISTURBING_PRESENCE (count-based). Skill-based modifiers are deferred
    /// until the Skill trait exposes pass_modifiers().
    pub fn find_modifiers<'a>(&'a self, context: &PassContext<'_>) -> Vec<&'a PassModifier> {
        let mut result: Vec<&'a PassModifier> = self.collection.get_modifiers().iter()
            .filter(|m| m.get_type() == ModifierType::REGULAR && m.applies_to_context(context))
            .collect();

        // isAffectedByTackleZones: ttm OR player doesn't have ignoreTacklezonesWhenPassing
        let affected_by_tz = context.is_ttm()
            || !context.player.has_skill_property(NamedProperties::IGNORE_TACKLEZONES_WHEN_PASSING);

        if affected_by_tz {
            let count = Self::pass_modifiers(context) as i32;
            if count > 0 {
                if let Some(tz_mod) = self.collection.get_modifiers().iter()
                    .find(|m| m.get_type() == ModifierType::TACKLEZONE && m.get_modifier() == count)
                {
                    result.push(tz_mod);
                }
            }
        }

        // isAffectedByDisturbingPresence: always true for pass
        let dp_count = UtilDisturbingPresence::find_opposing_disturbing_presences(
            context.game, &context.player.id);
        if dp_count > 0 {
            if let Some(dp_mod) = self.collection.get_modifiers().iter()
                .find(|m| m.get_type() == ModifierType::DISTURBING_PRESENCE && m.get_modifier() == dp_count)
            {
                result.push(dp_mod);
            }
        }

        result
    }

    /// 1:1 translation of PassMechanic.passModifiers (BB2025/BB2020 logic).
    ///
    /// Count = adjacent tacklezone players. In DUMP_OFF mode, subtract one if the
    /// acting player is adjacent and standing up.
    fn pass_modifiers(context: &PassContext<'_>) -> usize {
        let game = context.game;
        let player_id = &context.player.id;
        let players = UtilPlayer::find_tacklezone_players(game, player_id);
        let mut zones = players.len();

        if game.turn_mode == TurnMode::DumpOff {
            let ap = &game.acting_player;
            if let Some(ap_id) = ap.player_id.as_deref() {
                if ap.standing_up && players.iter().any(|id| id.as_str() == ap_id) {
                    zones = zones.saturating_sub(1);
                }
            }
        }

        zones
    }

    /// Compute the pass minimum roll from the thrower and applicable modifiers.
    /// 1:1 translation of PassMechanic.minimumRoll (BB2025): max(2, passing + distance + sum(modifiers)).
    pub fn minimum_roll(passing: i32, distance_modifier: i32, modifiers: &[&PassModifier]) -> Option<i32> {
        if passing <= 0 {
            return None; // Java returns Optional.empty() when no passing ability
        }
        let total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        Some((passing + distance_modifier + total).max(2))
    }
}

impl Default for PassModifierFactory {
    fn default() -> Self {
        Self::for_rules(Rules::Bb2025)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, Weather, PS_STANDING, PlayerState, PlayerType, PlayerGender, PassingDistance};
    use ffb_model::model::{Game, Player, Team};
    use crate::modifiers::pass_context::PassContext;

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

    fn make_game(rules: Rules, weather: Weather) -> Game {
        let mut g = Game::new(empty_team("home"), empty_team("away"), rules);
        g.field_model.weather = weather;
        g
    }

    fn minimal_player(id: &str, passing: i32) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    #[test]
    fn nice_weather_no_opponents_no_modifiers() {
        let mut g = make_game(Rules::Bb2025, Weather::Nice);
        g.team_home.players.push(minimal_player("h1", 4));
        g.field_model.set_player_coordinate("h1", ffb_model::types::FieldCoordinate::new(5, 5));
        g.field_model.set_player_state("h1", PlayerState(PS_STANDING));
        let p = g.team_home.players.first().unwrap();
        let factory = PassModifierFactory::for_rules(Rules::Bb2025);
        let ctx = PassContext::new(&g, p, PassingDistance::ShortPass, false);
        let mods = factory.find_modifiers(&ctx);
        assert!(!mods.iter().any(|m| m.get_name() == "Very Sunny"),
            "Nice weather should not trigger Very Sunny modifier");
        assert!(!mods.iter().any(|m| m.get_type() == ModifierType::TACKLEZONE),
            "No opponents means no tacklezone modifiers");
    }

    #[test]
    fn very_sunny_adds_regular_modifier() {
        let mut g = make_game(Rules::Bb2025, Weather::VerySunny);
        g.team_home.players.push(minimal_player("h1", 4));
        g.field_model.set_player_coordinate("h1", ffb_model::types::FieldCoordinate::new(5, 5));
        g.field_model.set_player_state("h1", PlayerState(PS_STANDING));
        let p = g.team_home.players.first().unwrap();
        let factory = PassModifierFactory::for_rules(Rules::Bb2025);
        let ctx = PassContext::new(&g, p, PassingDistance::ShortPass, false);
        let mods = factory.find_modifiers(&ctx);
        assert!(mods.iter().any(|m| m.get_name() == "Very Sunny"),
            "Very Sunny weather should add Very Sunny pass modifier");
    }

    #[test]
    fn minimum_roll_adds_passing_and_distance() {
        // passing=4, distance=0, no modifiers → max(2, 4) = 4
        assert_eq!(PassModifierFactory::minimum_roll(4, 0, &[]), Some(4));
    }

    #[test]
    fn minimum_roll_never_below_two() {
        assert_eq!(PassModifierFactory::minimum_roll(1, 0, &[]), Some(2));
    }

    #[test]
    fn minimum_roll_no_passing_ability_returns_none() {
        assert_eq!(PassModifierFactory::minimum_roll(0, 0, &[]), None);
    }

    #[test]
    fn for_name_returns_existing_modifier() {
        let factory = PassModifierFactory::for_rules(Rules::Bb2025);
        assert!(factory.for_name("Very Sunny").is_some());
        assert!(factory.for_name("1 Tacklezone").is_some());
        assert!(factory.for_name("NonExistent").is_none());
    }

    #[test]
    fn bb2016_has_blizzard_modifier() {
        let factory = PassModifierFactory::for_rules(Rules::Bb2016);
        assert!(factory.for_name("Blizzard").is_some(), "BB2016 should have Blizzard modifier");
    }
}
