/// 1:1 translation of com.fumbbl.ffb.server.net.ServerCommunication.
///
/// Java uses a `BlockingQueue<ReceivedCommand>` + single dispatch thread.
/// Rust uses a tokio mpsc channel + single async task.
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use crate::game_cache::GameCache;
use crate::handler::ServerCommandHandlerFactory;
use crate::model::received_command::ReceivedCommand;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommunication`
pub struct ServerCommunication {
    /// Java: the Jetty server reference (here we keep the sender to enqueue commands)
    tx: mpsc::UnboundedSender<ReceivedCommand>,
}

impl ServerCommunication {
    /// Java: `new ServerCommunication(server)` — creates the channel and spawns the dispatch task.
    pub fn new(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<ReceivedCommand>();
        let factory = ServerCommandHandlerFactory::new(game_cache, session_manager);
        tokio::spawn(dispatch_loop(rx, factory));
        Self { tx }
    }

    /// Java: `receiveCommand(ReceivedCommand)` — enqueue for dispatch.
    pub fn receive_command(&self, cmd: ReceivedCommand) {
        if let Err(e) = self.tx.send(cmd) {
            log::error!("dispatch channel closed, could not enqueue command: {}", e);
        }
    }

    /// Clone the sender so WebSocket tasks can enqueue commands.
    pub fn sender(&self) -> mpsc::UnboundedSender<ReceivedCommand> {
        self.tx.clone()
    }
}

/// Single async dispatch task — mirrors Java's single BlockingQueue consumer thread.
async fn dispatch_loop(
    mut rx: mpsc::UnboundedReceiver<ReceivedCommand>,
    factory: ServerCommandHandlerFactory,
) {
    log::info!("ServerCommunication dispatch loop started");
    while let Some(received) = rx.recv().await {
        factory.handle_command(received);
    }
    log::info!("ServerCommunication dispatch loop ended (channel closed)");
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_protocol::client_commands::{ClientCommand, ClientPing};

    #[tokio::test]
    async fn enqueue_ping_does_not_panic() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let sc = ServerCommunication::new(gc, sm);
        sc.receive_command(ReceivedCommand {
            command: ClientCommand::ClientPing(ClientPing { timestamp: 42 }),
            session_id: 0,
        });
        // Give the dispatch task a chance to run
        tokio::task::yield_now().await;
    }

    #[test]
    fn sender_clone_works() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let gc = Arc::new(Mutex::new(GameCache::new()));
            let sm = Arc::new(Mutex::new(SessionManager::new()));
            let sc = ServerCommunication::new(gc, sm);
            let _clone = sc.sender();
        });
    }
}
