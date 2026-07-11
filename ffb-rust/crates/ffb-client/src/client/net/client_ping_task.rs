//! 1:1 translation of `com.fumbbl.ffb.client.net.ClientPingTask`.
//!
//! Java: a `TimerTask` (`fClient.getScheduler().schedule(new ClientPingTask(this), ...)`-style
//! usage) whose `run()` sends a ping if the socket is open. `getClient()` (`FantasyFootballClient`)
//! is the permanently-skipped GUI shell, so `tick()` below takes the two things Java reads off it
//! (`getCommandEndpoint().isOpen()`, `getCommunication()`) as explicit parameters instead — same
//! substitution used throughout `ClientCommunication`'s own translation. Per the project's
//! established `TimerTask` port convention (see `ffb-server/src/net/server_game_time_task.rs`),
//! the per-tick logic is extracted into a plain, testable method rather than wrapped in a live
//! `tokio::time::interval` loop.

use crate::client::net::client_communication::ClientCommunication;

/// Java: `com.fumbbl.ffb.client.net.ClientPingTask`.
pub struct ClientPingTask;

impl ClientPingTask {
    /// Java: `ClientPingTask(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self
    }

    /// Java:
    /// ```java
    /// public void run() {
    ///     if (getClient().getCommandEndpoint().isOpen()) {
    ///         getClient().getCommunication().sendPing(System.currentTimeMillis());
    ///     }
    /// }
    /// ```
    /// `is_open`/`communication`/`now` stand in for `getClient().getCommandEndpoint().isOpen()`,
    /// `getClient().getCommunication()`, and `System.currentTimeMillis()` respectively.
    pub fn tick(is_open: bool, communication: &mut ClientCommunication, now: i64) {
        if is_open {
            communication.send_ping(now);
        }
    }
}

impl Default for ClientPingTask {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = ClientPingTask::new();
    }

    #[test]
    fn default_constructs() {
        let _ = ClientPingTask::default();
    }

    #[test]
    fn tick_sends_ping_when_open() {
        let mut comm = ClientCommunication::new();
        ClientPingTask::tick(true, &mut comm, 12345);
        assert_eq!(comm.outbox.len(), 1);
        assert_eq!(comm.outbox[0]["netCommandId"], "clientPing");
        assert_eq!(comm.outbox[0]["timestamp"], 12345);
    }

    #[test]
    fn tick_does_not_send_when_closed() {
        let mut comm = ClientCommunication::new();
        ClientPingTask::tick(false, &mut comm, 12345);
        assert!(comm.outbox.is_empty());
    }
}
