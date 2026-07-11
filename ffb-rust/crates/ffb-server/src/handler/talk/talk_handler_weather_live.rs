/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerWeatherLive.
/// Live variant of TalkHandlerWeather — uses IdentityCommandAdapter, SPEC client, EDIT_STATE privilege.
use super::talk_handler_weather::TalkHandlerWeather;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerWeatherLive;

impl TalkHandlerWeatherLive {
    /// Java: `super(new IdentityCommandAdapter(), Client.SPEC, Environment.NONE, Privilege.EDIT_STATE)`.
    pub fn new() -> TalkHandlerWeather {
        TalkHandlerWeather::new(Client::Spec, Environment::None, vec![Privilege::EditState])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_has_live_requirements() {
        let h = TalkHandlerWeatherLive::new();
        assert_eq!(h.required_client, Client::Spec);
        assert_eq!(h.requires_one_privilege_of, vec![Privilege::EditState]);
    }
}
