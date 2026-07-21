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
