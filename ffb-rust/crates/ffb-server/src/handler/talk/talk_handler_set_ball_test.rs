/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerSetBallTest.
/// Test variant of TalkHandlerSetBall — uses IdentityCommandAdapter, PLAYER client, TEST_GAME env.
use super::talk_handler_set_ball::TalkHandlerSetBall;
use super::talk_requirements::{Client, Environment};

pub struct TalkHandlerSetBallTest;

impl TalkHandlerSetBallTest {
    /// Java: `super(new IdentityCommandAdapter(), Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> TalkHandlerSetBall {
        TalkHandlerSetBall::new(Client::Player, Environment::TestGame, Vec::new())
    }
}
