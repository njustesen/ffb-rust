use std::collections::VecDeque;

use ffb_protocol::commands::server_command_replay::ServerCommandReplay;

use crate::server_replay::ServerReplay;

/// Java: `ServerReplayer`'s `markingAffectingChanges` set — the `ModelChangeId`s whose
/// presence in a `SERVER_MODEL_SYNC` command's model changes marks that command as
/// "marking affecting" (used by the client to redraw player markings during replay
/// scrubbing). Matched against each `AnyServerCommand::ServerModelSync`'s serialized
/// `modelChangeList.changes[].change_id` (a `ModelChangeId`, serialized via
/// `#[serde(rename_all = "camelCase")]`) by its JSON name.
const MARKING_AFFECTING_CHANGES: &[&str] = &[
    "fieldModelAddIntensiveTraining",
    "fieldModelAddCardEffect",
    "fieldModelRemoveCardEffect",
    "fieldModelAddPrayer",
    "fieldModelRemovePrayer",
    "playerResultSetSeriousInjury",
    "playerResultSetSeriousInjuryDecay",
];

/// Java: `NetCommandId.SERVER_ADD_PLAYER` / `NetCommandId.SERVER_MODEL_SYNC`'s JSON names.
const SERVER_ADD_PLAYER: &str = "serverAddPlayer";
const SERVER_MODEL_SYNC: &str = "serverModelSync";

