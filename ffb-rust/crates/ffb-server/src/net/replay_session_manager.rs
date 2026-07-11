/// 1:1 translation of com.fumbbl.ffb.server.net.ReplaySessionManager.
/// Manages sessions watching shared replays.
/// Java uses Jetty Session objects; here we use u64 session IDs.
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;

#[derive(Clone)]
struct ReplayClient {
    shared_replay_name: String,
    coach: String,
    control: bool,
}

impl ReplayClient {
    fn new(shared_replay_name: String, coach: String) -> Self {
        Self { shared_replay_name, coach, control: false }
    }

    fn get_coach(&self) -> &str {
        &self.coach
    }

    fn has_control(&self) -> bool {
        self.control
    }

    fn set_control(&mut self, control: bool) {
        self.control = control;
    }
}

pub struct ReplaySessionManager {
    sessions_for_replay: HashMap<String, HashSet<u64>>,
    replay_client_for_session: HashMap<u64, ReplayClient>,
    last_ping_by_session: HashMap<u64, i64>,
    /// AutoMarkingConfig stored as opaque String; full type not yet available.
    auto_marking_by_session: HashMap<u64, String>,
    /// Sender half of each replay session's output channel — no Java equivalent
    /// (Jetty's `Session.getRemote()` plays this role there); needed so
    /// `ServerCommunication.sendToReplaySession` has somewhere to write to.
    senders: HashMap<u64, mpsc::UnboundedSender<String>>,
}

impl ReplaySessionManager {
    pub fn new() -> Self {
        Self {
            sessions_for_replay: HashMap::new(),
            replay_client_for_session: HashMap::new(),
            last_ping_by_session: HashMap::new(),
            auto_marking_by_session: HashMap::new(),
            senders: HashMap::new(),
        }
    }

    /// Not a Java method — registers the outgoing channel for a replay session
    /// (the Rust stand-in for Jetty handing back `Session.getRemote()`), so
    /// `send_to`/`ServerCommunication.sendToReplaySession` has somewhere to write.
    pub fn register_sender(&mut self, session: u64, sender: mpsc::UnboundedSender<String>) {
        self.senders.insert(session, sender);
    }

    /// Java: `ServerCommunication.sendToReplaySession(Session, NetCommand)`'s
    /// actual network write (the private `send(Session, NetCommand)` helper).
    pub fn send_to(&self, session: u64, message: &str) {
        if let Some(sender) = self.senders.get(&session) {
            let _ = sender.send(message.to_string());
        }
    }

    pub fn add_session(&mut self, session: u64, name: String, coach: String) {
        let mut client = ReplayClient::new(name.clone(), coach);
        let sessions = self.sessions_for_replay.entry(name).or_insert_with(HashSet::new);
        client.set_control(sessions.is_empty());
        sessions.insert(session);
        self.replay_client_for_session.insert(session, client);
        self.last_ping_by_session.insert(session, 0);
    }

    pub fn remove_session(&mut self, session: u64) {
        let name = self.get_shared_replay_name(session);
        if let Some(sessions) = self.sessions_for_replay.get_mut(&name) {
            sessions.remove(&session);
            if sessions.is_empty() {
                self.sessions_for_replay.remove(&name);
            }
        }
        self.last_ping_by_session.remove(&session);
        self.replay_client_for_session.remove(&session);
        self.auto_marking_by_session.remove(&session);
        self.senders.remove(&session);
    }

    fn get_shared_replay_name(&self, session: u64) -> String {
        self.replay_client_for_session
            .get(&session)
            .map(|c| c.shared_replay_name.clone())
            .unwrap_or_default()
    }

    pub fn sessions_for_replay(&self, replay_name: &str) -> Option<Vec<u64>> {
        self.sessions_for_replay.get(replay_name).map(|s| s.iter().copied().collect())
    }

    pub fn replay_for_session(&self, session: u64) -> String {
        self.replay_client_for_session
            .get(&session)
            .map(|c| c.shared_replay_name.clone())
            .unwrap_or_default()
    }

    pub fn coach(&self, session: u64) -> Option<String> {
        self.replay_client_for_session.get(&session).map(|c| c.coach.clone())
    }

    pub fn replay_name_for_session(&self, session: u64) -> String {
        self.get_shared_replay_name(session)
    }

    pub fn other_sessions(&self, session: u64) -> Vec<u64> {
        let replay_name = self.replay_name_for_session(session);
        if replay_name.is_empty() {
            return vec![];
        }
        self.sessions_for_replay
            .get(&replay_name)
            .map(|s| s.iter().copied().filter(|&s2| s2 != session).collect())
            .unwrap_or_default()
    }

