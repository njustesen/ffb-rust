/// 1:1 translation of com.fumbbl.ffb.server.net.SessionManager.
use std::collections::{HashMap, HashSet};
use ffb_model::model::ClientMode;
use tokio::sync::mpsc;
use crate::model::joined_client::JoinedClient;
use crate::model::received_command::SessionId;

/// Manages the mapping between WebSocket sessions and game state.
///
/// Java: `SessionManager`
pub struct SessionManager {
    /// Java: `fClientBySession`
    client_by_session: HashMap<SessionId, JoinedClient>,
    /// Java: `fSessionsByGameId`
    sessions_by_game_id: HashMap<i64, HashSet<SessionId>>,
    /// Java: `fLastPingBySession`
    last_ping_by_session: HashMap<SessionId, i64>,
    /// Sender half of each session's output channel (no Java equivalent — replaces Jetty Session.getRemote())
    senders: HashMap<SessionId, mpsc::UnboundedSender<String>>,
}

impl SessionManager {
    /// Java: `new SessionManager()`
    pub fn new() -> Self {
        Self {
            client_by_session: HashMap::new(),
            sessions_by_game_id: HashMap::new(),
            last_ping_by_session: HashMap::new(),
            senders: HashMap::new(),
        }
    }

    /// Java: `addSession(Session, gameId, coach, mode, homeCoach, properties)`
    pub fn add_session(
        &mut self,
        session_id: SessionId,
        game_id: i64,
        coach: String,
        mode: ClientMode,
        home_coach: bool,
        account_properties: Vec<String>,
        sender: mpsc::UnboundedSender<String>,
    ) {
        let client = JoinedClient::new(game_id, coach, mode, home_coach, account_properties);
        self.client_by_session.insert(session_id, client);
        self.sessions_by_game_id.entry(game_id).or_default().insert(session_id);
        self.last_ping_by_session.insert(session_id, 0);
        self.senders.insert(session_id, sender);
    }

    /// Java: `removeSession(Session)`
    pub fn remove_session(&mut self, session_id: SessionId) {
        let game_id = self.get_game_id_for_session(session_id);
        self.client_by_session.remove(&session_id);
        self.last_ping_by_session.remove(&session_id);
        self.senders.remove(&session_id);
        if let Some(sessions) = self.sessions_by_game_id.get_mut(&game_id) {
            sessions.remove(&session_id);
            if sessions.is_empty() {
                self.sessions_by_game_id.remove(&game_id);
            }
        }
    }

    /// Java: `getGameIdForSession(Session)`
    pub fn get_game_id_for_session(&self, session_id: SessionId) -> i64 {
        self.client_by_session.get(&session_id).map(|c| c.get_game_id()).unwrap_or(0)
    }

    /// Java: `getCoachForSession(Session)`
    pub fn get_coach_for_session(&self, session_id: SessionId) -> Option<&str> {
        self.client_by_session.get(&session_id).map(|c| c.get_coach())
    }

    /// Java: `getModeForSession(Session)`
    pub fn get_mode_for_session(&self, session_id: SessionId) -> Option<ClientMode> {
        self.client_by_session.get(&session_id).map(|c| c.get_mode())
    }

    /// Java: `isSessionAdmin(Session)`
    pub fn is_session_admin(&self, session_id: SessionId) -> bool {
        self.client_by_session.get(&session_id).map(|c| c.has_property("ADMIN")).unwrap_or(false)
    }

    /// Java: `isSessionDev(Session)`
    pub fn is_session_dev(&self, session_id: SessionId) -> bool {
        self.client_by_session.get(&session_id).map(|c| c.has_property("DEV")).unwrap_or(false)
    }

    /// Java: `hasEditPrivilege(Session)`
    pub fn has_edit_privilege(&self, session_id: SessionId) -> bool {
        self.client_by_session.get(&session_id).map(|c| c.has_property("STATE_EDIT")).unwrap_or(false)
    }

    /// Java: `getSessionsForGameId(long)`
    pub fn get_sessions_for_game_id(&self, game_id: i64) -> Vec<SessionId> {
        self.sessions_by_game_id.get(&game_id).map(|s| s.iter().copied().collect()).unwrap_or_default()
    }

    /// Java: `getSessionOfHomeCoach(long)`
    pub fn get_session_of_home_coach(&self, game_id: i64) -> Option<SessionId> {
        let sessions = self.sessions_by_game_id.get(&game_id)?;
        sessions.iter().find(|&&sid| {
            self.client_by_session.get(&sid).map(|c| c.get_mode() == ClientMode::PLAYER && c.is_home_coach()).unwrap_or(false)
        }).copied()
    }

    /// Java: `getSessionOfAwayCoach(long)`
    pub fn get_session_of_away_coach(&self, game_id: i64) -> Option<SessionId> {
        let sessions = self.sessions_by_game_id.get(&game_id)?;
        sessions.iter().find(|&&sid| {
            self.client_by_session.get(&sid).map(|c| c.get_mode() == ClientMode::PLAYER && !c.is_home_coach()).unwrap_or(false)
        }).copied()
    }

    /// Java: `isHomeCoach(long, String)`
    pub fn is_home_coach(&self, game_id: i64, coach: &str) -> bool {
        self.get_session_of_home_coach(game_id)
            .and_then(|sid| self.client_by_session.get(&sid))
            .map(|c| c.get_coach().eq_ignore_ascii_case(coach))
            .unwrap_or(false)
    }

    /// Java: `isAwayCoach(long, String)`
    pub fn is_away_coach(&self, game_id: i64, coach: &str) -> bool {
        self.get_session_of_away_coach(game_id)
            .and_then(|sid| self.client_by_session.get(&sid))
            .map(|c| c.get_coach().eq_ignore_ascii_case(coach))
            .unwrap_or(false)
    }

