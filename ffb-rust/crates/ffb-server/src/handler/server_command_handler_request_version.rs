/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerRequestVersion.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ffb_model::enums::NetCommandId;
use ffb_model::types::constants::VERSION;
use ffb_protocol::commands::server_command_version::ServerCommandVersion;

use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommandHandlerRequestVersion`.
pub struct ServerCommandHandlerRequestVersion {
    session_manager: Arc<Mutex<SessionManager>>,
    /// Java: `getServer().getPropertyKeys()` filtered to keys starting with `"client."`,
    /// paired with `getServer().getProperty(property)`. No dynamic server-property store is
    /// wired into `ffb-server` yet (Phase ZV), so the already-filtered map is supplied at
    /// construction instead of being resolved from a live property store.
    client_properties: HashMap<String, String>,
    /// Java: `isServerInTestMode()`, resolved once at construction for the same reason.
    is_test_server: bool,
}

impl ServerCommandHandlerRequestVersion {
    /// Java: `protected ServerCommandHandlerRequestVersion(FantasyFootballServer pServer)`.
    pub fn new(
        session_manager: Arc<Mutex<SessionManager>>,
        client_properties: HashMap<String, String>,
        is_test_server: bool,
    ) -> Self {
        Self {
            session_manager,
            client_properties,
            is_test_server,
        }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_REQUEST_VERSION`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientRequestVersion
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    pub fn handle_command(&self, session_id: SessionId) -> bool {
        // Java iterates `getServer().getPropertyKeys()` collecting the ones prefixed
        // `"client."`; `self.client_properties` already holds exactly that filtered set.
        // Java: `sendVersion(session, VERSION, VERSION, names, values, isServerInTestMode())`.
        let version_msg = ServerCommandVersion::new(VERSION, VERSION, self.client_properties.clone(), self.is_test_server)
            .to_json_value();

        self.session_manager
            .lock()
            .unwrap()
            .send_to(session_id, &version_msg.to_string());

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;
    use tokio::sync::mpsc;

    fn setup() -> (
        Arc<Mutex<SessionManager>>,
        mpsc::UnboundedReceiver<String>,
    ) {
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, rx) = mpsc::unbounded_channel();
        sm.lock()
            .unwrap()
            .add_session(1, 100, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        (sm, rx)
    }

    #[test]
    fn handle_command_sends_version_with_client_properties() {
        let (sm, mut rx) = setup();
        let mut props = HashMap::new();
        props.insert("client.showReplayButton".to_string(), "true".to_string());
        let handler = ServerCommandHandlerRequestVersion::new(sm, props, true);

        let ok = handler.handle_command(1);
        assert!(ok);

        let msg = rx.try_recv().expect("expected a version message");
        let value: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(value["netCommandId"], "serverVersion");
        assert_eq!(value["serverVersion"], VERSION);
        assert_eq!(value["clientVersion"], VERSION);
        assert_eq!(value["testing"], true);
        assert_eq!(value["clientPropertyNames"][0], "client.showReplayButton");
        assert_eq!(value["clientPropertyValues"][0], "true");
    }

    #[test]
    fn handle_command_reflects_test_mode_flag() {
        let (sm, mut rx) = setup();
        let handler = ServerCommandHandlerRequestVersion::new(sm, HashMap::new(), false);

        handler.handle_command(1);

        let msg = rx.try_recv().unwrap();
        let value: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(value["testing"], false);
        assert!(value["clientPropertyNames"].as_array().unwrap().is_empty());
    }
}
