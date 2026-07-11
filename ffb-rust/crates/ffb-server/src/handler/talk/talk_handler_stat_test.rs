/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerStatTest.
/// Test variant of TalkHandlerStat — uses IdentityCommandAdapter, PLAYER client, TEST_GAME env.
use super::talk_handler_stat::TalkHandlerStat;
use super::talk_requirements::{Client, Environment};

pub struct TalkHandlerStatTest;

impl TalkHandlerStatTest {
    /// Java: `super(new IdentityCommandAdapter(), Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> TalkHandlerStat {
        TalkHandlerStat::new(Client::Player, Environment::TestGame, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_has_test_requirements() {
        let h = TalkHandlerStatTest::new();
        assert_eq!(h.required_client, Client::Player);
        assert_eq!(h.required_environment, Environment::TestGame);
    }
}
