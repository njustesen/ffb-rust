/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerWeatherTest.
/// Test variant of TalkHandlerWeather — uses IdentityCommandAdapter, PLAYER client, TEST_GAME env.
use super::talk_handler_weather::TalkHandlerWeather;
use super::talk_requirements::{Client, Environment};

pub struct TalkHandlerWeatherTest;

impl TalkHandlerWeatherTest {
    /// Java: `super(new IdentityCommandAdapter(), Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> TalkHandlerWeather {
        TalkHandlerWeather::new(Client::Player, Environment::TestGame, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_has_test_requirements() {
        let h = TalkHandlerWeatherTest::new();
        assert_eq!(h.required_client, Client::Player);
        assert_eq!(h.required_environment, Environment::TestGame);
    }
}
