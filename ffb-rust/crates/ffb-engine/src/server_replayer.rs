use std::collections::VecDeque;

use crate::server_replay::ServerReplay;

/// Background thread that sends replay commands to clients — 1:1 translation of Java ServerReplayer.
pub struct ServerReplayer {
    replay_queue: VecDeque<ServerReplay>,
    stopped: bool,
}

impl ServerReplayer {
    pub fn new() -> Self {
        Self {
            replay_queue: VecDeque::new(),
            stopped: false,
        }
    }

    pub fn add(&mut self, replay: ServerReplay) {
        self.replay_queue.push_back(replay);
    }

    pub fn stop(&mut self) {
        self.stopped = true;
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    pub fn queue_size(&self) -> usize {
        self.replay_queue.len()
    }

    pub fn run(&mut self) {
        // Phase ZU: process replay_queue and send commands over WebSocket
        todo!("Phase ZU: WebSocket replay dispatch")
    }
}

impl Default for ServerReplayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_not_stopped() {
        let replayer = ServerReplayer::new();
        assert!(!replayer.is_stopped());
        assert_eq!(replayer.queue_size(), 0);
    }

    #[test]
    fn test_add_to_queue() {
        let mut replayer = ServerReplayer::new();
        replayer.add(ServerReplay::new(1, 10));
        assert_eq!(replayer.queue_size(), 1);
    }
}
