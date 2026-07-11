//! 1:1 translation of `com.fumbbl.ffb.client.ClientParameters`.

use ffb_model::factory::client_mode_factory::ClientModeFactory;
use ffb_model::model::fantasy_football_exception::FantasyFootballException;
use ffb_model::model::ClientMode;

use crate::client::client_layout::ClientLayout;

const ARGUMENT_COACH: &str = "-coach";
const ARGUMENT_GAME_ID: &str = "-gameId";
const ARGUMENT_TEAM_ID: &str = "-teamId";
const ARGUMENT_TEAM_NAME: &str = "-teamName";
const ARGUMENT_TEAM_HOME: &str = "-teamHome";
const ARGUMENT_TEAM_AWAY: &str = "-teamAway";
const ARGUMENT_AUTHENTICATION: &str = "-auth";
const ARGUMENT_PORT: &str = "-port";
const ARGUMENT_SERVER: &str = "-server";
const ARGUMENT_BUILD: &str = "-build";
const ARGUMENT_LAYOUT: &str = "-layout";

/// Java: `com.fumbbl.ffb.client.ClientParameters`.
#[derive(Debug, Clone)]
pub struct ClientParameters {
    mode: Option<ClientMode>,
    coach: Option<String>,
    game_id: i64,
    team_id: Option<String>,
    team_name: Option<String>,
    team_home: Option<String>,
    team_away: Option<String>,
    authentication: Option<String>,
    port: i32,
    server: Option<String>,
    build: Option<String>,
    layout: ClientLayout,
}

impl ClientParameters {
    pub fn mode(&self) -> Option<ClientMode> {
        self.mode
    }

    pub fn coach(&self) -> Option<&str> {
        self.coach.as_deref()
    }

    pub fn game_id(&self) -> i64 {
        self.game_id
    }

    pub fn team_id(&self) -> Option<&str> {
        self.team_id.as_deref()
    }

    pub fn team_name(&self) -> Option<&str> {
        self.team_name.as_deref()
    }

    pub fn team_home(&self) -> Option<&str> {
        self.team_home.as_deref()
    }

    pub fn team_away(&self) -> Option<&str> {
        self.team_away.as_deref()
    }

    pub fn authentication(&self) -> Option<&str> {
        self.authentication.as_deref()
    }

    pub fn port(&self) -> i32 {
        self.port
    }

    pub fn server(&self) -> Option<&str> {
        self.server.as_deref()
    }

    pub fn build(&self) -> Option<&str> {
        self.build.as_deref()
    }

    pub fn layout(&self) -> ClientLayout {
        self.layout
    }

