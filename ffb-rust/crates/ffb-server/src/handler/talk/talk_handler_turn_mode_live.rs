/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerTurnModeLive.
/// Live variant of TalkHandlerTurnMode — uses IdentityCommandAdapter, SPEC client, EDIT_STATE privilege.
use super::talk_handler_turn_mode::TalkHandlerTurnMode;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerTurnModeLive;

impl TalkHandlerTurnModeLive {
    /// Java: `super(new IdentityCommandAdapter(), Client.SPEC, Environment.NONE, Privilege.EDIT_STATE)`.
    pub fn new() -> TalkHandlerTurnMode {
        TalkHandlerTurnMode::new(Client::Spec, Environment::None, vec![Privilege::EditState])
    }
}