    /// Java: `getSessionsWithoutAwayCoach(long)`
    pub fn get_sessions_without_away_coach(&self, game_id: i64) -> Vec<SessionId> {
        let away = self.get_session_of_away_coach(game_id);
        self.get_sessions_for_game_id(game_id).into_iter().filter(|&s| Some(s) != away).collect()
    }

    /// Java: `getSessionsOfSpectators(long)`
    pub fn get_sessions_of_spectators(&self, game_id: i64) -> Vec<SessionId> {
        let home = self.get_session_of_home_coach(game_id);
        let away = self.get_session_of_away_coach(game_id);
        self.get_sessions_for_game_id(game_id).into_iter().filter(|&s| Some(s) != home && Some(s) != away).collect()
    }

    /// Java: `getAllSessions()`
    pub fn get_all_sessions(&self) -> Vec<SessionId> {
        self.client_by_session.keys().copied().collect()
    }

    /// Java: `setLastPing(Session, long)`
    pub fn set_last_ping(&mut self, session_id: SessionId, ping: i64) {
        self.last_ping_by_session.insert(session_id, ping);
    }

    /// Java: `getLastPing(Session)`
    pub fn get_last_ping(&self, session_id: SessionId) -> i64 {
        *self.last_ping_by_session.get(&session_id).unwrap_or(&0)
    }

    /// Send a JSON string to a single session.
    pub fn send_to(&self, session_id: SessionId, msg: &str) {
        if let Some(sender) = self.senders.get(&session_id) {
            let _ = sender.send(msg.to_string());
        }
    }

    /// Send to all sessions in a game (Java: `sendAllSessions`).
    pub fn send_all(&self, game_id: i64, msg: &str) {
        for sid in self.get_sessions_for_game_id(game_id) {
            self.send_to(sid, msg);
        }
    }

    /// Send to home coach + spectators (Java: `sendHomeAndSpectatorSessions`).
    pub fn send_home_and_spectators(&self, game_id: i64, msg: &str) {
        for sid in self.get_sessions_without_away_coach(game_id) {
            self.send_to(sid, msg);
        }
    }

    /// Send only to away coach (Java: `sendAwaySession`).
    pub fn send_away(&self, game_id: i64, msg: &str) {
        if let Some(sid) = self.get_session_of_away_coach(game_id) {
            self.send_to(sid, msg);
        }
    }

    /// Find an existing session for a given coach in a game (used to close duplicates).
    pub fn find_other_session_for_coach(&self, game_id: i64, coach: &str, exclude: SessionId) -> Option<SessionId> {
        self.get_sessions_for_game_id(game_id).into_iter().find(|&sid| {
            sid != exclude && self.get_coach_for_session(sid).map(|c| c.eq_ignore_ascii_case(coach)).unwrap_or(false)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sender() -> (mpsc::UnboundedSender<String>, mpsc::UnboundedReceiver<String>) {
        mpsc::unbounded_channel()
    }

    #[test]
    fn add_and_lookup_session() {
        let mut sm = SessionManager::new();
        let (tx, _rx) = make_sender();
        sm.add_session(1, 100, "Home".into(), ClientMode::PLAYER, true, vec![], tx);
        assert_eq!(sm.get_game_id_for_session(1), 100);
        assert_eq!(sm.get_coach_for_session(1), Some("Home"));
        assert_eq!(sm.get_mode_for_session(1), Some(ClientMode::PLAYER));
    }

    #[test]
    fn home_and_away_lookup() {
        let mut sm = SessionManager::new();
        let (tx1, _) = make_sender();
        let (tx2, _) = make_sender();
        sm.add_session(1, 100, "Home".into(), ClientMode::PLAYER, true, vec![], tx1);
        sm.add_session(2, 100, "Away".into(), ClientMode::PLAYER, false, vec![], tx2);
        assert_eq!(sm.get_session_of_home_coach(100), Some(1));
        assert_eq!(sm.get_session_of_away_coach(100), Some(2));
    }

    #[test]
    fn remove_session_cleans_up() {
        let mut sm = SessionManager::new();
        let (tx, _) = make_sender();
        sm.add_session(1, 100, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        sm.remove_session(1);
        assert_eq!(sm.get_game_id_for_session(1), 0);
        assert!(sm.get_sessions_for_game_id(100).is_empty());
    }

    #[test]
    fn sessions_without_away_excludes_away() {
        let mut sm = SessionManager::new();
        let (tx1, _) = make_sender();
        let (tx2, _) = make_sender();
        let (tx3, _) = make_sender();
        sm.add_session(1, 100, "Home".into(), ClientMode::PLAYER, true, vec![], tx1);
        sm.add_session(2, 100, "Away".into(), ClientMode::PLAYER, false, vec![], tx2);
        sm.add_session(3, 100, "Spec".into(), ClientMode::SPECTATOR, false, vec![], tx3);
        let without_away = sm.get_sessions_without_away_coach(100);
        assert!(without_away.contains(&1));
        assert!(without_away.contains(&3));
        assert!(!without_away.contains(&2));
    }

    #[test]
    fn send_all_delivers_to_all_sessions() {
        let mut sm = SessionManager::new();
        let (tx1, mut rx1) = make_sender();
        let (tx2, mut rx2) = make_sender();
        sm.add_session(1, 100, "A".into(), ClientMode::PLAYER, true, vec![], tx1);
        sm.add_session(2, 100, "B".into(), ClientMode::PLAYER, false, vec![], tx2);
        sm.send_all(100, "hello");
        assert_eq!(rx1.try_recv().unwrap(), "hello");
        assert_eq!(rx2.try_recv().unwrap(), "hello");
    }
}
