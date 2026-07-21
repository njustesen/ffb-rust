/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerUsedActionsTest.
/// Test variant of TalkHandlerUsedActions — uses IdentityCommandAdapter, PLAYER client, TEST_GAME env.
use super::talk_handler_used_actions::TalkHandlerUsedActions;
use super::talk_requirements::{Client, Environment};

pub struct TalkHandlerUsedActionsTest;

impl TalkHandlerUsedActionsTest {
    /// Java: `super(new IdentityCommandAdapter(), Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> TalkHandlerUsedActions {
        TalkHandlerUsedActions::new(Client::Player, Environment::TestGame, Vec::new())
    }
}
