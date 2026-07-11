/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerOptions.
/// Handles /options command — lists all game options sorted by name.
///
/// Java iterates every `GameOptionId` enum constant, resolves each one's current
/// (or default) value via `game.getOptions().getOptionWithDefault(optionId)`, sorts
/// by name, then sends one "Option {name} = {value}" line per option.
use std::collections::HashSet;
use ffb_model::model::game::Game;
use ffb_model::option::game_option_id::GameOptionId;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_requirements::{Client, Environment};

pub struct TalkHandlerOptions {
    base: TalkHandler,
}

impl TalkHandlerOptions {
    /// Java: `TalkHandlerOptions()` — `super("/options", 0, Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> Self {
        let mut commands = HashSet::new();
        commands.insert("/options".to_string());
        Self {
            base: TalkHandler::new(commands, 0, Client::Player, Environment::TestGame, HashSet::new()),
        }
    }

    pub fn base(&self) -> &TalkHandler { &self.base }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` — lists
    /// every game option (with defaults for unset ones), sorted by name, as
    /// "Option {name} = {value}" info lines Java would have sent via
    /// `communication.sendPlayerTalk`.
    pub fn handle(&self, game: &Game) -> Vec<String> {
        let mut option_list: Vec<Box<dyn ffb_model::option::i_game_option::IGameOption>> =
            GameOptionId::values()
                .iter()
                .map(|id| game.options.get_option_with_default(*id))
                .collect();
        option_list.sort_by(|a, b| a.get_id().cmp(b.get_id()));
        option_list
            .iter()
            .map(|option| format!("Option {} = {}", option.get_id(), option.get_value_as_string()))
            .collect()
    }
}

impl Default for TalkHandlerOptions {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn construct() { let _ = TalkHandlerOptions::new(); }

    #[test]
    fn base_has_expected_command_and_threshold() {
        let h = TalkHandlerOptions::new();
        assert_eq!(h.base().command_parts_threshold, 0);
    }

    #[test]
    fn handle_lists_every_option_sorted_by_name() {
        let h = TalkHandlerOptions::new();
        let game = make_game();
        let lines = h.handle(&game);
        assert_eq!(lines.len(), GameOptionId::values().len());
        let mut sorted = lines.clone();
        sorted.sort();
        assert_eq!(lines, sorted);
        assert!(lines.iter().all(|l| l.starts_with("Option ")));
    }

    #[test]
    fn handle_uses_default_for_unset_option() {
        let h = TalkHandlerOptions::new();
        let game = make_game();
        let lines = h.handle(&game);
        assert!(lines.contains(&"Option turntime = 240".to_string()));
    }

    #[test]
    fn handle_uses_stored_value_when_set() {
        let h = TalkHandlerOptions::new();
        let mut game = make_game();
        game.options.set("turntime", "120");
        let lines = h.handle(&game);
        assert!(lines.contains(&"Option turntime = 120".to_string()));
    }
}
