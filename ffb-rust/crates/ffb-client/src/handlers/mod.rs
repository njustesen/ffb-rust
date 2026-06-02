use ffb_model::model::game::Game;
use ffb_model::events::GameEvent;
use ffb_protocol::server_commands::ServerCommand;

/// Apply a server command to the local game state and return resulting events.
pub fn handle(game: &mut Game, cmd: ServerCommand) -> Vec<GameEvent> {
    match cmd {
        ServerCommand::ServerGameState(s) => {
            *game = *s.game;
            vec![]
        }

        ServerCommand::ServerModelSync(s) => {
            log::debug!("ModelSync: {} changes (cmd {})", s.changes.len(), s.command_nr);
            vec![]
        }

        ServerCommand::ServerGameTime(t) => {
            game.half = t.half;
            vec![]
        }

        ServerCommand::ServerStatus(s) => {
            game.status = s.status;
            vec![]
        }

        ServerCommand::ServerJoin(j) => {
            log::info!("Coach {} joined (team {}, side {})", j.coach, j.team_id, j.side);
            vec![]
        }

        ServerCommand::ServerLeave(l) => {
            log::info!("Coach {} left", l.coach);
            vec![]
        }

        ServerCommand::ServerTalk(t) => {
            log::info!("[{}] {}", t.coach, t.message);
            vec![]
        }

        ServerCommand::ServerAdminMessage(m) => {
            log::info!("[Admin] {}", m.message);
            vec![]
        }

        ServerCommand::ServerSound(_) => {
            // No audio in Rust client
            vec![]
        }

        ServerCommand::ServerPong(p) => {
            log::trace!("Pong: {}", p.timestamp);
            vec![]
        }

        ServerCommand::ServerPasswordChallenge(c) => {
            log::debug!("Password challenge: {}", c.challenge);
            vec![]
        }

        ServerCommand::ServerVersion(v) => {
            log::info!("Server version: {}", v.server_version);
            vec![]
        }

        ServerCommand::ServerAddPlayer(a) => {
            log::debug!("AddPlayer: {:?}", a.player);
            vec![]
        }

        ServerCommand::ServerZapPlayer(z) => {
            log::debug!("ZapPlayer: {} -> {}", z.player_id, z.position_id);
            vec![]
        }

        ServerCommand::ServerUnzapPlayer(u) => {
            log::debug!("UnzapPlayer: {}", u.player_id);
            vec![]
        }

        ServerCommand::ServerGameList(_) | ServerCommand::ServerTeamList(_) => {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{GameStatus, Rules};
    use ffb_model::model::team::Team;
    use ffb_protocol::server_commands::*;

    fn make_game() -> Game {
        let home = Team {
            id: "home".into(), name: "home".into(), race: String::new(),
            roster_id: String::new(), coach: String::new(), rerolls: 0,
            apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, fan_factor: 0,
            assistant_coaches: 0, cheerleaders: 0, dedicated_fans: 0,
            treasury: 0, team_value: 0, players: vec![], special_rules: vec![],
        };
        let away = Team { id: "away".into(), ..home.clone() };
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn handle_server_game_state_replaces_game() {
        let mut game = make_game();
        game.half = 1;
        let new_game = make_game();
        // new_game has half=1 (default from Game::new). Set half=2 to distinguish.
        let mut new_g = make_game();
        new_g.half = 2;
        let cmd = ServerCommand::ServerGameState(ServerGameState {
            command_nr: 1,
            game: Box::new(new_g),
        });
        handle(&mut game, cmd);
        assert_eq!(game.half, 2, "ServerGameState must replace the game");
    }

    #[test]
    fn handle_server_game_time_updates_half() {
        let mut game = make_game();
        assert_eq!(game.half, 1);
        handle(&mut game, ServerCommand::ServerGameTime(ServerGameTime {
            half: 2, turn_nr: 5, seconds_left: 30,
        }));
        assert_eq!(game.half, 2, "ServerGameTime must update game.half");
    }

    #[test]
    fn handle_server_status_updates_status() {
        let mut game = make_game();
        handle(&mut game, ServerCommand::ServerStatus(ServerStatus {
            status: GameStatus::Finished,
        }));
        assert_eq!(game.status, GameStatus::Finished, "ServerStatus must update game status");
    }

    #[test]
    fn handle_informational_commands_return_empty_events() {
        let mut game = make_game();
        let events = handle(&mut game, ServerCommand::ServerPong(ServerPong { timestamp: 0 }));
        assert!(events.is_empty(), "ServerPong must return no events");
        let events2 = handle(&mut game, ServerCommand::ServerTalk(ServerTalk {
            coach: "c".into(), message: "hi".into(),
        }));
        assert!(events2.is_empty(), "ServerTalk must return no events");
    }
}
