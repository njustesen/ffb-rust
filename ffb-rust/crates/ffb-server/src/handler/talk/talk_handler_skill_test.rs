/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerSkillTest.
/// Test variant of TalkHandlerSkill — uses IdentityCommandAdapter, PLAYER client, TEST_GAME env.
use super::talk_handler_skill::TalkHandlerSkill;
use super::talk_requirements::{Client, Environment};

pub struct TalkHandlerSkillTest;

impl TalkHandlerSkillTest {
    /// Java: `super(new IdentityCommandAdapter(), Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> TalkHandlerSkill {
        TalkHandlerSkill::new(Client::Player, Environment::TestGame, Vec::new())
    }
}
