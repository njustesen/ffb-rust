/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerSetPlayerTest.
/// Test variant of TalkHandlerSetPlayer — uses IdentityCommandAdapter, PLAYER client, TEST_GAME env.
use super::talk_handler_set_player::TalkHandlerSetPlayer;
use super::talk_requirements::{Client, Environment};

pub struct TalkHandlerSetPlayerTest;

impl TalkHandlerSetPlayerTest {
    /// Java: `super(new IdentityCommandAdapter(), Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> TalkHandlerSetPlayer {
        TalkHandlerSetPlayer::new(Client::Player, Environment::TestGame, Vec::new())
    }
}
