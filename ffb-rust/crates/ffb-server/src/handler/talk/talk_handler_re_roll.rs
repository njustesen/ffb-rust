/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerReRoll.
/// Abstract handler for `/set_rerolls` command — sets reroll count for a team.
use std::collections::HashSet;
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use crate::handler::talk::command_adapter::CommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerReRoll {
    base: TalkHandler,
}

impl TalkHandlerReRoll {
    /// Java: `TalkHandlerReRoll(CommandAdapter, Client, Environment, Privilege...)`.
    pub fn new(
        command_adapter: &dyn CommandAdapter,
        required_client: Client,
        required_env: Environment,
        requires_one_privilege_of: HashSet<Privilege>,
    ) -> Self {
        let mut commands = HashSet::new();
        commands.insert("/set_rerolls".to_string());
        let commands = command_adapter.decorate_commands(commands);
        Self {
            base: TalkHandler::new(commands, 1, required_client, required_env, requires_one_privilege_of),
        }
    }

    pub fn base(&self) -> &TalkHandler { &self.base }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` —
    /// clamps `commands[1]` to `[0, 9]` and sets the reroll count of the given
    /// team's turn data. Java wraps the whole body in a swallow-all
    /// `try { ... } catch (Exception e) { // ignored }` (covers both a missing
    /// `commands[1]` and a non-numeric value); the Rust port mirrors that by
    /// simply doing nothing when the token is absent or unparsable.
    pub fn handle(&self, game: &mut Game, commands: &[String], team: &Team) {
        if let Some(raw) = commands.get(1) {
            if let Ok(parsed) = raw.parse::<i32>() {
                let new_value = parsed.max(0).min(9);
                if team.id == game.team_home.id {
                    game.turn_data_home.rerolls = new_value;
                } else {
                    game.turn_data_away.rerolls = new_value;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use crate::handler::talk::identity_command_adapter::IdentityCommandAdapter;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn handler() -> TalkHandlerReRoll {
        let adapter = IdentityCommandAdapter::new();
        TalkHandlerReRoll::new(&adapter, Client::Player, Environment::None, HashSet::new())
    }

    #[test]
    fn construct() { let _ = handler(); }

    #[test]
    fn handle_sets_home_team_rerolls() {
        let h = handler();
        let mut game = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025);
        let team = game.team_home.clone();
        let commands = vec!["/set_rerolls".to_string(), "3".to_string()];
        h.handle(&mut game, &commands, &team);
        assert_eq!(game.turn_data_home.rerolls, 3);
        assert_eq!(game.turn_data_away.rerolls, 0);
    }

    #[test]
    fn handle_sets_away_team_rerolls() {
        let h = handler();
        let mut game = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025);
        let team = game.team_away.clone();
        let commands = vec!["/set_rerolls".to_string(), "5".to_string()];
        h.handle(&mut game, &commands, &team);
        assert_eq!(game.turn_data_away.rerolls, 5);
        assert_eq!(game.turn_data_home.rerolls, 0);
    }

    #[test]
    fn handle_clamps_value_to_max_nine() {
        let h = handler();
        let mut game = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025);
        let team = game.team_home.clone();
        let commands = vec!["/set_rerolls".to_string(), "42".to_string()];
        h.handle(&mut game, &commands, &team);
        assert_eq!(game.turn_data_home.rerolls, 9);
    }

    #[test]
    fn handle_clamps_negative_value_to_zero() {
        let h = handler();
        let mut game = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025);
        let team = game.team_home.clone();
        let commands = vec!["/set_rerolls".to_string(), "-5".to_string()];
        h.handle(&mut game, &commands, &team);
        assert_eq!(game.turn_data_home.rerolls, 0);
    }

    #[test]
    fn handle_ignores_non_numeric_value() {
        let h = handler();
        let mut game = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025);
        game.turn_data_home.rerolls = 2;
        let team = game.team_home.clone();
        let commands = vec!["/set_rerolls".to_string(), "not-a-number".to_string()];
        h.handle(&mut game, &commands, &team);
        assert_eq!(game.turn_data_home.rerolls, 2);
    }
}
