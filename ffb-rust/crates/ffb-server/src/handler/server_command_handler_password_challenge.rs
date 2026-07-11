/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerPasswordChallenge.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_password_challenge::ClientCommandPasswordChallenge;
use ffb_protocol::server_commands::{ServerCommand, ServerPasswordChallenge};
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;
use crate::request::fumbbl::fumbbl_request_password_challenge::FumbblRequestPasswordChallenge;
use crate::request::fumbbl::util_fumbbl_request::HttpClient;

/// Java: `ServerCommandHandlerPasswordChallenge extends ServerCommandHandler`.
///
/// Java gates on `ServerMode.FUMBBL == getServer().getMode()`. The Rust
/// server has no `ServerMode` type wired up yet (see
/// `db/db_query_factory.rs`'s `is_standalone()` comment), so the mode is
/// threaded through explicitly as `fumbbl_mode`, defaulting to `true` to
/// match the codebase's documented FUMBBL-mode default.
pub struct ServerCommandHandlerPasswordChallenge {
    session_manager: Arc<Mutex<SessionManager>>,
    fumbbl_mode: bool,
    /// Java: `ServerUrlProperty.FUMBBL_AUTH_CHALLENGE.url(server.getProperties())` — the
    /// `$1`-templated challenge URL. No properties-file layer exists yet (see
    /// `fantasy_football_server.rs`'s env-var config), so it is threaded through explicitly.
    challenge_url_template: String,
}

impl ServerCommandHandlerPasswordChallenge {
    pub fn new(session_manager: Arc<Mutex<SessionManager>>) -> Self {
        Self { session_manager, fumbbl_mode: true, challenge_url_template: String::new() }
    }

    pub fn with_fumbbl_mode(session_manager: Arc<Mutex<SessionManager>>, fumbbl_mode: bool) -> Self {
        Self { session_manager, fumbbl_mode, challenge_url_template: String::new() }
    }

    pub fn with_challenge_url_template(
        session_manager: Arc<Mutex<SessionManager>>,
        fumbbl_mode: bool,
        challenge_url_template: impl Into<String>,
    ) -> Self {
        Self { session_manager, fumbbl_mode, challenge_url_template: challenge_url_template.into() }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_PASSWORD_CHALLENGE`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientPasswordChallenge
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    ///
    /// ```java
    /// String challenge = null;
    /// if ((ServerMode.FUMBBL == getServer().getMode()) && StringTool.isProvided(passwordChallengeCommand.getCoach())) {
    ///     getServer().getRequestProcessor().add(new FumbblRequestPasswordChallenge(passwordChallengeCommand.getCoach(), pReceivedCommand.getSession()));
    /// } else {
    ///     getServer().getCommunication().sendPasswordChallenge(pReceivedCommand.getSession(), challenge);
    /// }
    /// return true;
    /// ```
    ///
    /// `FumbblRequestPasswordChallenge::process` (Java: `FumbblRequestPasswordChallenge.process`)
    /// fetches the challenge over HTTP and itself calls `communication.sendPasswordChallenge` at
    /// the end — since there is no `ServerRequestProcessor` queue in this crate yet, that same
    /// fetch-then-send is performed inline here instead of being enqueued.
    pub fn handle_command(
        &self,
        password_challenge_command: &ClientCommandPasswordChallenge,
        session_id: SessionId,
        client: &dyn HttpClient,
    ) -> bool {
        let coach_provided = password_challenge_command
            .get_coach()
            .map(|c| !c.is_empty())
            .unwrap_or(false);

        let challenge: Option<String> = if self.fumbbl_mode && coach_provided {
            let coach = password_challenge_command.get_coach().unwrap_or_default().to_string();
            let mut request = FumbblRequestPasswordChallenge::new(coach);
            // Java: `IOException` -> `FantasyFootballException` (propagated); here a failed
            // fetch degrades to `challenge = null`, matching the "no response" branch of the
            // same method (StringTool.isProvided(responseXml) == false).
            request.process(client, &self.challenge_url_template).unwrap_or(None)
        } else {
            None
        };

        let command = ServerCommand::ServerPasswordChallenge(ServerPasswordChallenge {
            challenge: challenge.unwrap_or_default(),
        });
        if let Ok(json) = serde_json::to_string(&command) {
            self.session_manager.lock().unwrap().send_to(session_id, &json);
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;
    use tokio::sync::mpsc;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    fn setup_session() -> (Arc<Mutex<SessionManager>>, mpsc::UnboundedReceiver<String>) {
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, 100, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        (sm, rx)
    }

    #[test]
    fn construct() {
        let (sm, _rx) = setup_session();
        let _ = ServerCommandHandlerPasswordChallenge::new(sm);
    }

    #[test]
    fn get_id_is_client_password_challenge() {
        let (sm, _rx) = setup_session();
        let handler = ServerCommandHandlerPasswordChallenge::new(sm);
        assert_eq!(handler.get_id(), NetCommandId::ClientPasswordChallenge);
    }

    #[test]
    fn standalone_mode_sends_password_challenge_directly() {
        let (sm, mut rx) = setup_session();
        let handler = ServerCommandHandlerPasswordChallenge::with_fumbbl_mode(sm, false);
        let cmd = ClientCommandPasswordChallenge::with_coach("Coach");
        let client = MockHttpClient { response: Ok(String::new()) };
        assert!(handler.handle_command(&cmd, 1, &client));
        let msg = rx.try_recv().expect("expected a ServerPasswordChallenge message");
        assert!(msg.contains("serverPasswordChallenge"));
    }

    #[test]
    fn fumbbl_mode_without_coach_sends_directly() {
        let (sm, mut rx) = setup_session();
        let handler = ServerCommandHandlerPasswordChallenge::with_fumbbl_mode(sm, true);
        let cmd = ClientCommandPasswordChallenge::new();
        let client = MockHttpClient { response: Ok(String::new()) };
        assert!(handler.handle_command(&cmd, 1, &client));
        assert!(rx.try_recv().is_ok());
    }

    #[test]
    fn fumbbl_mode_with_coach_fetches_challenge_over_http() {
        let (sm, mut rx) = setup_session();
        let handler = ServerCommandHandlerPasswordChallenge::with_challenge_url_template(
            sm,
            true,
            "http://fumbbl/auth/challenge/$1",
        );
        let cmd = ClientCommandPasswordChallenge::with_coach("Coach");
        let client = MockHttpClient { response: Ok("<challenge>abc123</challenge>".to_string()) };
        assert!(handler.handle_command(&cmd, 1, &client));
        let msg = rx.try_recv().expect("expected a ServerPasswordChallenge message");
        assert!(msg.contains("abc123"));
    }

    #[test]
    fn fumbbl_mode_with_coach_and_http_error_sends_null_challenge() {
        let (sm, mut rx) = setup_session();
        let handler = ServerCommandHandlerPasswordChallenge::with_challenge_url_template(
            sm,
            true,
            "http://fumbbl/auth/challenge/$1",
        );
        let cmd = ClientCommandPasswordChallenge::with_coach("Coach");
        let client = MockHttpClient { response: Err("connection refused".to_string()) };
        assert!(handler.handle_command(&cmd, 1, &client));
        let msg = rx.try_recv().expect("expected a ServerPasswordChallenge message");
        assert!(msg.contains("\"challenge\":\"\""));
    }

}
