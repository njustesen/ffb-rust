/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerStunTest.
/// Test variant of TalkHandlerStun — uses IdentityCommandAdapter, PLAYER client, TEST_GAME env.
use super::talk_handler_stun::TalkHandlerStun;
use super::talk_requirements::{Client, Environment};

pub struct TalkHandlerStunTest;

impl TalkHandlerStunTest {
    /// Java: `super(new IdentityCommandAdapter(), Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> TalkHandlerStun {
        TalkHandlerStun::new(Client::Player, Environment::TestGame, Vec::new())
    }
}
