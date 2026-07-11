/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerGames.
/// Handles /games command — lists active games (DEV privilege, TEST_SERVER env).
///
/// Java builds the listing from
/// `server.getGameCache().findActiveGames().getEntriesSorted()` — the Rust
/// `GameCache` (ffb-server, in-memory MVP) has no `findActiveGames`/game-entry
/// listing yet. Following the same adaptation as
/// `TalkHandler::play_sound_after_cooldown` (take the missing state as an
/// explicit parameter), this handler takes the already-sorted
/// `(home_coach, away_coach)` pairs directly.
///
/// Java's `server.getCommunication().sendTalk(session, null, response)` has no
/// wired outbound-send equivalent yet (see `talk_handler_activated.rs`) — this
/// returns the message lines instead.
pub struct TalkHandlerGames;

impl TalkHandlerGames {
    pub fn new() -> Self { Self }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` —
    /// formats each active game entry as "{home coach} vs {away coach}".
    pub fn handle(&self, active_games: &[(String, String)]) -> Vec<String> {
        active_games
            .iter()
            .map(|(home_coach, away_coach)| format!("{home_coach} vs {away_coach}"))
            .collect()
    }
}

impl Default for TalkHandlerGames {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() { let _ = TalkHandlerGames::new(); }

    #[test]
    fn handle_returns_empty_when_no_active_games() {
        let h = TalkHandlerGames::new();
        let info = h.handle(&[]);
        assert!(info.is_empty());
    }

    #[test]
    fn handle_formats_single_game() {
        let h = TalkHandlerGames::new();
        let games = vec![("Alice".to_string(), "Bob".to_string())];
        let info = h.handle(&games);
        assert_eq!(info, vec!["Alice vs Bob".to_string()]);
    }

    #[test]
    fn handle_formats_multiple_games_preserving_order() {
        let h = TalkHandlerGames::new();
        let games = vec![
            ("Alice".to_string(), "Bob".to_string()),
            ("Carl".to_string(), "Dana".to_string()),
        ];
        let info = h.handle(&games);
        assert_eq!(info, vec!["Alice vs Bob".to_string(), "Carl vs Dana".to_string()]);
    }
}
