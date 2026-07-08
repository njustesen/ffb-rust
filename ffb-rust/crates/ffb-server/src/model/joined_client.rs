/// 1:1 translation of the private inner class JoinedClient in
/// com.fumbbl.ffb.server.net.SessionManager.
use ffb_model::model::ClientMode;

#[derive(Debug, Clone)]
pub struct JoinedClient {
    /// Java: `fGameId`
    pub game_id: i64,
    /// Java: `fCoach`
    pub coach: String,
    /// Java: `fMode`
    pub mode: ClientMode,
    /// Java: `fHomeCoach`
    pub home_coach: bool,
    /// Java: `fAccountProperties`
    pub account_properties: Vec<String>,
}

impl JoinedClient {
    pub fn new(game_id: i64, coach: String, mode: ClientMode, home_coach: bool, account_properties: Vec<String>) -> Self {
        Self { game_id, coach, mode, home_coach, account_properties }
    }

    /// Java: `getGameId()`
    pub fn get_game_id(&self) -> i64 { self.game_id }

    /// Java: `getCoach()`
    pub fn get_coach(&self) -> &str { &self.coach }

    /// Java: `getMode()`
    pub fn get_mode(&self) -> ClientMode { self.mode }

    /// Java: `isHomeCoach()`
    pub fn is_home_coach(&self) -> bool { self.home_coach }

    /// Java: `hasProperty(String)`
    pub fn has_property(&self, property: &str) -> bool {
        self.account_properties.iter().any(|p| p == property)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_fields() {
        let c = JoinedClient::new(42, "coach".into(), ClientMode::PLAYER, true, vec!["DEV".into()]);
        assert_eq!(c.get_game_id(), 42);
        assert_eq!(c.get_coach(), "coach");
        assert_eq!(c.get_mode(), ClientMode::PLAYER);
        assert!(c.is_home_coach());
        assert!(c.has_property("DEV"));
        assert!(!c.has_property("ADMIN"));
    }
}
