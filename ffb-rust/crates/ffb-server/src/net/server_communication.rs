/// 1:1 translation of com.fumbbl.ffb.server.net.ServerCommunication.
///
/// Java uses a `BlockingQueue<ReceivedCommand>` + single dispatch thread.
/// Rust uses a tokio mpsc channel + single async task.
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use crate::db::db_connection_manager::DbConnectionManager;
use crate::game_cache::GameCache;
use crate::handler::ServerCommandHandlerFactory;
use crate::model::received_command::{ReceivedCommand, SessionId};
use crate::net::commands::any_internal_server_command::AnyInternalServerCommand;
use crate::net::replay_session_manager::ReplaySessionManager;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommunication`
pub struct ServerCommunication {
    /// Java: the Jetty server reference (here we keep the sender to enqueue commands)
    tx: mpsc::UnboundedSender<ReceivedCommand>,
    session_manager: Arc<Mutex<SessionManager>>,
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
}

impl ServerCommunication {
    /// Java: `new ServerCommunication(server)` — creates the channel and spawns the dispatch task.
    pub fn new(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
        db_connection_manager: Arc<Mutex<DbConnectionManager>>,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<ReceivedCommand>();
        // Own the ReplaySessionManager here and share it with the handler
        // factory, so `sendToReplaySession`/`close` (below) and the replay
        // handlers (`ServerCommandHandlerJoinReplay`, etc.) see the same
        // bookkeeping — mirrors Java's single `FantasyFootballServer`-owned
        // `ReplaySessionManager` instance.
        let replay_session_manager = Arc::new(Mutex::new(ReplaySessionManager::new()));
        let factory = ServerCommandHandlerFactory::with_replay_session_manager(
            Arc::clone(&game_cache),
            Arc::clone(&session_manager),
            Arc::clone(&replay_session_manager),
            db_connection_manager,
            tx.clone(),
        );
        tokio::spawn(dispatch_loop(rx, factory));
        Self { tx, session_manager, replay_session_manager }
    }

    /// Builds a `ServerCommunication` handle sharing an existing `tx`/`session_manager`/
    /// `replay_session_manager` triple, instead of spawning a fresh dispatch task and private
    /// `ReplaySessionManager` the way `new` does. `ServerCommandHandlerFactory` uses this to
    /// get a real `&ServerCommunication` to hand to `ServerCommandHandlerCloseGame` (Java:
    /// `getServer().getCommunication()`) — it already owns these same three cheaply-cloneable
    /// handles, and calling `new` here would spawn a second, disconnected dispatch loop (and,
    /// since the factory itself is what `new`'s spawned task owns, calling it from inside the
    /// factory's own constructor would recurse indefinitely).
    pub(crate) fn from_parts(
        tx: mpsc::UnboundedSender<ReceivedCommand>,
        session_manager: Arc<Mutex<SessionManager>>,
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    ) -> Self {
        Self { tx, session_manager, replay_session_manager }
    }

    /// Java: `receiveCommand(ReceivedCommand)` — enqueue for dispatch.
    pub fn receive_command(&self, cmd: ReceivedCommand) {
        if let Err(e) = self.tx.send(cmd) {
            log::error!("dispatch channel closed, could not enqueue command: {}", e);
        }
    }

    /// Java: `handleCommand(InternalServerCommand)` — wraps the internal command in a
    /// `ReceivedCommand` and enqueues it on the same dispatch queue as client commands
    /// (`fCommandQueue.offer(new ReceivedCommand(internalCommand, null))`). This is the
    /// redispatch sink a handler uses to hand a follow-up command back through dispatch —
    /// e.g. `ServerCommandHandlerJoin` enqueuing an `InternalServerCommandJoinApproved`
    /// (see that handler's doc comment for why it isn't wired to call this yet: its own
    /// blocker is the async DB password check that must happen first, not this sink).
    /// The single-consumer `dispatch_loop` task already serializes processing exactly like
    /// Java's single `BlockingQueue` thread, so redispatching here needs no extra
    /// synchronization beyond the existing channel.
    pub fn receive_internal(&self, cmd: AnyInternalServerCommand, session_id: SessionId) {
        if let Err(e) = self.tx.send(ReceivedCommand::new_internal(cmd, session_id)) {
            log::error!("dispatch channel closed, could not enqueue internal command: {}", e);
        }
    }

    /// Clone the sender so WebSocket tasks can enqueue commands.
    pub fn sender(&self) -> mpsc::UnboundedSender<ReceivedCommand> {
        self.tx.clone()
    }

    /// Java: `getServer().getSessionManager()`-equivalent accessor, exposed so
    /// callers that construct `ServerCommunication` (e.g. `FantasyFootballServer::run`)
    /// can share the same `SessionManager` with tasks like `SessionTimeoutTask`.
    pub fn session_manager(&self) -> Arc<Mutex<SessionManager>> {
        Arc::clone(&self.session_manager)
    }

    /// Shares the `ReplaySessionManager` this instance owns, so replay-aware
    /// tasks (`SessionTimeoutTask`) and handlers see the same bookkeeping.
    pub fn replay_session_manager(&self) -> Arc<Mutex<ReplaySessionManager>> {
        Arc::clone(&self.replay_session_manager)
    }

    /// Java: `close(Session pSession)`.
    /// ```java
    /// public void close(Session pSession) {
    ///     if (pSession == null) { return; }
    ///     pSession.close();
    ///     handleCommand(new ReceivedCommand(new InternalServerCommandSocketClosed(), pSession));
    /// }
    /// ```
    /// `pSession.close()` (the actual network close) is modeled by dropping the
    /// session's sender via `remove_session` — see `command_socket.rs`'s
    /// `out_rx.recv() -> None` branch, which is exactly what a dropped sender
    /// triggers. The fuller `InternalServerCommandSocketClosed` side effects
    /// (leave broadcast, sketch cleanup, replay-control handoff) already exist
    /// as `ServerCommandHandlerSocketClosed`, but that handler needs
    /// `GameCache`/`ServerSketchManager` this struct doesn't hold and isn't
    /// wired into command dispatch yet (see `ServerCommandHandlerFactory`'s
    /// documented "Known gap" comment) — a further follow-up.
    pub fn close(&self, session_id: SessionId) {
        let is_replay = self.replay_session_manager.lock().unwrap().has(session_id);
        if is_replay {
            self.replay_session_manager.lock().unwrap().remove_session(session_id);
        } else {
            self.session_manager.lock().unwrap().remove_session(session_id);
        }
    }

    /// Java: `sendToReplaySession(Session session, NetCommand command)`.
    /// ```java
    /// public void sendToReplaySession(Session session, NetCommand command) {
    ///     if ((session == null) || (command == null)) { return; }
    ///     getServer().getDebugLog().logReplay(...);
    ///     send(session, command, false);
    /// }
    /// ```
    /// `message` stands in for the already-serialized `NetCommand` (this crate's
    /// wire commands are serialized to JSON strings before being handed to a
    /// session's outgoing channel, same as `SessionManager::send_to`).
    pub fn send_to_replay_session(&self, session_id: SessionId, message: &str) {
        log::debug!("replay send to session {}: {}", session_id, message);
        self.replay_session_manager.lock().unwrap().send_to(session_id, message);
    }

    /// Java: `sendGameTime(GameState gameState)`.
    /// ```java
    /// public void sendGameTime(GameState gameState) {
    ///     if (gameState != null) {
    ///         ServerCommandGameTime gameTimeCommand = new ServerCommandGameTime(...);
    ///         sendAllSessions(gameState, gameTimeCommand, false);
    ///     }
    /// }
    /// ```
    /// `sendAllSessions` ultimately broadcasts to `sessionManager.getSessionsForGameId(...)`,
    /// which `SessionManager::send_all` already does directly.
    pub fn send_game_time(&self, game_id: i64, message: &str) {
        self.session_manager.lock().unwrap().send_all(game_id, message);
    }
}

