/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerStunLive.
/// Live variant of TalkHandlerStun — uses DecoratingCommandAdapter, SPEC client, EDIT_STATE privilege.
use super::talk_handler_stun::TalkHandlerStun;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerStunLive;

impl TalkHandlerStunLive {
    /// Java: `super(new DecoratingCommandAdapter(), Client.SPEC, Environment.NONE, Privilege.EDIT_STATE)`.
    pub fn new() -> TalkHandlerStun {
        TalkHandlerStun::new(Client::Spec, Environment::None, vec![Privilege::EditState])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_has_live_requirements() {
        let h = TalkHandlerStunLive::new();
        assert_eq!(h.required_client, Client::Spec);
        assert_eq!(h.requires_one_privilege_of, vec![Privilege::EditState]);
    }
}
