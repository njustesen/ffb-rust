/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerStandupTest.
/// Test variant of TalkHandlerStandup — uses IdentityCommandAdapter, PLAYER client, TEST_GAME env.
use super::talk_handler_standup::TalkHandlerStandup;
use super::talk_requirements::{Client, Environment};

pub struct TalkHandlerStandupTest;

impl TalkHandlerStandupTest {
    /// Java: `super(new IdentityCommandAdapter(), Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> TalkHandlerStandup {
        TalkHandlerStandup::new(Client::Player, Environment::TestGame, Vec::new())
    }
}
