/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerSetPlayerLive.
/// Live variant of TalkHandlerSetPlayer — uses DecoratingCommandAdapter, SPEC client, EDIT_STATE privilege.
use super::talk_handler_set_player::TalkHandlerSetPlayer;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerSetPlayerLive;

impl TalkHandlerSetPlayerLive {
    /// Java: `super(new DecoratingCommandAdapter(), Client.SPEC, Environment.NONE, Privilege.EDIT_STATE)`.
    pub fn new() -> TalkHandlerSetPlayer {
        TalkHandlerSetPlayer::new(Client::Spec, Environment::None, vec![Privilege::EditState])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_has_live_requirements() {
        let h = TalkHandlerSetPlayerLive::new();
        assert_eq!(h.required_client, Client::Spec);
        assert_eq!(h.requires_one_privilege_of, vec![Privilege::EditState]);
    }
}
