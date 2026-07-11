/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerWeather.
/// Abstract handler for `/weather` command — sets the field weather by short name.
use ffb_model::model::game::Game;
use ffb_model::factory::weather_factory::WeatherFactory;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerWeather {
    pub required_client: Client,
    pub required_environment: Environment,
    pub requires_one_privilege_of: Vec<Privilege>,
    weather_factory: WeatherFactory,
}

impl TalkHandlerWeather {
    /// Java: `super("/weather", 1, ...)`.
    pub const COMMAND: &'static str = "/weather";
    pub const COMMAND_PARTS_THRESHOLD: usize = 1;

    pub fn new(
        required_client: Client,
        required_environment: Environment,
        requires_one_privilege_of: Vec<Privilege>,
    ) -> Self {
        Self {
            required_client,
            required_environment,
            requires_one_privilege_of,
            weather_factory: WeatherFactory,
        }
    }

    /// Java: `handle(...)` — `commands[1]` is the weather short name (`WeatherFactory.forShortName`).
    pub fn handle(&self, game: &mut Game, commands: &[String]) -> Option<String> {
        let weather = self.weather_factory.for_short_name(commands.get(1)?)?;
        game.field_model.weather = weather;
        Some(format!("Setting weather to {}.", weather.name()))
    }
}

impl Default for TalkHandlerWeather {
    fn default() -> Self {
        Self::new(Client::None, Environment::None, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, Weather};
    use ffb_model::model::team::Team;

    fn team(name: &str) -> Team {
        Team {
            id: name.into(), name: name.into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn game() -> Game {
        Game::new(team("Home"), team("Away"), Rules::Bb2020)
    }

    #[test]
    fn construct() {
        let h = TalkHandlerWeather::new(Client::Spec, Environment::None, vec![Privilege::EditState]);
        assert_eq!(h.required_client, Client::Spec);
    }

    #[test]
    fn handle_sets_weather_by_short_name() {
        let h = TalkHandlerWeather::default();
        let mut g = game();
        let commands = vec!["/weather".to_string(), "rain".to_string()];
        let info = h.handle(&mut g, &commands).unwrap();
        assert_eq!(g.field_model.weather, Weather::PouringRain);
        assert!(info.contains("Pouring Rain"));
    }

    #[test]
    fn handle_unknown_weather_returns_none() {
        let h = TalkHandlerWeather::default();
        let mut g = game();
        let commands = vec!["/weather".to_string(), "not_a_weather".to_string()];
        assert!(h.handle(&mut g, &commands).is_none());
    }
}
