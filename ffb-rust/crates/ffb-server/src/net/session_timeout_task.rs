/// 1:1 translation of com.fumbbl.ffb.server.net.SessionTimeoutTask.
///
/// Java: a `TimerTask` (scheduled only when `TIMER_SESSION_TIMEOUT_ENABLED` is
/// set) that closes any session — regular or replay — whose last ping is
/// older than `timeout`.
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;
use crate::net::server_communication::ServerCommunication;
use crate::net::session_manager::SessionManager;

pub struct SessionTimeoutTask {
    pub timeout: i64,
    session_manager: Arc<Mutex<SessionManager>>,
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    communication: Arc<ServerCommunication>,
}

impl SessionTimeoutTask {
    /// Java: `SessionTimeoutTask(SessionManager, ReplaySessionManager, ServerCommunication, long)`.
    pub fn new(
        session_manager: Arc<Mutex<SessionManager>>,
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
        communication: Arc<ServerCommunication>,
        timeout: i64,
    ) -> Self {
        Self { timeout, session_manager, replay_session_manager, communication }
    }

    /// Java:
    /// ```java
    /// public void run() {
    ///     Arrays.stream(sessionManager.getAllSessions())
    ///         .filter(session -> sessionManager.getLastPing(session) + timeout < System.currentTimeMillis())
    ///         .forEach(communication::close);
    ///     Arrays.stream(replaySessionManager.getAllSessions())
    ///         .filter(session -> replaySessionManager.getLastPing(session) + timeout < System.currentTimeMillis())
    ///         .forEach(communication::close);
    /// }
    /// ```
    pub fn run(&self) {
        let now = current_time_millis();

        let expired: Vec<SessionId> = {
            let sm = self.session_manager.lock().unwrap();
            Self::expired_sessions(&sm, self.timeout, now)
        };
        for session in expired {
            self.communication.close(session);
        }

        let expired_replay: Vec<SessionId> = {
            let rsm = self.replay_session_manager.lock().unwrap();
            Self::expired_replay_sessions(&rsm, self.timeout, now)
        };
        for session in expired_replay {
            self.communication.close(session);
        }
    }

    /// Java: `sessionManager.getAllSessions()` filtered by `getLastPing(session) + timeout < now`.
    /// Extracted as a pure, testable function per project convention for `TimerTask` ports.
    fn expired_sessions(session_manager: &SessionManager, timeout: i64, now: i64) -> Vec<SessionId> {
        session_manager
            .get_all_sessions()
            .into_iter()
            .filter(|&session| session_manager.get_last_ping(session) + timeout < now)
            .collect()
    }

    /// Java: `replaySessionManager.getAllSessions()` filtered the same way.
    fn expired_replay_sessions(
        replay_session_manager: &ReplaySessionManager,
        timeout: i64,
        now: i64,
    ) -> Vec<SessionId> {
        replay_session_manager
            .get_all_sessions()
            .into_iter()
            .filter(|&session| replay_session_manager.get_last_ping(session) + timeout < now)
            .collect()
    }
}

fn current_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;
    use tokio::sync::mpsc;

    fn make_communication() -> (
        Arc<ServerCommunication>,
        Arc<Mutex<crate::game_cache::GameCache>>,
        Arc<Mutex<SessionManager>>,
    ) {
        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let db = Arc::new(Mutex::new(crate::db::db_connection_manager::DbConnectionManager::new()));
        let comm = Arc::new(ServerCommunication::new(Arc::clone(&gc), Arc::clone(&sm), db));
        (comm, gc, sm)
    }

    #[test]
    fn expired_sessions_filters_by_timeout() {
        let mut sm = SessionManager::new();
        let (tx1, _) = mpsc::unbounded_channel();
        let (tx2, _) = mpsc::unbounded_channel();
        sm.add_session(1, 100, "Stale".into(), ClientMode::PLAYER, true, vec![], tx1);
        sm.add_session(2, 100, "Fresh".into(), ClientMode::PLAYER, false, vec![], tx2);
        sm.set_last_ping(1, 0);
        sm.set_last_ping(2, 10_000);

        let expired = SessionTimeoutTask::expired_sessions(&sm, 5_000, 10_000);
        assert_eq!(expired, vec![1]);
    }

    #[test]
    fn expired_sessions_empty_when_all_fresh() {
        let mut sm = SessionManager::new();
        let (tx, _) = mpsc::unbounded_channel();
        sm.add_session(1, 100, "Fresh".into(), ClientMode::PLAYER, true, vec![], tx);
        sm.set_last_ping(1, 9_000);
        let expired = SessionTimeoutTask::expired_sessions(&sm, 5_000, 10_000);
        assert!(expired.is_empty());
    }

    #[test]
    fn expired_replay_sessions_filters_by_timeout() {
        let mut rsm = ReplaySessionManager::new();
        rsm.add_session(1, "replay".into(), "Coach".into());
        rsm.set_last_ping(1, 0);
        let expired = SessionTimeoutTask::expired_replay_sessions(&rsm, 5_000, 10_000);
        assert_eq!(expired, vec![1]);
    }

    #[tokio::test]
    async fn run_closes_expired_regular_session() {
        let (comm, _gc, sm) = make_communication();
        let (tx, _rx) = mpsc::unbounded_channel();
        {
            let mut guard = sm.lock().unwrap();
            guard.add_session(1, 100, "Stale".into(), ClientMode::PLAYER, true, vec![], tx);
            guard.set_last_ping(1, 0);
        }
        let rsm = Arc::new(Mutex::new(ReplaySessionManager::new()));
        let task = SessionTimeoutTask::new(Arc::clone(&sm), rsm, comm, 5_000);

        task.run();

        assert_eq!(sm.lock().unwrap().get_game_id_for_session(1), 0);
    }

    #[tokio::test]
    async fn run_leaves_fresh_session_untouched() {
        let (comm, _gc, sm) = make_communication();
        let (tx, _rx) = mpsc::unbounded_channel();
        {
            let mut guard = sm.lock().unwrap();
            guard.add_session(1, 100, "Fresh".into(), ClientMode::PLAYER, true, vec![], tx);
            guard.set_last_ping(1, current_time_millis());
        }
        let rsm = Arc::new(Mutex::new(ReplaySessionManager::new()));
        let task = SessionTimeoutTask::new(Arc::clone(&sm), rsm, comm, 5_000);

        task.run();

        assert_eq!(sm.lock().unwrap().get_game_id_for_session(1), 100);
    }
}
