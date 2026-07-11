/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerStatLive.
/// Live variant of TalkHandlerStat — uses DecoratingCommandAdapter, SPEC client, EDIT_STATE privilege.
use super::talk_handler_stat::TalkHandlerStat;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerStatLive;

impl TalkHandlerStatLive {
    /// Java: `super(new DecoratingCommandAdapter(), Client.SPEC, Environment.NONE, Privilege.EDIT_STATE)`.
    pub fn new() -> TalkHandlerStat {
        TalkHandlerStat::new(Client::Spec, Environment::None, vec![Privilege::EditState])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_has_live_requirements() {
        let h = TalkHandlerStatLive::new();
        assert_eq!(h.required_client, Client::Spec);
        assert_eq!(h.requires_one_privilege_of, vec![Privilege::EditState]);
    }
}
