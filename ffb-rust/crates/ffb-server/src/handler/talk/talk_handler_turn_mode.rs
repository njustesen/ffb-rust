/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerTurnMode.
/// Abstract handler for `/turn_mode` command — sets or lists TurnMode values.
use ffb_model::model::game::Game;
use ffb_model::enums::TurnMode;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerTurnMode {
    pub required_client: Client,
    pub required_environment: Environment,
    pub requires_one_privilege_of: Vec<Privilege>,
}

impl TalkHandlerTurnMode {
    /// Java: `super("/turn_mode", 0, ...)`.
    pub const COMMAND: &'static str = "/turn_mode";
    pub const COMMAND_PARTS_THRESHOLD: usize = 0;

    pub fn new(
        required_client: Client,
        required_environment: Environment,
        requires_one_privilege_of: Vec<Privilege>,
    ) -> Self {
        Self { required_client, required_environment, requires_one_privilege_of }
    }

    /// Java: `handle(...)` — with no extra args, lists every `TurnMode` sorted by name;
    /// otherwise `commands[1]` sets the current mode and `commands[2]` (when there are more
    /// than 3 tokens) sets the "last" mode, clearing it otherwise.
    pub fn handle(&self, game: &mut Game, commands: &[String]) -> Vec<String> {
        if commands.len() == 1 {
            let mut response = vec!["Available TurnModes:".to_string()];
            let mut names: Vec<&'static str> = Self::all_turn_mode_names();
            names.sort();
            response.extend(names.into_iter().map(|n| format!("  - {n}")));
            return Self::with_current_modes(game, response);
        }

        if let Some(turn_mode) = commands.get(1).and_then(|s| TurnMode::from_name(s)) {
            game.turn_mode = turn_mode;
        }

        if commands.len() > 3 {
            if let Some(last) = commands.get(2).and_then(|s| TurnMode::from_name(s)) {
                game.last_turn_mode = Some(last);
            }
        } else {
            game.last_turn_mode = None;
        }

        Self::with_current_modes(game, Vec::new())
    }

    /// Java: `sendResponseWithCurrentModes` — appends the resulting mode(s) to the response.
    fn with_current_modes(game: &Game, mut response: Vec<String>) -> Vec<String> {
        response.push(format!("Set turnMode to: {}", game.turn_mode.name()));
        if let Some(last) = game.last_turn_mode {
            response.push(format!("Set lastTurnMode to: {}", last.name()));
        }
        response
    }

    /// Java: `Arrays.stream(TurnMode.values()).map(TurnMode::getName)`.
    fn all_turn_mode_names() -> Vec<&'static str> {
        use TurnMode::*;
        [
            Regular, Setup, Kickoff, PerfectDefence, SolidDefence, QuickSnap, HighKick, StartGame,
            Blitz, Touchback, Interception, EndGame, Swarming, KickoffReturn, Wizard, PassBlock,
            DumpOff, NoPlayersToField, BombHome, BombAway, BombHomeBlitz, BombAwayBlitz,
            IllegalSubstitution, SelectBlitzTarget, SelectGazeTarget, SafePairOfHands,
            SelectBlockKind, BetweenTurns, Trickster, RaidingParty, HitAndRun, ThenIStartedBlastin,
        ]
        .iter()
        .map(|m| m.name())
        .collect()
    }
}

impl Default for TalkHandlerTurnMode {
    fn default() -> Self {
        Self::new(Client::None, Environment::None, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::team::Team;
    use ffb_model::enums::Rules;

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
        Game::new(team("Home"), team("Away"), Rules::Bb2020)
    }

    #[test]
    fn construct() {
        let h = TalkHandlerTurnMode::new(Client::Spec, Environment::None, vec![Privilege::EditState]);
        assert_eq!(h.required_client, Client::Spec);
    }

    #[test]
    fn handle_lists_modes_when_no_args() {
        let h = TalkHandlerTurnMode::default();
        let mut g = game();
        let commands = vec!["/turn_mode".to_string()];
        let response = h.handle(&mut g, &commands);
        assert_eq!(response[0], "Available TurnModes:");
        assert!(response.iter().any(|s| s.contains("Set turnMode to")));
    }

    #[test]
    fn handle_sets_turn_mode() {
        let h = TalkHandlerTurnMode::default();
        let mut g = game();
        let commands = vec!["/turn_mode".to_string(), "blitz".to_string()];
        let response = h.handle(&mut g, &commands);
        assert_eq!(g.turn_mode, TurnMode::Blitz);
        assert!(response.iter().any(|s| s.contains("blitz") || s.contains("Blitz")));
    }

    #[test]
    fn handle_sets_last_turn_mode_with_enough_args() {
        let h = TalkHandlerTurnMode::default();
        let mut g = game();
        let commands = vec!["/turn_mode".to_string(), "blitz".to_string(), "regular".to_string(), "extra".to_string()];
        h.handle(&mut g, &commands);
        assert_eq!(g.last_turn_mode, Some(TurnMode::Regular));
    }

    #[test]
    fn handle_clears_last_turn_mode_without_extra_args() {
        let h = TalkHandlerTurnMode::default();
        let mut g = game();
        g.last_turn_mode = Some(TurnMode::Blitz);
        let commands = vec!["/turn_mode".to_string(), "regular".to_string()];
        h.handle(&mut g, &commands);
        assert_eq!(g.last_turn_mode, None);
    }
}
