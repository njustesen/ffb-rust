/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerMoveBallTest.
/// Test variant of TalkHandlerMoveBall — uses IdentityCommandAdapter, PLAYER client, TEST_GAME env.
use ffb_model::model::field_model::FieldModel;
use crate::handler::talk::identity_command_adapter::IdentityCommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_handler_move_ball::TalkHandlerMoveBall;
use crate::handler::talk::talk_requirements::{Client, Environment};
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;

pub struct TalkHandlerMoveBallTest {
    base: TalkHandlerMoveBall,
}

impl TalkHandlerMoveBallTest {
    /// Java: `TalkHandlerMoveBallTest()`.
    pub fn new() -> Self {
        let adapter = IdentityCommandAdapter::new();
        Self {
            base: TalkHandlerMoveBall::new(&adapter, Client::Player, Environment::TestGame, Default::default()),
        }
    }

    pub fn base(&self) -> &TalkHandler { self.base.base() }

    /// Java: `handle` — delegates to TalkHandlerMoveBall with test game settings.
    pub fn handle(
        &self,
        field_model: &mut FieldModel,
        commands: &[String],
        session_manager: &SessionManager,
        game_id: i64,
        session: SessionId,
    ) -> Vec<String> {
        self.base.handle(field_model, commands, session_manager, game_id, session)
    }
}

impl Default for TalkHandlerMoveBallTest {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::FieldCoordinate;

    #[test]
    fn construct() { let _ = TalkHandlerMoveBallTest::new(); }

    #[test]
    fn handle_delegates_to_base_logic() {
        let h = TalkHandlerMoveBallTest::new();
        let sm = SessionManager::new();
        let mut fm = FieldModel::default();
        fm.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        let commands = vec!["/move_ball".to_string(), "North".to_string(), "3".to_string()];
        let info = h.handle(&mut fm, &commands, &sm, 100, 1);
        assert_eq!(fm.ball_coordinate, Some(FieldCoordinate::new(5, 2)));
        assert!(!info.is_empty());
    }
}
