/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerGameId.
/// Handles /gameid command — sends game ID and spectate URL to the player.
///
/// Java's `server.getCommunication().sendPlayerTalk(gameState, null, message)`
/// has no wired outbound-send equivalent yet (see `talk_handler_activated.rs`
/// for the documented adaptation) — this returns the message string instead.
pub struct TalkHandlerGameId;

const TEST_BASE_URL: &str = "https://fumbbl.com/ffbtest.jnlp?spectate=";
const LIVE_BASE_URL: &str = "https://fumbbl.com/ffblive.jnlp?spectate=";

impl TalkHandlerGameId {
    pub fn new() -> Self { Self }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` —
    /// builds the "Game Id: ... / Spectate: ..." message. `server_test_mode`
    /// is Java's `server.isInTestMode()`.
    pub fn handle(&self, game_id: u64, server_test_mode: bool) -> String {
        let base = if server_test_mode { TEST_BASE_URL } else { LIVE_BASE_URL };
        format!("Game Id: {game_id}\nSpectate: {base}{game_id}")
    }
}

impl Default for TalkHandlerGameId {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() { let _ = TalkHandlerGameId::new(); }

    #[test]
    fn handle_uses_test_url_in_test_mode() {
        let h = TalkHandlerGameId::new();
        let msg = h.handle(42, true);
        assert_eq!(msg, "Game Id: 42\nSpectate: https://fumbbl.com/ffbtest.jnlp?spectate=42");
    }

    #[test]
    fn handle_uses_live_url_outside_test_mode() {
        let h = TalkHandlerGameId::new();
        let msg = h.handle(42, false);
        assert_eq!(msg, "Game Id: 42\nSpectate: https://fumbbl.com/ffblive.jnlp?spectate=42");
    }

    #[test]
    fn handle_embeds_game_id_twice() {
        let h = TalkHandlerGameId::new();
        let msg = h.handle(999, false);
        assert_eq!(msg.matches("999").count(), 2);
    }
}