/// Not a Java type — Java's `run()` reaches the client directly via
/// `server.getCommunication().send(serverReplay.getSession(), replayCommand, false)`.
/// `ffb-engine` has no networking/session layer (and must not depend on `ffb-server`, which
/// depends on it), so the actual send is abstracted behind this trait; `ffb-server` provides
/// the real implementation (against its `SessionManager`), keeping the dependency pointed
/// the same direction as every other crate boundary in this workspace. This is the
/// "parametrize via trait" option documented as preferred for this exact situation (see
/// `ServerReplayer::run`'s doc comment).
pub trait ReplaySender {
    fn send(&self, session: u64, message: &str);
}

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

    /// Java: `run()`.
    ///
    /// Java runs this as an infinite background `Thread`, blocking on the queue until
    /// `stop()` is called. This crate has no thread/event-loop layer wired in yet (same
    /// documented convention as `request::ServerRequestProcessor::run`), so this drains
    /// whatever is currently queued — for each queued `ServerReplay`, chunking its relevant
    /// commands into `ServerCommandReplay::MAX_NR_OF_COMMANDS`-sized batches exactly as Java's
    /// inner `while (serverReplay != null)` loop does, sending one batch per `sender.send(...)`
    /// call — rather than blocking forever for new work.
    pub fn run(&mut self, sender: &dyn ReplaySender) {
        while !self.stopped {
            let Some(mut replay) = self.replay_queue.pop_front() else { break };

            loop {
                replay.set_complete(true);

                let mut total_nr_of_commands = replay.size() as i32;
                let mut last_command = true;
                let mut command_array: Vec<serde_json::Value> = Vec::new();
                let mut marking_affecting_commands: Vec<i32> = Vec::new();

                for raw in replay.find_relevant_commands_in_log() {
                    let value: serde_json::Value = match serde_json::from_str(raw) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    let command_nr = value.get("commandNr").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let net_command_id = value.get("netCommandId").and_then(|v| v.as_str()).unwrap_or("");

                    match net_command_id {
                        id if id == SERVER_ADD_PLAYER => {
                            marking_affecting_commands.push(command_nr);
                        }
                        id if id == SERVER_MODEL_SYNC => {
                            let changes_marking_affecting = value
                                .get("modelChangeList")
                                .and_then(|v| v.get("changes"))
                                .and_then(|v| v.as_array())
                                .map(|changes| {
                                    changes.iter().any(|change| {
                                        change
                                            .get("change_id")
                                            .and_then(|v| v.as_str())
                                            .map(|id| MARKING_AFFECTING_CHANGES.contains(&id))
                                            .unwrap_or(false)
                                    })
                                })
                                .unwrap_or(false);
                            if changes_marking_affecting {
                                marking_affecting_commands.push(command_nr);
                            }
                        }
                        _ => {}
                    }

                    command_array.push(value);
                    if command_array.len() >= ServerCommandReplay::MAX_NR_OF_COMMANDS {
                        replay.set_complete(false);
                        last_command = false;
                        break;
                    }
                }

                let highest_command_nr = command_array
                    .iter()
                    .filter_map(|v| v.get("commandNr").and_then(|n| n.as_i64()))
                    .max()
                    .unwrap_or(0) as i32;

                if total_nr_of_commands < 0 {
                    total_nr_of_commands = 0;
                }

                let message = serde_json::json!({
                    "netCommandId": "serverReplay",
                    "commandNr": 0,
                    "totalNrOfCommands": total_nr_of_commands,
                    "commandArray": command_array,
                    "lastCommand": last_command,
                    "markingIntervalIndexes": marking_affecting_commands,
                })
                .to_string();

                sender.send(replay.get_session(), &message);

                if !replay.is_complete() {
                    replay.set_from_command_nr(highest_command_nr + 1);
                } else {
                    break;
                }
            }
        }
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
    use crate::game_log::GameLog;
    use ffb_protocol::commands::any_server_command::AnyServerCommand;
    use ffb_protocol::commands::server_command_add_player::ServerCommandAddPlayer;
    use ffb_protocol::commands::server_command_pong::ServerCommandPong;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockSender {
        sent: Mutex<Vec<(u64, String)>>,
    }

    impl ReplaySender for MockSender {
        fn send(&self, session: u64, message: &str) {
            self.sent.lock().unwrap().push((session, message.to_string()));
        }
    }

    fn pong_cmd(command_nr: i32) -> AnyServerCommand {
        let mut cmd = ServerCommandPong::default();
        cmd.command_nr = command_nr;
        AnyServerCommand::ServerPong(cmd)
    }

    #[test]
    fn test_new_not_stopped() {
        let replayer = ServerReplayer::new();
        assert!(!replayer.is_stopped());
        assert_eq!(replayer.queue_size(), 0);
    }

    #[test]
    fn test_add_to_queue() {
        let log = GameLog::new();
        let mut replayer = ServerReplayer::new();
        replayer.add(ServerReplay::new(1, 10, 1, &log));
        assert_eq!(replayer.queue_size(), 1);
    }

    #[test]
    fn run_drains_queue_and_sends_one_batch_for_a_small_replay() {
        let log = GameLog::new();
        log.add(pong_cmd(1));
        log.add(pong_cmd(2));
        let mut replayer = ServerReplayer::new();
        replayer.add(ServerReplay::new(1, 0, 42, &log));

        let sender = MockSender::default();
        replayer.run(&sender);

        assert_eq!(replayer.queue_size(), 0);
        let sent = sender.sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, 42);
        let parsed: serde_json::Value = serde_json::from_str(&sent[0].1).unwrap();
        assert_eq!(parsed["netCommandId"], "serverReplay");
        assert_eq!(parsed["lastCommand"], true);
        assert_eq!(parsed["commandArray"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn run_marks_add_player_commands_as_marking_affecting() {
        let log = GameLog::new();
        log.add(AnyServerCommand::ServerAddPlayer(ServerCommandAddPlayer::default()));
        let mut replayer = ServerReplayer::new();
        replayer.add(ServerReplay::new(1, 0, 1, &log));

        let sender = MockSender::default();
        replayer.run(&sender);

        let sent = sender.sent.lock().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&sent[0].1).unwrap();
        assert_eq!(parsed["markingIntervalIndexes"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["markingIntervalIndexes"][0], 1);
    }

    #[test]
    fn run_chunks_large_replays_into_multiple_batches() {
        let log = GameLog::new();
        for i in 1..=(ServerCommandReplay::MAX_NR_OF_COMMANDS as i32 + 5) {
            log.add(pong_cmd(i));
        }
        let mut replayer = ServerReplayer::new();
        replayer.add(ServerReplay::new(1, 0, 1, &log));

        let sender = MockSender::default();
        replayer.run(&sender);

        let sent = sender.sent.lock().unwrap();
        assert_eq!(sent.len(), 2, "expected two batches for a replay over MAX_NR_OF_COMMANDS");
        let first: serde_json::Value = serde_json::from_str(&sent[0].1).unwrap();
        assert_eq!(first["lastCommand"], false);
        assert_eq!(first["commandArray"].as_array().unwrap().len(), ServerCommandReplay::MAX_NR_OF_COMMANDS);
        let second: serde_json::Value = serde_json::from_str(&sent[1].1).unwrap();
        assert_eq!(second["lastCommand"], true);
        assert_eq!(second["commandArray"].as_array().unwrap().len(), 5);
    }

    #[test]
    fn run_with_empty_queue_sends_nothing() {
        let mut replayer = ServerReplayer::new();
        let sender = MockSender::default();
        replayer.run(&sender);
        assert!(sender.sent.lock().unwrap().is_empty());
    }
}
