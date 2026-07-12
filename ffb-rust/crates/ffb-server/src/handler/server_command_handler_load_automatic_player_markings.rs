/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerLoadAutomaticPlayerMarkings.
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use ffb_model::enums::NetCommandId;
use ffb_model::model::Game;
use ffb_protocol::commands::client_command_load_automatic_player_markings::ClientCommandLoadAutomaticPlayerMarkings;
use crate::model::received_command::{ReceivedCommand, SessionId};
use crate::net::commands::any_internal_server_command::AnyInternalServerCommand;
use crate::net::commands::internal_server_command_calculate_automatic_player_markings::InternalServerCommandCalculateAutomaticPlayerMarkings;
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
    dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
}

/// A `ServerRequest` adapter around [`FumbblRequestLoadPlayerMarkingsForGameVersion`].
///
/// Java's `FumbblRequestLoadPlayerMarkingsForGameVersion.process(ServerRequestProcessor)`
/// fetches the auto-marking config (via `AbstractFumbblRequestLoadPlayerMarkings`) and then
/// dispatches an `InternalServerCommandCalculateAutomaticPlayerMarkings` with the result. That
/// command already exists and is wired into `ServerCommandHandlerFactory`'s dispatch, so this
/// adapter redispatches it for real once the config is fetched; if the client command carried
/// no `game` (see this file's `build_request` doc comment on why that field can be absent),
/// there is nothing to build the command from and the fetch result is dropped, matching Java's
/// behavior of operating on the game the client-supplied command actually carried.
struct QueuedLoadAutomaticPlayerMarkingsRequest {
    request: FumbblRequestLoadPlayerMarkingsForGameVersion,
    client: Arc<dyn HttpClient + Send + Sync>,
    markings_url_template: String,
    game: Option<Game>,
    dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
    session_id: SessionId,
}

impl ServerRequest for QueuedLoadAutomaticPlayerMarkingsRequest {
    fn process(&self) -> Result<(), String> {
        let mut request = FumbblRequestLoadPlayerMarkingsForGameVersion::new(
            self.request.get_index(),
            self.request.get_coach().to_string(),
        );
        let config = request.process(self.client.as_ref(), &self.markings_url_template, None)?;
        if let (Some(config), Some(game)) = (config, self.game.clone()) {
            let cmd = InternalServerCommandCalculateAutomaticPlayerMarkings::new(config, request.get_index(), game);
            let _ = self.dispatch_tx.send(ReceivedCommand::new_internal(
                AnyInternalServerCommand::CalculateAutomaticPlayerMarkings(cmd),
                self.session_id,
            ));
        }
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
        dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
    ) -> Self {
        Self { request_processor, client, markings_url_template: markings_url_template.into(), dispatch_tx }
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
        let request = Self::build_request(command);
        let queued = QueuedLoadAutomaticPlayerMarkingsRequest {
            request,
            client: Arc::clone(&self.client),
            markings_url_template: self.markings_url_template.clone(),
            game: command.get_game().cloned(),
            dispatch_tx: self.dispatch_tx.clone(),
            session_id,
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
        let (dispatch_tx, _dispatch_rx) = mpsc::unbounded_channel();
        ServerCommandHandlerLoadAutomaticPlayerMarkings::new(
            Arc::new(Mutex::new(ServerRequestProcessor::new())),
            Arc::new(client),
            "http://fumbbl/markings/$1",
            dispatch_tx,
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
        let (dispatch_tx, _dispatch_rx) = mpsc::unbounded_channel();
        let h = ServerCommandHandlerLoadAutomaticPlayerMarkings::new(
            Arc::clone(&processor),
            Arc::new(MockHttpClient { response: Ok("{}".to_string()) }),
            "http://fumbbl/markings/$1",
            dispatch_tx,
        );
        let command = ClientCommandLoadAutomaticPlayerMarkings { entropy: None, index: 1, coach: Some("Coach".to_string()), game: None };
        assert!(h.handle_command(&command, 1));
        assert_eq!(processor.lock().unwrap().queue_len(), 1);
    }

    #[test]
    fn queued_request_processes_and_fetches_config_over_http() {
        let processor = Arc::new(Mutex::new(ServerRequestProcessor::new()));
        let (dispatch_tx, _dispatch_rx) = mpsc::unbounded_channel();
        let h = ServerCommandHandlerLoadAutomaticPlayerMarkings::new(
            Arc::clone(&processor),
            Arc::new(MockHttpClient { response: Ok("{\"markings\":[]}".to_string()) }),
            "http://fumbbl/markings/$1",
            dispatch_tx,
        );
        let command = ClientCommandLoadAutomaticPlayerMarkings { entropy: None, index: 1, coach: Some("Coach".to_string()), game: None };
        assert!(h.handle_command(&command, 1));
        assert!(processor.lock().unwrap().run().is_ok());
        assert_eq!(processor.lock().unwrap().queue_len(), 0);
    }

    #[test]
    fn queued_request_with_game_dispatches_calculate_command() {
        let processor = Arc::new(Mutex::new(ServerRequestProcessor::new()));
        let (dispatch_tx, mut dispatch_rx) = mpsc::unbounded_channel();
        let h = ServerCommandHandlerLoadAutomaticPlayerMarkings::new(
            Arc::clone(&processor),
            Arc::new(MockHttpClient {
                response: Ok(r#"{"autoMarkingSeparator":"-","autoMarkingRecords":[]}"#.to_string()),
            }),
            "http://fumbbl/markings/$1",
            dispatch_tx,
        );
        let game = ffb_model::model::game::Game::new(
            ffb_model::model::team::Team {
                id: "home".into(), name: "home".into(), race: "Human".into(), roster_id: "human".into(),
                coach: "c".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
                prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
                assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
                special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
            },
            ffb_model::model::team::Team {
                id: "away".into(), name: "away".into(), race: "Human".into(), roster_id: "human".into(),
                coach: "c".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
                prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
                assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
                special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
            },
            ffb_model::enums::Rules::Bb2025,
        );
        let command = ClientCommandLoadAutomaticPlayerMarkings {
            entropy: None, index: 2, coach: Some("Coach".to_string()), game: Some(game),
        };
        assert!(h.handle_command(&command, 9));
        assert!(processor.lock().unwrap().run().is_ok());

        let received = dispatch_rx.try_recv().expect("expected a redispatched CalculateAutomaticPlayerMarkings command");
        assert_eq!(received.session_id, 9);
        match received.command {
            crate::model::received_command::ReceivedNetCommand::Internal(
                AnyInternalServerCommand::CalculateAutomaticPlayerMarkings(cmd),
            ) => {
                assert_eq!(cmd.get_index(), 2);
                assert_eq!(cmd.get_auto_marking_config().get_separator(), "-");
                assert_eq!(cmd.get_game().team_home.id, "home");
            }
            _ => panic!("expected an internal CalculateAutomaticPlayerMarkings command"),
        }
    }
}
