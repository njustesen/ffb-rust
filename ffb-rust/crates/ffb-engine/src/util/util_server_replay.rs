use crate::game_state::GameState;
use crate::server_replay::ServerReplay;

/// Replay utility: construct and send replay sequences — 1:1 translation of Java UtilServerReplay.
pub struct UtilServerReplay;

impl UtilServerReplay {
    pub fn send_replay(game_state: &GameState, from_command_nr: i32, to_command_nr: i32, session_id: &str) {
        // Phase ZU: queue a ServerReplay to ServerReplayer over WebSocket
        todo!("Phase ZU: WebSocket replay dispatch")
    }

    pub fn send_game_log(game_state: &GameState, session_id: &str) {
        // Phase ZU: send the full game log to a newly-joined spectator
        todo!("Phase ZU: full game log send")
    }

    pub fn build_replay(game_state: &GameState, to_command_nr: i32, session_id: &str) -> ServerReplay {
        ServerReplay::new(0, to_command_nr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_replay_returns_correct_to_command_nr() {
        let gs = GameState::new();
        let replay = UtilServerReplay::build_replay(&gs, 50, "session1");
        assert_eq!(replay.get_to_command_nr(), 50);
    }

    #[test]
    fn test_build_replay_not_complete() {
        let gs = GameState::new();
        let replay = UtilServerReplay::build_replay(&gs, 10, "s");
        assert!(!replay.is_complete());
    }
}