    pub fn has(&self, session: u64) -> bool {
        self.replay_client_for_session.contains_key(&session)
            || self.auto_marking_by_session.contains_key(&session)
    }

    pub fn add_auto_marking(&mut self, session: u64, config: String) {
        self.auto_marking_by_session.insert(session, config);
    }

    pub fn get_auto_marking(&self, session: u64) -> Option<&str> {
        self.auto_marking_by_session.get(&session).map(|s| s.as_str())
    }

    pub fn set_last_ping(&mut self, session: u64, ping: i64) {
        self.last_ping_by_session.insert(session, ping);
    }

    pub fn get_last_ping(&self, session: u64) -> i64 {
        self.last_ping_by_session.get(&session).copied().unwrap_or(0)
    }

    pub fn get_all_sessions(&self) -> Vec<u64> {
        self.replay_client_for_session.keys().copied().collect()
    }

    pub fn has_control(&self, session: u64) -> bool {
        self.replay_client_for_session
            .get(&session)
            .map(|c| c.has_control())
            .unwrap_or(false)
    }

    pub fn controlling_coach(&self, session: u64) -> String {
        let replay_name = self.replay_name_for_session(session);
        if let Some(sessions) = self.sessions_for_replay.get(&replay_name) {
            for &s in sessions {
                if let Some(client) = self.replay_client_for_session.get(&s) {
                    if client.has_control() {
                        return client.get_coach().to_string();
                    }
                }
            }
        }
        String::new()
    }

    pub fn transfer_control(&mut self, controlling_session: u64, coach: &str) -> bool {
        if !self.has_control(controlling_session) {
            return false;
        }
        let others = self.other_sessions(controlling_session);
        let target = others
            .iter()
            .copied()
            .find(|&s| self.coach(s).as_deref() == Some(coach));
        if let Some(target_session) = target {
            if let Some(client) = self.replay_client_for_session.get_mut(&controlling_session) {
                client.set_control(false);
            }
            if let Some(client) = self.replay_client_for_session.get_mut(&target_session) {
                client.set_control(true);
            }
            true
        } else {
            false
        }
    }
}

impl Default for ReplaySessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = ReplaySessionManager::new();
    }

    #[test]
    fn add_and_remove_session() {
        let mut m = ReplaySessionManager::new();
        m.add_session(1, "replay1".to_string(), "coach1".to_string());
        assert!(m.has(1));
        m.remove_session(1);
        assert!(!m.has(1));
    }

    #[test]
    fn first_session_gets_control() {
        let mut m = ReplaySessionManager::new();
        m.add_session(1, "r".to_string(), "coach1".to_string());
        assert!(m.has_control(1));
    }

    #[test]
    fn second_session_no_control() {
        let mut m = ReplaySessionManager::new();
        m.add_session(1, "r".to_string(), "coach1".to_string());
        m.add_session(2, "r".to_string(), "coach2".to_string());
        assert!(!m.has_control(2));
    }

    #[test]
    fn get_last_ping_default_zero() {
        let m = ReplaySessionManager::new();
        assert_eq!(m.get_last_ping(99), 0);
    }

    #[test]
    fn set_and_get_last_ping() {
        let mut m = ReplaySessionManager::new();
        m.add_session(1, "r".to_string(), "c".to_string());
        m.set_last_ping(1, 12345);
        assert_eq!(m.get_last_ping(1), 12345);
    }

    #[test]
    fn transfer_control() {
        let mut m = ReplaySessionManager::new();
        m.add_session(1, "r".to_string(), "coach1".to_string());
        m.add_session(2, "r".to_string(), "coach2".to_string());
        assert!(m.transfer_control(1, "coach2"));
        assert!(!m.has_control(1));
        assert!(m.has_control(2));
    }

    #[test]
    fn register_sender_and_send_to_delivers_message() {
        let mut m = ReplaySessionManager::new();
        m.add_session(1, "r".to_string(), "coach1".to_string());
        let (tx, mut rx) = mpsc::unbounded_channel();
        m.register_sender(1, tx);
        m.send_to(1, "hello");
        assert_eq!(rx.try_recv().unwrap(), "hello");
    }

    #[test]
    fn send_to_without_registered_sender_does_not_panic() {
        let mut m = ReplaySessionManager::new();
        m.add_session(1, "r".to_string(), "coach1".to_string());
        m.send_to(1, "hello");
    }

    #[test]
    fn remove_session_cleans_up_sender() {
        let mut m = ReplaySessionManager::new();
        m.add_session(1, "r".to_string(), "coach1".to_string());
        let (tx, mut rx) = mpsc::unbounded_channel();
        m.register_sender(1, tx);
        m.remove_session(1);
        m.send_to(1, "should not arrive");
        assert!(rx.try_recv().is_err());
    }
}
