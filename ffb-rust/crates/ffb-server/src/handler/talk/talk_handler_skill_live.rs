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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_has_live_requirements() {
        let h = TalkHandlerSkillLive::new();
        assert_eq!(h.required_client, Client::Spec);
        assert_eq!(h.requires_one_privilege_of, vec![Privilege::EditState]);
    }
}
