/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerLoadAutomaticPlayerMarkings.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_load_automatic_player_markings::ClientCommandLoadAutomaticPlayerMarkings;
use crate::model::received_command::SessionId;
use crate::request::fumbbl::fumbbl_request_load_player_markings_for_game_version::FumbblRequestLoadPlayerMarkingsForGameVersion;
use crate::request::fumbbl::util_fumbbl_request::HttpClient;
use crate::request::server_request::ServerRequest;
use crate::request::server_request_processor::ServerRequestProcessor;

pub struct ServerCommandHandlerLoadAutomaticPlayerMarkings {
    /// `pub(crate)` (rather than private) so `ServerCommandHandlerFactory`'s own tests can
    /// observe the enqueued request without a network round trip.
    pub(crate) request_processor: Arc<Mutex<ServerRequestProcessor>>,
    client: Arc<dyn HttpClient + Send + Sync>,
    markings_url_template: String,
}

/// A `ServerRequest` adapter around [`FumbblRequestLoadPlayerMarkingsForGameVersion`].
///
/// Java's `FumbblRequestLoadPlayerMarkingsForGameVersion.process(ServerRequestProcessor)`
/// fetches the auto-marking config (via `AbstractFumbblRequestLoadPlayerMarkings`) and then
/// dispatches an `InternalServerCommandCalculateAutomaticPlayerMarkings` with the result — the
/// session/command plumbing for that dispatch has no equivalent in this crate yet, so this
/// adapter performs the real HTTP fetch (the portable piece) and discards the response, matching
/// how other narrowly-gated handlers in this phase document a still-missing tail step.
struct QueuedLoadAutomaticPlayerMarkingsRequest {
    request: FumbblRequestLoadPlayerMarkingsForGameVersion,
    client: Arc<dyn HttpClient + Send + Sync>,
    markings_url_template: String,
}

impl ServerRequest for QueuedLoadAutomaticPlayerMarkingsRequest {
    fn process(&self) -> Result<(), String> {
        let mut request = FumbblRequestLoadPlayerMarkingsForGameVersion::new(
            self.request.get_index(),
            self.request.get_coach().to_string(),
        );
        request.process(self.client.as_ref(), &self.markings_url_template, None)?;
        Ok(())
    }

    fn get_request_url(&self) -> &str {
        self.request.get_request_url()
    }

    fn set_request_url(&mut self, url: String) {
        self.request.set_request_url(url);
    }
}

impl ServerCommandHandlerLoadAutomaticPlayerMarkings {
    pub fn new(
        request_processor: Arc<Mutex<ServerRequestProcessor>>,
        client: Arc<dyn HttpClient + Send + Sync>,
        markings_url_template: impl Into<String>,
    ) -> Self {
        Self { request_processor, client, markings_url_template: markings_url_template.into() }
    }

    /// Java: getId() — returns NetCommandId for LOAD_AUTOMATIC_PLAYER_MARKINGS.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientLoadAutomaticPlayerMarkings
    }

    /// Java: `build_request` step of `handleCommand`:
    /// `new FumbblRequestLoadPlayerMarkingsForGameVersion(command.getGame(), command.getIndex(),
    /// command.getCoach(), receivedCommand.getSession())`.
    ///
    /// (The Rust `ClientCommandLoadAutomaticPlayerMarkings` omits the `game` field — see its own
    /// doc comment — and `FumbblRequestLoadPlayerMarkingsForGameVersion`'s constructor was ported
    /// to match, taking only `index` and `coach`.)
    fn build_request(
        command: &ClientCommandLoadAutomaticPlayerMarkings,
    ) -> FumbblRequestLoadPlayerMarkingsForGameVersion {
        FumbblRequestLoadPlayerMarkingsForGameVersion::new(
            command.get_index(),
            command.get_coach().unwrap_or_default().to_string(),
        )
    }

    /// Java: handleCommand(ReceivedCommand) — loads automatic player markings from storage.
    /// ```java
    /// getServer().getRequestProcessor().add(new FumbblRequestLoadPlayerMarkingsForGameVersion(
    ///     command.getGame(), command.getIndex(), command.getCoach(), receivedCommand.getSession()));
    /// return true;
    /// ```
    pub fn handle_command(&self, command: &ClientCommandLoadAutomaticPlayerMarkings, session_id: SessionId) -> bool {
        let _ = session_id;
        let request = Self::build_request(command);
        let queued = QueuedLoadAutomaticPlayerMarkingsRequest {
            request,
            client: Arc::clone(&self.client),
            markings_url_template: self.markings_url_template.clone(),
        };
        self.request_processor.lock().unwrap().add(Box::new(queued));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    fn handler_with_client(client: MockHttpClient) -> ServerCommandHandlerLoadAutomaticPlayerMarkings {
        ServerCommandHandlerLoadAutomaticPlayerMarkings::new(
            Arc::new(Mutex::new(ServerRequestProcessor::new())),
            Arc::new(client),
            "http://fumbbl/markings/$1",
        )
    }

    #[test]
    fn construct() {
        let _ = handler_with_client(MockHttpClient { response: Ok(String::new()) });
    }

    #[test]
    fn get_id_is_load_automatic_player_markings() {
        let h = handler_with_client(MockHttpClient { response: Ok(String::new()) });
        assert_eq!(h.get_id(), NetCommandId::ClientLoadAutomaticPlayerMarkings);
    }

    #[test]
    fn build_request_carries_index_and_coach() {
        let command = ClientCommandLoadAutomaticPlayerMarkings {
            entropy: None,
            index: 5,
            coach: Some("CoachA".to_string()),
            game: None,
        };
        let request = ServerCommandHandlerLoadAutomaticPlayerMarkings::build_request(&command);
        assert_eq!(request.get_index(), 5);
        assert_eq!(request.get_coach(), "CoachA");
    }

    #[test]
    fn build_request_defaults_missing_coach_to_empty() {
        let command = ClientCommandLoadAutomaticPlayerMarkings { entropy: None, index: 0, coach: None, game: None };
        let request = ServerCommandHandlerLoadAutomaticPlayerMarkings::build_request(&command);
        assert_eq!(request.get_coach(), "");
    }

    #[test]
    fn handle_command_enqueues_a_request() {
        let processor = Arc::new(Mutex::new(ServerRequestProcessor::new()));
        let h = ServerCommandHandlerLoadAutomaticPlayerMarkings::new(
            Arc::clone(&processor),
            Arc::new(MockHttpClient { response: Ok("{}".to_string()) }),
            "http://fumbbl/markings/$1",
        );
        let command = ClientCommandLoadAutomaticPlayerMarkings { entropy: None, index: 1, coach: Some("Coach".to_string()), game: None };
        assert!(h.handle_command(&command, 1));
        assert_eq!(processor.lock().unwrap().queue_len(), 1);
    }

    #[test]
    fn queued_request_processes_and_fetches_config_over_http() {
        let processor = Arc::new(Mutex::new(ServerRequestProcessor::new()));
        let h = ServerCommandHandlerLoadAutomaticPlayerMarkings::new(
            Arc::clone(&processor),
            Arc::new(MockHttpClient { response: Ok("{\"markings\":[]}".to_string()) }),
            "http://fumbbl/markings/$1",
        );
        let command = ClientCommandLoadAutomaticPlayerMarkings { entropy: None, index: 1, coach: Some("Coach".to_string()), game: None };
        assert!(h.handle_command(&command, 1));
        assert!(processor.lock().unwrap().run().is_ok());
        assert_eq!(processor.lock().unwrap().queue_len(), 0);
    }
}