/// Single async dispatch task — mirrors Java's single BlockingQueue consumer thread.
async fn dispatch_loop(
    mut rx: mpsc::UnboundedReceiver<ReceivedCommand>,
    factory: ServerCommandHandlerFactory,
) {
    log::info!("ServerCommunication dispatch loop started");
    while let Some(received) = rx.recv().await {
        factory.handle_command(received).await;
    }
    log::info!("ServerCommunication dispatch loop ended (channel closed)");
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_protocol::client_commands::{ClientCommand, ClientPing};

    fn db() -> Arc<Mutex<DbConnectionManager>> {
        Arc::new(Mutex::new(DbConnectionManager::new()))
    }

    #[tokio::test]
    async fn enqueue_ping_does_not_panic() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let sc = ServerCommunication::new(gc, sm, db());
        sc.receive_command(ReceivedCommand::new(ClientCommand::ClientPing(ClientPing { timestamp: 42 }), 0));
        // Give the dispatch task a chance to run
        tokio::task::yield_now().await;
    }

    #[tokio::test]
    async fn receive_internal_reaches_socket_closed_handler_via_dispatch_loop() {
        use crate::net::commands::internal_server_command_socket_closed::InternalServerCommandSocketClosed;
        use ffb_model::model::ClientMode;

        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        {
            let (tx, _rx) = mpsc::unbounded_channel();
            sm.lock().unwrap().add_session(1, 0, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        }
        let sc = ServerCommunication::new(Arc::clone(&gc), Arc::clone(&sm), db());

        assert!(sm.lock().unwrap().get_coach_for_session(1).is_some());
        sc.receive_internal(AnyInternalServerCommand::SocketClosed(InternalServerCommandSocketClosed), 1);

        // Give the real dispatch loop task a chance to drain the queue.
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }

        assert!(sm.lock().unwrap().get_coach_for_session(1).is_none(), "SocketClosed should have removed the session");
    }

    #[test]
    fn sender_clone_works() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let gc = Arc::new(Mutex::new(GameCache::new()));
            let sm = Arc::new(Mutex::new(SessionManager::new()));
            let sc = ServerCommunication::new(gc, sm, db());
            let _clone = sc.sender();
        });
    }

    #[tokio::test]
    async fn close_removes_regular_session() {
        use ffb_model::model::ClientMode;
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, 100, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        let sc = ServerCommunication::new(Arc::clone(&gc), Arc::clone(&sm), db());

        sc.close(1);

        assert_eq!(sm.lock().unwrap().get_game_id_for_session(1), 0);
    }

    #[tokio::test]
    async fn close_removes_replay_session() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let sc = ServerCommunication::new(gc, sm, db());
        sc.replay_session_manager().lock().unwrap().add_session(7, "replay".into(), "Coach".into());

        sc.close(7);

        assert!(!sc.replay_session_manager().lock().unwrap().has(7));
    }

    #[tokio::test]
    async fn send_to_replay_session_delivers_message() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let sc = ServerCommunication::new(gc, sm, db());
        let (tx, mut rx) = mpsc::unbounded_channel();
        {
            let rsm = sc.replay_session_manager();
            let mut guard = rsm.lock().unwrap();
            guard.add_session(3, "replay".into(), "Coach".into());
            guard.register_sender(3, tx);
        }

        sc.send_to_replay_session(3, "hello replay");

        assert_eq!(rx.try_recv().unwrap(), "hello replay");
    }

    #[tokio::test]
    async fn send_game_time_broadcasts_to_game_sessions() {
        use ffb_model::model::ClientMode;
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, 100, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        let sc = ServerCommunication::new(gc, Arc::clone(&sm), db());

        sc.send_game_time(100, "tick");

        assert_eq!(rx.try_recv().unwrap(), "tick");
    }

    #[tokio::test]
    async fn session_manager_accessor_shares_same_arc() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let sc = ServerCommunication::new(gc, Arc::clone(&sm), db());
        assert!(Arc::ptr_eq(&sm, &sc.session_manager()));
    }
}
