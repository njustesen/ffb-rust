/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerTurnTest.
/// Handles `/turn` command in test games — syncs both teams' turn numbers together.
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use super::talk_requirements::{Client, Environment};

pub struct TalkHandlerTurnTest {
    pub required_client: Client,
    pub required_environment: Environment,
}

impl TalkHandlerTurnTest {
    /// Java: `super("/turn", 1, new IdentityCommandAdapter(), Client.PLAYER, Environment.TEST_GAME)`.
    pub const COMMAND: &'static str = "/turn";
    pub const COMMAND_PARTS_THRESHOLD: usize = 1;

    pub fn new() -> Self {
        Self { required_client: Client::Player, required_environment: Environment::TestGame }
    }

    /// Java: `handle(...)` — computes the delta between the routed team's current turn
    /// number and the requested one, then applies that same delta to *both* teams so they
    /// stay in sync during test games.
    pub fn handle(&self, game: &mut Game, team: &Team, commands: &[String]) -> Option<String> {
        let new_turn_nr: i32 = commands.get(1)?.parse().ok()?;
        if new_turn_nr < 0 {
            return None;
        }
        let turn_diff = if team.id == game.team_home.id {
            new_turn_nr - game.turn_data_home.turn_nr
        } else {
            new_turn_nr - game.turn_data_away.turn_nr
        };
        game.turn_data_home.turn_nr += turn_diff;
        game.turn_data_away.turn_nr += turn_diff;
        Some(format!("Jumping to turn {new_turn_nr}."))
    }
}

impl Default for TalkHandlerTurnTest {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    fn team(id: &str, name: &str) -> Team {
        Team {
            id: id.into(), name: name.into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn game() -> Game {
        let mut g = Game::new(team("home", "Home"), team("away", "Away"), Rules::Bb2020);
        g.turn_data_home.turn_nr = 2;
        g.turn_data_away.turn_nr = 2;
        g
    }

    #[test]
    fn construct() {
        let h = TalkHandlerTurnTest::new();
        assert_eq!(h.required_client, Client::Player);
        assert_eq!(h.required_environment, Environment::TestGame);
    }

    #[test]
    fn handle_syncs_both_teams_by_delta() {
        let h = TalkHandlerTurnTest::new();
        let mut g = game();
        let home = g.team_home.clone();
        let commands = vec!["/turn".into(), "5".into()];
        let info = h.handle(&mut g, &home, &commands).unwrap();
        assert_eq!(g.turn_data_home.turn_nr, 5);
        assert_eq!(g.turn_data_away.turn_nr, 5);
        assert!(info.contains("5"));
    }

    #[test]
    fn handle_rejects_negative_turn() {
        let h = TalkHandlerTurnTest::new();
        let mut g = game();
        let home = g.team_home.clone();
        let commands = vec!["/turn".into(), "-1".into()];
        assert!(h.handle(&mut g, &home, &commands).is_none());
    }
}
