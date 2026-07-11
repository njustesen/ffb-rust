/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerTurnModelTest.
/// Test variant of TalkHandlerTurnMode — uses IdentityCommandAdapter, PLAYER client, TEST_GAME env.
use super::talk_handler_turn_mode::TalkHandlerTurnMode;
use super::talk_requirements::{Client, Environment};

pub struct TalkHandlerTurnModelTest;

impl TalkHandlerTurnModelTest {
    /// Java: `super(new IdentityCommandAdapter(), Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> TalkHandlerTurnMode {
        TalkHandlerTurnMode::new(Client::Player, Environment::TestGame, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_has_test_requirements() {
        let h = TalkHandlerTurnModelTest::new();
        assert_eq!(h.required_client, Client::Player);
        assert_eq!(h.required_environment, Environment::TestGame);
    }
}
