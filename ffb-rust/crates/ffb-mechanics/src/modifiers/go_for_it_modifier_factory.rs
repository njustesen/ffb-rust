use ffb_model::enums::Rules;
use crate::modifiers::bb2025::go_for_it_modifier_collection::GoForItModifierCollection as Bb2025Collection;
use crate::modifiers::go_for_it_context::GoForItContext;
use crate::modifiers::go_for_it_modifier::GoForItModifier;
use crate::modifiers::go_for_it_modifier_collection::GoForItModifierCollection;

/// Edition-agnostic trait for a GFI modifier collection.
trait GfiCollection: Send + Sync {
    fn find_applicable<'a>(&'a self, ctx: &GoForItContext<'_>) -> Vec<&'a GoForItModifier>;
    fn get_modifiers(&self) -> &[GoForItModifier];
}

impl GfiCollection for GoForItModifierCollection {
    fn find_applicable<'a>(&'a self, ctx: &GoForItContext<'_>) -> Vec<&'a GoForItModifier> {
        self.find_applicable(ctx)
    }
    fn get_modifiers(&self) -> &[GoForItModifier] { self.get_modifiers() }
}

impl GfiCollection for Bb2025Collection {
    fn find_applicable<'a>(&'a self, ctx: &GoForItContext<'_>) -> Vec<&'a GoForItModifier> {
        self.find_applicable(ctx)
    }
    fn get_modifiers(&self) -> &[GoForItModifier] { self.get_modifiers() }
}

/// 1:1 translation of com.fumbbl.ffb.factory.common.GoForItModifierFactory.
///
/// Holds the edition-specific GoForItModifierCollection and provides modifier
/// lookup. Unlike Java's generified factory, this Rust version holds the
/// collection directly rather than via a Scanner / DI framework.
pub struct GoForItModifierFactory {
    collection: Box<dyn GfiCollection>,
}

impl GoForItModifierFactory {
    /// Construct a factory for the given rules edition.
    pub fn for_rules(rules: Rules) -> Self {
        let collection: Box<dyn GfiCollection> = match rules {
            Rules::Bb2025 | Rules::Common => Box::new(Bb2025Collection::new()),
            _ => Box::new(GoForItModifierCollection::new()),
        };
        Self { collection }
    }

    /// 1:1 translation of GoForItModifierFactory.forName.
    pub fn for_name(&self, name: &str) -> Option<&GoForItModifier> {
        self.collection.get_modifiers().iter().find(|m| m.get_name() == name)
    }

    /// Returns the modifiers applicable to the given context.
    /// 1:1 translation of GenerifiedModifierFactory.findModifiers.
    pub fn find_applicable<'a>(&'a self, context: &GoForItContext<'_>) -> Vec<&'a GoForItModifier> {
        self.collection.find_applicable(context)
    }

    /// 1:1 translation of DiceInterpreter.minimumRollGoingForIt.
    /// `max(2, 2 + sum(modifier))`
    pub fn minimum_roll_going_for_it(modifiers: &[&GoForItModifier]) -> i32 {
        let total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        2_i32.max(2 + total)
    }
}

impl Default for GoForItModifierFactory {
    fn default() -> Self {
        Self::for_rules(Rules::Bb2025)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifiers::go_for_it_context::GoForItContext;
    use ffb_model::enums::{Rules, Weather};
    use ffb_model::model::{Game, Team, Player};
    use ffb_model::enums::{PlayerType, PlayerGender};

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
        }
    }

    fn make_game(weather: Weather) -> Game {
        let mut game = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025);
        game.field_model.weather = weather;
        game
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
        }
    }

    #[test]
    fn minimum_roll_base_is_two() {
        assert_eq!(GoForItModifierFactory::minimum_roll_going_for_it(&[]), 2);
    }

    #[test]
    fn minimum_roll_adds_modifier_total() {
        let m = GoForItModifier::new("Blizzard", 1);
        assert_eq!(GoForItModifierFactory::minimum_roll_going_for_it(&[&m]), 3);
    }

    #[test]
    fn minimum_roll_never_below_two() {
        let m = GoForItModifier::new("Bonus", -5);
        assert_eq!(GoForItModifierFactory::minimum_roll_going_for_it(&[&m]), 2);
    }

    #[test]
    fn nice_weather_no_modifiers() {
        let game = make_game(Weather::Nice);
        let player = minimal_player();
        let factory = GoForItModifierFactory::for_rules(Rules::Bb2025);
        let ctx = GoForItContext::new(&game, &player);
        let mods = factory.find_applicable(&ctx);
        assert!(mods.is_empty(), "Nice weather should yield no GFI modifiers");
    }

    #[test]
    fn blizzard_weather_adds_one_modifier() {
        let game = make_game(Weather::Blizzard);
        let player = minimal_player();
        let factory = GoForItModifierFactory::for_rules(Rules::Bb2025);
        let ctx = GoForItContext::new(&game, &player);
        let mods = factory.find_applicable(&ctx);
        assert_eq!(mods.len(), 1, "Blizzard should add one GFI modifier");
        assert_eq!(GoForItModifierFactory::minimum_roll_going_for_it(&mods), 3);
    }

    #[test]
    fn for_name_returns_blizzard_modifier() {
        let factory = GoForItModifierFactory::for_rules(Rules::Bb2025);
        assert!(factory.for_name("Blizzard").is_some());
        assert!(factory.for_name("NonExistent").is_none());
    }
}
