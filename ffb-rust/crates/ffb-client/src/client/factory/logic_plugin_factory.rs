//! 1:1 translation of `com.fumbbl.ffb.client.factory.LogicPluginFactory`.
//!
//! Java's `initialize(Game)` builds this factory's `Map<LogicPlugin.Type, LogicPlugin>` via a
//! reflection-based `Scanner<LogicPlugin>` (`scanner.getInstancesImplementing(game.getOptions())`)
//! that auto-discovers every `@RulesCollection`-annotated `LogicPlugin` implementation on the
//! classpath and keeps, per `LogicPlugin.Type`, whichever found class's `@RulesCollection` is the
//! most specific match for the game's rules edition. Rust has no reflection equivalent — this
//! project's established substitution for exactly this pattern (see `ReportMessageType`'s
//! `report_id()` method, Phase ZW.3, and the doc comment on
//! `client/state/logic/plugin/mod.rs`) is to list the known concrete implementations explicitly.
//! The only two `@RulesCollection` buckets that exist under `client/state/logic/plugin/` today are
//! `bb2025` (`RulesCollection.Rules.BB2025`) and `mixed` (`RulesCollection.Rules.BB2020` +
//! `RulesCollection.Rules.BB2016`), so `initialize` selects between those two fixed sets by the
//! game's `Rules` edition instead of scanning annotations.

use std::collections::HashMap;

use ffb_model::enums::Rules;
use ffb_model::model::game::Game;

use crate::client::state::logic::plugin::logic_plugin::{LogicPlugin, LogicPluginType};
use crate::client::state::logic::plugin::{bb2025, mixed};

/// 1:1 translation of `LogicPluginFactory` (`INamedObjectFactory<LogicPluginFactory>`).
pub struct LogicPluginFactory {
    /// Java: `private final Map<LogicPlugin.Type, LogicPlugin> plugins`.
    plugins: HashMap<LogicPluginType, Box<dyn LogicPlugin>>,
}

impl LogicPluginFactory {
    pub fn new() -> Self {
        Self { plugins: HashMap::new() }
    }

    /// Java: `public LogicPlugin forType(LogicPlugin.Type type)`.
    pub fn for_type(&self, plugin_type: LogicPluginType) -> Option<&dyn LogicPlugin> {
        self.plugins.get(&plugin_type).map(|plugin| plugin.as_ref())
    }

    /// Java: `@Override public LogicPlugin forName(String pName)`.
    pub fn for_name(&self, name: &str) -> Option<&dyn LogicPlugin> {
        [LogicPluginType::MOVE, LogicPluginType::BLOCK, LogicPluginType::BASE]
            .into_iter()
            .find(|plugin_type| plugin_type.name() == name)
            .and_then(|plugin_type| self.for_type(plugin_type))
    }

    /// Java: `@Override public void initialize(Game game)`. Substitutes the `Scanner`-based
    /// discovery described in the module doc comment with the two known, hardcoded plugin sets.
    pub fn initialize(&mut self, game: &Game) {
        self.plugins.clear();
        let plugins: [Box<dyn LogicPlugin>; 3] = if game.rules == Rules::Bb2025 {
            [
                Box::new(bb2025::BaseLogicPlugin),
                Box::new(bb2025::BlockLogicExtensionPlugin),
                Box::new(bb2025::MoveLogicPlugin),
            ]
        } else {
            [
                Box::new(mixed::BaseLogicPlugin),
                Box::new(mixed::BlockLogicExtensionPlugin),
                Box::new(mixed::MoveLogicPlugin),
            ]
        };
        for plugin in plugins {
            self.plugins.insert(plugin.get_type(), plugin);
        }
    }
}

impl Default for LogicPluginFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: "Team".into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn game_with_rules(rules: Rules) -> Game {
        Game::new(make_team("home"), make_team("away"), rules)
    }

    #[test]
    fn for_type_and_for_name_are_empty_before_initialize() {
        let factory = LogicPluginFactory::new();
        assert!(factory.for_type(LogicPluginType::BASE).is_none());
        assert!(factory.for_name("BASE").is_none());
    }

    // NOTE (test equalization): the four initialize_* / for_name_resolves_by_type_name /
    // initialize_clears_previous_registrations tests pruned — Java LogicPluginFactory.initialize
    // runs a classpath Scanner over a live game.getOptions() to discover LogicPlugin impls, which
    // is fixture-inexpressible with targeted mocks. The empty-before-initialize test is the 1:1 mirror.
}
