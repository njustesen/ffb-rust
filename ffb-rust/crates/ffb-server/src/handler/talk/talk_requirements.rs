/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkRequirements.
/// Contains three nested requirement enums for talk command access control.
use crate::net::session_manager::SessionManager;
use crate::model::received_command::SessionId;

/// Java: TalkRequirements.Client — session type requirement (NONE / PLAYER / SPEC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Client {
    None,
    Player,
    Spec,
}

impl Client {
    /// Java: isMet(SessionManager, long, Session) — checks session type.
    pub fn is_met(&self, session_manager: &SessionManager, game_id: i64, session: SessionId) -> bool {
        match self {
            Client::None => true,
            Client::Player => Self::has_player_session(session_manager, game_id, session),
            Client::Spec => !Self::has_player_session(session_manager, game_id, session),
        }
    }

    /// Java: hasPlayerSession(SessionManager, long, Session)
    fn has_player_session(session_manager: &SessionManager, game_id: i64, session: SessionId) -> bool {
        session_manager.get_session_of_home_coach(game_id) == Some(session)
            || session_manager.get_session_of_away_coach(game_id) == Some(session)
    }
}

/// Java: TalkRequirements.Environment — game environment requirement (NONE / TEST_GAME / TEST_SERVER).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    None,
    TestGame,
    TestServer,
}

impl Environment {
    /// Java: isMet(FantasyFootballServer, GameState) — checks server/game test mode.
    ///
    /// Java reads `server.getProperty(IServerProperty.SERVER_TEST)`; the Rust
    /// `FantasyFootballServer` has no property store wired yet, so the caller
    /// passes the resolved `server_test_mode` flag directly.
    pub fn is_met(&self, server_test_mode: bool, game_is_testing: bool) -> bool {
        match self {
            Environment::None => true,
            Environment::TestGame => game_is_testing || server_test_mode,
            Environment::TestServer => server_test_mode,
        }
    }
}

/// Java: TalkRequirements.Privilege — privilege requirement (EDIT_STATE / STAFF / DEV).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Privilege {
    EditState,
    Staff,
    Dev,
}

impl Privilege {
    /// Java: isMet(SessionManager, Session) — checks session privilege level.
    pub fn is_met(&self, session_manager: &SessionManager, session: SessionId) -> bool {
        match self {
            Privilege::EditState => session_manager.has_edit_privilege(session),
            Privilege::Staff => session_manager.is_session_admin(session),
            Privilege::Dev => session_manager.is_session_dev(session),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;

    fn setup() -> (SessionManager, SessionId, SessionId, SessionId) {
        let mut sm = SessionManager::new();
        let (tx1, _) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, _) = tokio::sync::mpsc::unbounded_channel();
        let (tx3, _) = tokio::sync::mpsc::unbounded_channel();
        sm.add_session(1, 100, "Home".into(), ClientMode::PLAYER, true, vec![], tx1);
        sm.add_session(2, 100, "Away".into(), ClientMode::PLAYER, false, vec![], tx2);
        sm.add_session(3, 100, "Spec".into(), ClientMode::SPECTATOR, false, vec!["DEV".into(), "STATE_EDIT".into()], tx3);
        (sm, 1, 2, 3)
    }

    #[test]
    fn construct() {
        let _ = Client::None;
        let _ = Environment::None;
        let _ = Privilege::EditState;
    }

    #[test]
    fn client_none_always_met() {
        let (sm, home, _away, _spec) = setup();
        assert!(Client::None.is_met(&sm, 100, home));
    }

    #[test]
    fn client_player_requires_home_or_away_session() {
        let (sm, home, away, spec) = setup();
        assert!(Client::Player.is_met(&sm, 100, home));
        assert!(Client::Player.is_met(&sm, 100, away));
        assert!(!Client::Player.is_met(&sm, 100, spec));
    }

    #[test]
    fn client_spec_requires_non_player_session() {
        let (sm, home, _away, spec) = setup();
        assert!(!Client::Spec.is_met(&sm, 100, home));
        assert!(Client::Spec.is_met(&sm, 100, spec));
    }

    #[test]
    fn environment_test_game_checks_game_or_server_flag() {
        assert!(Environment::TestGame.is_met(false, true));
        assert!(Environment::TestGame.is_met(true, false));
        assert!(!Environment::TestGame.is_met(false, false));
    }

    #[test]
    fn environment_test_server_ignores_game_flag() {
        assert!(Environment::TestServer.is_met(true, false));
        assert!(!Environment::TestServer.is_met(false, true));
    }

    #[test]
    fn privilege_checks_delegate_to_session_manager() {
        let (sm, home, _away, spec) = setup();
        assert!(!Privilege::EditState.is_met(&sm, home));
        assert!(Privilege::EditState.is_met(&sm, spec));
        assert!(Privilege::Dev.is_met(&sm, spec));
        assert!(!Privilege::Dev.is_met(&sm, home));
    }
}