    /// Java: private constructor `ClientParameters(String[] pArguments)`.
    fn parse(arguments: &[String]) -> Result<Self, FantasyFootballException> {
        let mut params = ClientParameters {
            mode: None,
            coach: None,
            game_id: 0,
            team_id: None,
            team_name: None,
            team_home: None,
            team_away: None,
            authentication: None,
            port: 0,
            server: None,
            build: None,
            layout: ClientLayout::LANDSCAPE,
        };

        if arguments.is_empty() {
            return Ok(params);
        }

        let client_mode_factory = ClientModeFactory::default();
        let mut pos = 0usize;
        while pos < arguments.len() {
            let argument = Self::fetch_argument(arguments, pos)?;
            pos += 1;

            if let Some(mode) = client_mode_factory.for_argument(argument) {
                params.mode = Some(mode);
            } else if argument.eq_ignore_ascii_case(ARGUMENT_COACH) {
                params.coach = Some(Self::fetch_argument(arguments, pos)?.to_string());
                pos += 1;
            } else if argument.eq_ignore_ascii_case(ARGUMENT_GAME_ID) {
                let value = Self::fetch_argument(arguments, pos)?;
                pos += 1;
                params.game_id = value
                    .parse::<i64>()
                    .map_err(|_| FantasyFootballException::new("GameId must be numeric.".to_string()))?;
            } else if argument.eq_ignore_ascii_case(ARGUMENT_TEAM_ID) {
                params.team_id = Some(Self::fetch_argument(arguments, pos)?.to_string());
                pos += 1;
            } else if argument.eq_ignore_ascii_case(ARGUMENT_TEAM_NAME) {
                params.team_name = Some(Self::fetch_argument(arguments, pos)?.to_string());
                pos += 1;
            } else if argument.eq_ignore_ascii_case(ARGUMENT_TEAM_HOME) {
                params.team_home = Some(Self::fetch_argument(arguments, pos)?.to_string());
                pos += 1;
            } else if argument.eq_ignore_ascii_case(ARGUMENT_TEAM_AWAY) {
                params.team_away = Some(Self::fetch_argument(arguments, pos)?.to_string());
                pos += 1;
            } else if argument.eq_ignore_ascii_case(ARGUMENT_AUTHENTICATION) {
                params.authentication = Some(Self::fetch_argument(arguments, pos)?.to_string());
                pos += 1;
            } else if argument.eq_ignore_ascii_case(ARGUMENT_PORT) {
                let value = Self::fetch_argument(arguments, pos)?;
                pos += 1;
                params.port = value
                    .parse::<i32>()
                    .map_err(|_| FantasyFootballException::new("Port must be numeric.".to_string()))?;
            } else if argument.eq_ignore_ascii_case(ARGUMENT_SERVER) {
                params.server = Some(Self::fetch_argument(arguments, pos)?.to_string());
                pos += 1;
            } else if argument.eq_ignore_ascii_case(ARGUMENT_BUILD) {
                params.build = Some(Self::fetch_argument(arguments, pos)?.to_string());
                pos += 1;
            } else if argument.eq_ignore_ascii_case(ARGUMENT_LAYOUT) {
                let value = Self::fetch_argument(arguments, pos)?;
                pos += 1;
                params.layout = match value {
                    "LANDSCAPE" => ClientLayout::LANDSCAPE,
                    "PORTRAIT" => ClientLayout::PORTRAIT,
                    "SQUARE" => ClientLayout::SQUARE,
                    "WIDE" => ClientLayout::WIDE,
                    other => {
                        return Err(FantasyFootballException::new(format!(
                            "No enum constant ClientLayout.{other}"
                        )))
                    }
                };
            } else {
                return Err(FantasyFootballException::new(format!("Unknown argument {argument}")));
            }
        }

        Ok(params)
    }

    fn fetch_argument(arguments: &[String], position: usize) -> Result<&str, FantasyFootballException> {
        arguments
            .get(position)
            .map(String::as_str)
            .ok_or_else(|| FantasyFootballException::new("Argument list too short".to_string()))
    }

    /// Java: `private boolean validate()`.
    fn validate(&self) -> bool {
        match self.mode {
            None => false,
            Some(ClientMode::PLAYER) => {
                if self.coach.as_deref().unwrap_or_default().is_empty() {
                    return false;
                }
                if self.game_id > 0 {
                    if self.team_home.as_deref().is_some_and(|s| !s.is_empty()) {
                        return self.team_away.as_deref().is_some_and(|s| !s.is_empty());
                    }
                    if self.team_away.as_deref().is_some_and(|s| !s.is_empty()) {
                        return self.team_home.as_deref().is_some_and(|s| !s.is_empty());
                    }
                } else {
                    if self.team_id.as_deref().is_some_and(|s| !s.is_empty()) {
                        return self.team_name.as_deref().is_some_and(|s| !s.is_empty());
                    }
                    if self.team_name.as_deref().is_some_and(|s| !s.is_empty()) {
                        return self.team_id.as_deref().is_some_and(|s| !s.is_empty());
                    }
                }
                true
            }
            Some(ClientMode::SPECTATOR) => self.coach.as_deref().is_some_and(|s| !s.is_empty()),
            Some(ClientMode::REPLAY) => self.game_id > 0,
        }
    }

