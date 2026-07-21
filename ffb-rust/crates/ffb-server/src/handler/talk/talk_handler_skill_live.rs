/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerSkillLive.
/// Live variant of TalkHandlerSkill — uses DecoratingCommandAdapter, SPEC client, EDIT_STATE privilege.
use super::talk_handler_skill::TalkHandlerSkill;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerSkillLive;

impl TalkHandlerSkillLive {
    /// Java: `super(new DecoratingCommandAdapter(), Client.SPEC, Environment.NONE, Privilege.EDIT_STATE)`.
    pub fn new() -> TalkHandlerSkill {
        TalkHandlerSkill::new(Client::Spec, Environment::None, vec![Privilege::EditState])
    }
}
