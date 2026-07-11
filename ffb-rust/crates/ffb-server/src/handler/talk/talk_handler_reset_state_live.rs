/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerResetStateLive.
/// Handles `/reset_state` command — resets step stack and game state (EDIT_STATE privilege, live only).
use crate::admin::game_state_service::GameStateService;
use crate::game_state::GameState;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerResetStateLive {
    pub required_client: Client,
    pub required_environment: Environment,
    pub requires_one_privilege_of: Vec<Privilege>,
    game_state_service: GameStateService,
}

impl TalkHandlerResetStateLive {
    pub const COMMAND: &'static str = "/reset_state";
    pub const COMMAND_PARTS_THRESHOLD: usize = 0;

    /// Java: `super("/reset_state", 0, new IdentityCommandAdapter(), Client.SPEC, Environment.NONE, Privilege.EDIT_STATE)`.
    pub fn new() -> Self {
        Self {
            required_client: Client::Spec,
            required_environment: Environment::None,
            requires_one_privilege_of: vec![Privilege::EditState],
            game_state_service: GameStateService::new(),
        }
    }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` — resets
    /// the step stack via `GameStateService.resetStepStack` and reports what was reset.
    pub fn handle(&self, game_state: &mut GameState) -> Result<String, String> {
        self.game_state_service.reset_step_stack(game_state)?;
        Ok(Self::reset_message())
    }

    /// Java: the literal info string passed to `server.getCommunication().sendPlayerTalk(...)`.
    /// Split out so the message content is testable without the not-yet-wired game state.
    pub fn reset_message() -> String {
        "Reset done:\n  - Acting player\n  - Player action\n  - Step stack cleared and init sequence pushed\n  - TurnMode set to regular\n  - Last TurnMode deleted\n  - New PassState set\n  - Target selection reset (blitz and gaze)\n  - Blitz turn data deleted".to_string()
    }
}

impl Default for TalkHandlerResetStateLive {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let h = TalkHandlerResetStateLive::new();
        assert_eq!(h.required_client, Client::Spec);
        assert_eq!(h.requires_one_privilege_of, vec![Privilege::EditState]);
    }

    #[test]
    fn reset_message_lists_all_reset_items() {
        let msg = TalkHandlerResetStateLive::reset_message();
        assert!(msg.contains("Acting player"));
        assert!(msg.contains("TurnMode set to regular"));
        assert!(msg.contains("Blitz turn data deleted"));
    }

    #[test]
    fn handle_errors_when_game_not_started() {
        let h = TalkHandlerResetStateLive::new();
        let mut gs = GameState::new(1);
        assert!(h.handle(&mut gs).is_err());
    }
}
