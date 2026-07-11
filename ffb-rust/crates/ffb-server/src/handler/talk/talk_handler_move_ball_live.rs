/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerMoveBallLive.
/// Live variant of TalkHandlerMoveBall — uses IdentityCommandAdapter, SPEC client, EDIT_STATE privilege.
use std::collections::HashSet;
use ffb_model::model::field_model::FieldModel;
use crate::handler::talk::identity_command_adapter::IdentityCommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_handler_move_ball::TalkHandlerMoveBall;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;

pub struct TalkHandlerMoveBallLive {
    base: TalkHandlerMoveBall,
}

impl TalkHandlerMoveBallLive {
    /// Java: `TalkHandlerMoveBallLive()`.
    pub fn new() -> Self {
        let adapter = IdentityCommandAdapter::new();
        let mut privileges = HashSet::new();
        privileges.insert(Privilege::EditState);
        Self {
            base: TalkHandlerMoveBall::new(&adapter, Client::Spec, Environment::None, privileges),
        }
    }

    pub fn base(&self) -> &TalkHandler { self.base.base() }

    /// Java: `handle` — delegates to TalkHandlerMoveBall with live game settings.
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

impl Default for TalkHandlerMoveBallLive {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::FieldCoordinate;

    #[test]
    fn construct() { let _ = TalkHandlerMoveBallLive::new(); }

    #[test]
    fn handle_delegates_to_base_logic() {
        let h = TalkHandlerMoveBallLive::new();
        let sm = SessionManager::new();
        let mut fm = FieldModel::default();
        fm.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        let commands = vec!["/move_ball".to_string(), "South".to_string(), "1".to_string()];
        let info = h.handle(&mut fm, &commands, &sm, 100, 1);
        assert_eq!(fm.ball_coordinate, Some(FieldCoordinate::new(5, 6)));
        assert!(!info.is_empty());
    }
}
