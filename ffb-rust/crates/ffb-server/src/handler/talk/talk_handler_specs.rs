/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerSpecs.
/// Handles `/specs` chat command — lists spectators or shows spec info.
use crate::net::session_manager::SessionManager;
use crate::model::received_command::SessionId;
use super::talk_requirements::{Client, Environment};

pub struct TalkHandlerSpecs {
    pub required_client: Client,
    pub required_environment: Environment,
}

impl TalkHandlerSpecs {
    /// Java: `super(Set.of(ChatCommand.SPECS.getCommand()), 0, Client.NONE, Environment.NONE)`.
    pub const COMMAND: &'static str = "/specs";
    pub const COMMAND_PARTS_THRESHOLD: usize = 0;

    pub fn new() -> Self {
        Self { required_client: Client::None, required_environment: Environment::None }
    }

    /// Java: `handle(...)` — determines whether the issuing session is itself a spectator
    /// (neither the home nor away coach) and delegates to `TalkHandler.handleSpecs`.
    pub fn handle(&self, session_manager: &SessionManager, game_id: i64, session: SessionId) -> Vec<String> {
        let issued_by_spec = session_manager.get_session_of_home_coach(game_id) != Some(session)
            && session_manager.get_session_of_away_coach(game_id) != Some(session);
        Self::handle_specs(session_manager, game_id, session, issued_by_spec)
    }

    /// Java: `TalkHandler.handleSpecs`. Duplicated locally because the abstract
    /// `TalkHandler` base class is owned by a concurrent translation pass.
    pub(super) fn handle_specs(
        session_manager: &SessionManager,
        game_id: i64,
        _session: SessionId,
        issued_by_spec: bool,
    ) -> Vec<String> {
        let mut spectators: Vec<String> = session_manager
            .get_sessions_of_spectators(game_id)
            .into_iter()
            .filter(|&sid| !session_manager.is_session_admin(sid))
            .filter_map(|sid| session_manager.get_coach_for_session(sid).map(str::to_string))
            .collect();
        spectators.sort_by_key(|s| s.to_lowercase());

        if spectators.is_empty() {
            return vec!["There are no spectators.".to_string()];
        }
        if issued_by_spec && spectators.len() == 1 {
            return vec!["You are the only spectator of this game.".to_string()];
        }
        let mut info = vec![format!("{} spectators are watching this game:", spectators.len())];
        info.extend(spectators);
        info
    }
}

impl Default for TalkHandlerSpecs {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;

    fn session_manager() -> (SessionManager, SessionId, SessionId, SessionId) {
        let mut sm = SessionManager::new();
        let (tx1, _) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, _) = tokio::sync::mpsc::unbounded_channel();
        let (tx3, _) = tokio::sync::mpsc::unbounded_channel();
        sm.add_session(1, 100, "Home".into(), ClientMode::PLAYER, true, vec![], tx1);
        sm.add_session(2, 100, "Away".into(), ClientMode::PLAYER, false, vec![], tx2);
        sm.add_session(3, 100, "Watcher".into(), ClientMode::SPECTATOR, false, vec![], tx3);
        (sm, 1, 2, 3)
    }

    #[test]
    fn construct() {
        let h = TalkHandlerSpecs::new();
        assert_eq!(h.required_client, Client::None);
    }

    #[test]
    fn handle_lists_no_spectators() {
        let mut sm = SessionManager::new();
        let (tx1, _) = tokio::sync::mpsc::unbounded_channel();
        sm.add_session(1, 100, "Home".into(), ClientMode::PLAYER, true, vec![], tx1);
        let h = TalkHandlerSpecs::new();
        assert_eq!(h.handle(&sm, 100, 1), vec!["There are no spectators.".to_string()]);
    }

    #[test]
    fn handle_reports_sole_spectator_when_issued_by_spec() {
        let (sm, _home, _away, spec) = session_manager();
        let h = TalkHandlerSpecs::new();
        let result = h.handle(&sm, 100, spec);
        assert_eq!(result, vec!["You are the only spectator of this game.".to_string()]);
    }

    #[test]
    fn handle_lists_spectators_when_issued_by_coach() {
        let (sm, home, _away, _spec) = session_manager();
        let h = TalkHandlerSpecs::new();
        let result = h.handle(&sm, 100, home);
        assert_eq!(result[0], "1 spectators are watching this game:");
        assert_eq!(result[1], "Watcher");
    }
}
