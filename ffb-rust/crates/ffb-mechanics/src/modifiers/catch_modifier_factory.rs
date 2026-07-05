use ffb_model::enums::Rules;
use ffb_model::model::Player;
use crate::modifiers::bb2025::catch_modifier_collection::CatchModifierCollection as Bb2025Collection;
use crate::modifiers::catch_context::CatchContext;
use crate::modifiers::catch_modifier::CatchModifier;
use crate::modifiers::catch_modifier_collection::CatchModifierCollection;

/// Edition-agnostic trait for a catch modifier collection.
trait CatchCollection: Send + Sync {
    fn find_applicable<'a>(&'a self, ctx: &CatchContext<'_>) -> Vec<&'a CatchModifier>;
    fn get_modifiers(&self) -> &[CatchModifier];
}

impl CatchCollection for CatchModifierCollection {
    fn find_applicable<'a>(&'a self, ctx: &CatchContext<'_>) -> Vec<&'a CatchModifier> {
        self.find_applicable(ctx)
    }
    fn get_modifiers(&self) -> &[CatchModifier] { self.get_modifiers() }
}

impl CatchCollection for Bb2025Collection {
    fn find_applicable<'a>(&'a self, ctx: &CatchContext<'_>) -> Vec<&'a CatchModifier> {
        self.find_applicable(ctx)
    }
    fn get_modifiers(&self) -> &[CatchModifier] { self.get_modifiers() }
}

/// 1:1 translation of com.fumbbl.ffb.factory.CatchModifierFactory.
///
/// Holds the edition-specific CatchModifierCollection and provides modifier
/// lookup. Unlike Java's generified factory, this Rust version holds the
/// collection directly rather than via a Scanner / DI framework.
pub struct CatchModifierFactory {
    collection: Box<dyn CatchCollection>,
}

impl CatchModifierFactory {
    /// Construct a factory for the given rules edition.
    pub fn for_rules(rules: Rules) -> Self {
        let collection: Box<dyn CatchCollection> = match rules {
            Rules::Bb2025 | Rules::Common => Box::new(Bb2025Collection::new()),
            _ => Box::new(CatchModifierCollection::new()),
        };
        Self { collection }
    }

    /// Returns the modifiers applicable to the given context.
    /// 1:1 translation of GenerifiedModifierFactory.findModifiers.
    pub fn find_applicable<'a>(&'a self, context: &CatchContext<'_>) -> Vec<&'a CatchModifier> {
        self.collection.find_applicable(context)
    }

    /// Compute the catch minimum roll from the catcher and applicable modifiers.
    /// 1:1 translation of AgilityMechanic.minimumRollCatch (BB2025: max(2, agility + sum)).
    pub fn minimum_roll_catch(player: &Player, modifiers: &[&CatchModifier]) -> i32 {
        let total: i32 = modifiers.iter().map(|m| m.get_modifier()).sum();
        2_i32.max(player.agility_with_modifiers() + total)
    }
}

impl Default for CatchModifierFactory {
    fn default() -> Self {
        Self::for_rules(Rules::Bb2025)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifiers::catch_context::CatchContext;
    use ffb_model::enums::{Rules, Weather};
    use ffb_model::model::{Game, Team, Player};
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::model::CatchScatterThrowInMode;

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

    fn minimal_player(agility: i32) -> Player {
        Player {
            id: "p1".into(), name: "Joe".into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        }
    }

    #[test]
    fn nice_weather_no_modifiers() {
        let game = make_game(Weather::Nice);
        let player = minimal_player(3);
        let factory = CatchModifierFactory::for_rules(Rules::Bb2025);
        let ctx = CatchContext::new(&game, Some(&player), CatchScatterThrowInMode::CatchScatter, None);
        let mods = factory.find_applicable(&ctx);
        // Tacklezone and disturbing presence modifiers need game state; without any on-pitch
        // opponents the only potential trigger is weather. Nice → no weather modifier.
        // Inaccurate-pass modifier fires for CATCH_SCATTER — that is expected.
        // Only check that the pouring-rain modifier is absent.
        assert!(!mods.iter().any(|m| m.get_name() == "Pouring Rain"));
    }

    #[test]
    fn pouring_rain_adds_modifier() {
        let game = make_game(Weather::PouringRain);
        let player = minimal_player(3);
        let factory = CatchModifierFactory::for_rules(Rules::Bb2025);
        let ctx = CatchContext::new(&game, Some(&player), CatchScatterThrowInMode::CatchHandOff, None);
        let mods = factory.find_applicable(&ctx);
        assert!(mods.iter().any(|m| m.get_name() == "Pouring Rain"),
            "Pouring rain modifier should be present");
    }

    #[test]
    fn minimum_roll_catch_base() {
        let player = minimal_player(3);
        let min = CatchModifierFactory::minimum_roll_catch(&player, &[]);
        assert_eq!(min, 3);
    }

    #[test]
    fn minimum_roll_catch_never_below_two() {
        let player = minimal_player(1);
        let modifier = crate::modifiers::catch_modifier::CatchModifier::new(
            "Bonus", -5,
            crate::modifiers::modifier_type::ModifierType::REGULAR,
        );
        let min = CatchModifierFactory::minimum_roll_catch(&player, &[&modifier]);
        assert_eq!(min, 2);
    }
}
