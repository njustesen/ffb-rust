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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_has_test_requirements() {
        let h = TalkHandlerSetBallTest::new();
        assert_eq!(h.required_client, Client::Player);
        assert_eq!(h.required_environment, Environment::TestGame);
        assert!(h.requires_one_privilege_of.is_empty());
    }
}
