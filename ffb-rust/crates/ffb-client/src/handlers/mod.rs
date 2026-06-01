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
