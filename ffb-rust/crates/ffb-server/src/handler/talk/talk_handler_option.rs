/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerOption.
/// Handles /option command — sets a single game option by ID and value.
///
/// Java builds a typed `IGameOption` via `GameOptionFactory.createGameOption(GameOptionId)`
/// (int/bool/string wrapper) before storing it in `game.getOptions()`. The Rust
/// `GameOptionFactory` (`ffb_model::factory::game_option_factory`) is still an
/// unimplemented stub with no `create_game_option`, and the Rust `GameOptions`
/// model (`ffb_model::model::game_options::GameOptions`) stores raw strings
/// directly rather than typed option objects — so this port stores the value
/// string directly under the option id instead of constructing an intermediate
/// typed option. `GameOptionIdFactory::for_name` is still used for parity with
/// Java's null-check (it treats any non-empty name as a valid id, matching the
/// current Rust factory's behavior). Java's trailing
/// `UtilServerGame.syncGameModel(...)` (only invoked once `game.getStarted() != null`)
/// has no wired Rust target yet (see `talk_handler_activated.rs`), and `Game`
/// has no `started` timestamp field yet either — both are the caller's
/// responsibility once that infra exists.
use std::collections::HashSet;
use ffb_model::factory::game_option_id_factory::GameOptionIdFactory;
use ffb_model::model::game::Game;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_requirements::{Client, Environment};

pub struct TalkHandlerOption {
    base: TalkHandler,
}

impl TalkHandlerOption {
    /// Java: `TalkHandlerOption()` — `super("/option", 1, Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> Self {
        let mut commands = HashSet::new();
        commands.insert("/option".to_string());
        Self {
            base: TalkHandler::new(commands, 1, Client::Player, Environment::TestGame, HashSet::new()),
        }
    }

    pub fn base(&self) -> &TalkHandler { &self.base }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` — sets
    /// the game option named by `commands[1]` to `commands[2]`. Returns the info message
    /// Java would have sent via `communication.sendPlayerTalk`, or `None` if the option id
    /// or the resulting option object couldn't be resolved (mirrors the Java early returns).
    pub fn handle(&self, game: &mut Game, commands: &[String]) -> Option<String> {
        let option_name = commands.get(1)?;
        let option_id = GameOptionIdFactory::default().for_name(option_name)?;
        let value = commands.get(2)?;
        game.options.set(option_id.0.clone(), value.clone());
        Some(format!("Setting game option {} to value {}.", option_id.0, value))
    }
}

impl Default for TalkHandlerOption {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn team(name: &str) -> Team {
        Team {
            id: name.into(), name: name.into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn game() -> Game {
        Game::new(team("Home"), team("Away"), Rules::Bb2025)
    }

    #[test]
    fn construct() { let _ = TalkHandlerOption::new(); }

    #[test]
    fn handle_sets_option_value() {
        let h = TalkHandlerOption::new();
        let mut g = game();
        let commands = vec!["/option".to_string(), "maxPlayersOnField".to_string(), "11".to_string()];
        let info = h.handle(&mut g, &commands).unwrap();
        assert_eq!(g.options.get("maxPlayersOnField"), Some("11"));
        assert!(info.contains("maxPlayersOnField"));
        assert!(info.contains("11"));
    }

    #[test]
    fn handle_returns_none_when_option_missing() {
        let h = TalkHandlerOption::new();
        let mut g = game();
        let commands = vec!["/option".to_string(), "maxPlayersOnField".to_string()];
        assert!(h.handle(&mut g, &commands).is_none());
        assert!(g.options.get("maxPlayersOnField").is_none());
    }

    #[test]
    fn handle_returns_none_when_option_id_empty() {
        let h = TalkHandlerOption::new();
        let mut g = game();
        let commands = vec!["/option".to_string(), "".to_string(), "11".to_string()];
        assert!(h.handle(&mut g, &commands).is_none());
    }

    #[test]
    fn handle_overwrites_existing_value() {
        let h = TalkHandlerOption::new();
        let mut g = game();
        g.options.set("turntime", "60");
        let commands = vec!["/option".to_string(), "turntime".to_string(), "90".to_string()];
        h.handle(&mut g, &commands).unwrap();
        assert_eq!(g.options.get("turntime"), Some("90"));
    }
}