    /// Java: `public static ClientParameters createValidParams(String[] args)`.
    pub fn create_valid_params(args: &[String]) -> Option<Self> {
        let parameters = match Self::parse(args) {
            Ok(p) => p,
            Err(_) => return None,
        };
        if parameters.validate() {
            Some(parameters)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_mode_fails_validation() {
        assert!(ClientParameters::create_valid_params(&args(&["-coach", "bob"])).is_none());
    }

    #[test]
    fn player_without_coach_fails() {
        assert!(ClientParameters::create_valid_params(&args(&["-player"])).is_none());
    }

    #[test]
    fn player_with_coach_succeeds() {
        let params = ClientParameters::create_valid_params(&args(&["-player", "-coach", "bob"])).unwrap();
        assert_eq!(params.mode(), Some(ClientMode::PLAYER));
        assert_eq!(params.coach(), Some("bob"));
    }

    #[test]
    fn player_with_game_id_and_only_team_home_fails() {
        let result = ClientParameters::create_valid_params(&args(&[
            "-player", "-coach", "bob", "-gameId", "1", "-teamHome", "home",
        ]));
        assert!(result.is_none());
    }

    #[test]
    fn player_with_game_id_and_both_teams_succeeds() {
        let result = ClientParameters::create_valid_params(&args(&[
            "-player", "-coach", "bob", "-gameId", "1", "-teamHome", "home", "-teamAway", "away",
        ]));
        assert!(result.is_some());
    }

    #[test]
    fn player_with_team_id_but_no_team_name_fails() {
        let result = ClientParameters::create_valid_params(&args(&[
            "-player", "-coach", "bob", "-teamId", "1",
        ]));
        assert!(result.is_none());
    }

    #[test]
    fn spectator_requires_coach() {
        assert!(ClientParameters::create_valid_params(&args(&["-spectator"])).is_none());
        assert!(ClientParameters::create_valid_params(&args(&["-spectator", "-coach", "bob"])).is_some());
    }

    #[test]
    fn replay_requires_positive_game_id() {
        assert!(ClientParameters::create_valid_params(&args(&["-replay"])).is_none());
        let ok = ClientParameters::create_valid_params(&args(&["-replay", "-gameId", "42"])).unwrap();
        assert_eq!(ok.game_id(), 42);
    }

    #[test]
    fn non_numeric_game_id_returns_none() {
        assert!(ClientParameters::create_valid_params(&args(&["-replay", "-gameId", "abc"])).is_none());
    }

    #[test]
    fn unknown_argument_returns_none() {
        assert!(ClientParameters::create_valid_params(&args(&["-bogus"])).is_none());
    }

    #[test]
    fn trailing_argument_missing_value_returns_none() {
        assert!(ClientParameters::create_valid_params(&args(&["-replay", "-gameId"])).is_none());
    }

    #[test]
    fn layout_defaults_to_landscape() {
        let params = ClientParameters::create_valid_params(&args(&["-spectator", "-coach", "bob"])).unwrap();
        assert!(matches!(params.layout(), ClientLayout::LANDSCAPE));
    }

    #[test]
    fn layout_argument_is_parsed() {
        let params = ClientParameters::create_valid_params(&args(&[
            "-spectator", "-coach", "bob", "-layout", "PORTRAIT",
        ]))
        .unwrap();
        assert!(matches!(params.layout(), ClientLayout::PORTRAIT));
    }

    #[test]
    fn port_and_server_and_build_round_trip() {
        let params = ClientParameters::create_valid_params(&args(&[
            "-spectator", "-coach", "bob", "-port", "1234", "-server", "example.com", "-build", "42",
        ]))
        .unwrap();
        assert_eq!(params.port(), 1234);
        assert_eq!(params.server(), Some("example.com"));
        assert_eq!(params.build(), Some("42"));
    }
}
