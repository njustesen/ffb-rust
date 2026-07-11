/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerTurnLive.
/// Handles `/turn` command — sets turn number for a team (EDIT_STATE privilege, live only).
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerTurnLive {
    pub required_client: Client,
    pub required_environment: Environment,
    pub requires_one_privilege_of: Vec<Privilege>,
}

impl TalkHandlerTurnLive {
    /// Java: `super("/turn", 1, new DecoratingCommandAdapter(), Client.SPEC, Environment.NONE, Privilege.EDIT_STATE)`.
    pub const COMMAND: &'static str = "/turn";
    pub const COMMAND_PARTS_THRESHOLD: usize = 1;

    pub fn new() -> Self {
        Self {
            required_client: Client::Spec,
            required_environment: Environment::None,
            requires_one_privilege_of: vec![Privilege::EditState],
        }
    }

    /// Java: `handle(...)` — `commands[1]` is the new (non-negative) turn number for
    /// whichever team the command was routed to.
    pub fn handle(&self, game: &mut Game, team: &Team, commands: &[String]) -> Option<String> {
        let new_turn_nr: i32 = commands.get(1)?.parse().ok()?;
        if new_turn_nr < 0 {
            return None;
        }
        if team.id == game.team_home.id {
            game.turn_data_home.turn_nr = new_turn_nr;
        } else {
            game.turn_data_away.turn_nr = new_turn_nr;
        }
        Some(format!("Jumping to turn {new_turn_nr} for {}.", team.name))
    }
}

impl Default for TalkHandlerTurnLive {
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
        Game::new(team("home", "Home"), team("away", "Away"), Rules::Bb2020)
    }

    #[test]
    fn construct() {
        let h = TalkHandlerTurnLive::new();
        assert_eq!(h.required_client, Client::Spec);
        assert_eq!(h.requires_one_privilege_of, vec![Privilege::EditState]);
    }

    #[test]
    fn handle_sets_home_turn_number() {
        let h = TalkHandlerTurnLive::new();
        let mut g = game();
        let home = g.team_home.clone();
        let commands = vec!["/turn".into(), "5".into()];
        let info = h.handle(&mut g, &home, &commands).unwrap();
        assert_eq!(g.turn_data_home.turn_nr, 5);
        assert!(info.contains("Home"));
    }

    #[test]
    fn handle_sets_away_turn_number() {
        let h = TalkHandlerTurnLive::new();
        let mut g = game();
        let away = g.team_away.clone();
        let commands = vec!["/turn".into(), "3".into()];
        h.handle(&mut g, &away, &commands).unwrap();
        assert_eq!(g.turn_data_away.turn_nr, 3);
    }

    #[test]
    fn handle_rejects_negative_turn() {
        let h = TalkHandlerTurnLive::new();
        let mut g = game();
        let home = g.team_home.clone();
        let commands = vec!["/turn".into(), "-1".into()];
        assert!(h.handle(&mut g, &home, &commands).is_none());
    }
}
