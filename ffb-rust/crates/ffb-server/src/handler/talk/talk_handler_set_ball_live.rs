/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerSetBallLive.
/// Live variant of TalkHandlerSetBall — uses DecoratingCommandAdapter, SPEC client, EDIT_STATE privilege.
use super::talk_handler_set_ball::TalkHandlerSetBall;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerSetBallLive;

impl TalkHandlerSetBallLive {
    /// Java: `super(new DecoratingCommandAdapter(), Client.SPEC, Environment.NONE, Privilege.EDIT_STATE)`.
    pub fn new() -> TalkHandlerSetBall {
        TalkHandlerSetBall::new(Client::Spec, Environment::None, vec![Privilege::EditState])
    }
}
