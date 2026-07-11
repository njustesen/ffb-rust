/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerPlayingLive.
/// Handles /playing_home and /playing_away commands — sets the currently playing team.
use std::collections::HashSet;
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use crate::handler::talk::command_adapter::CommandAdapter;
use crate::handler::talk::decorating_command_adapter::DecoratingCommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerPlayingLive {
    base: TalkHandler,
}

impl TalkHandlerPlayingLive {
    /// Java: `TalkHandlerPlayingLive()` — `super("/playing", 0, new DecoratingCommandAdapter(),
    /// Client.SPEC, Environment.NONE, Privilege.EDIT_STATE)`.
    pub fn new() -> Self {
        let adapter = DecoratingCommandAdapter::new();
        let mut commands = HashSet::new();
        commands.insert("/playing".to_string());
        let commands = adapter.decorate_commands(commands);
        let mut privileges = HashSet::new();
        privileges.insert(Privilege::EditState);
        Self {
            base: TalkHandler::new(commands, 0, Client::Spec, Environment::None, privileges),
        }
    }

    pub fn base(&self) -> &TalkHandler { &self.base }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` — sets
    /// `game.homePlaying` to whether `team` is the home team. Returns the info message
    /// Java would have sent via `communication.sendPlayerTalk`.
    pub fn handle(&self, game: &mut Game, team: &Team) -> String {
        game.home_playing = team.id == game.team_home.id;
        format!("Set playing team to {}.", team.name)
    }
}

impl Default for TalkHandlerPlayingLive {
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
        Game::new(team("home", "Home"), team("away", "Away"), Rules::Bb2025)
    }

    #[test]
    fn construct() { let _ = TalkHandlerPlayingLive::new(); }

    #[test]
    fn handle_sets_home_playing_true_for_home_team() {
        let h = TalkHandlerPlayingLive::new();
        let mut g = game();
        g.home_playing = false;
        let home = team("home", "Home");
        let info = h.handle(&mut g, &home);
        assert!(g.home_playing);
        assert_eq!(info, "Set playing team to Home.");
    }

    #[test]
    fn handle_sets_home_playing_false_for_away_team() {
        let h = TalkHandlerPlayingLive::new();
        let mut g = game();
        g.home_playing = true;
        let away = team("away", "Away");
        let info = h.handle(&mut g, &away);
        assert!(!g.home_playing);
        assert_eq!(info, "Set playing team to Away.");
    }
}
