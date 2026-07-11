use crate::client::status_report::StatusReport;
use crate::client::report::report_message_base::ReportMessage;
use crate::client::text_style::TextStyle;
use ffb_model::enums::Weather;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_weather_mage_result::{ReportWeatherMageResult, WeatherMageEffect};
use ffb_model::report::report_id::ReportId;

/// java: `report.getNewWeather()`/`getOldWeather()` return `Weather` enum objects in Java;
/// here the report stores only the (Java-side) SCREAMING_SNAKE_CASE enum-constant name as a
/// string, so it is parsed back into `ffb_model::enums::Weather` here.
fn parse_weather(name: &str) -> Option<Weather> {
    match name {
        "SWELTERING_HEAT" => Some(Weather::SwelteringHeat),
        "VERY_SUNNY" => Some(Weather::VerySunny),
        "NICE" => Some(Weather::Nice),
        "POURING_RAIN" => Some(Weather::PouringRain),
        "BLIZZARD" => Some(Weather::Blizzard),
        "INTRO" => Some(Weather::Intro),
        _ => None,
    }
}

/// 1:1 translation of `WeatherMageResultMessage.java`.
pub struct WeatherMageResultMessage;

impl WeatherMageResultMessage {
    fn report_changed_weather(&self, status_report: &mut StatusReport, game: &Game, report: &ReportWeatherMageResult) {
        let indent = status_report.get_indent();
        let new_weather = report.get_new_weather().and_then(parse_weather);
        let old_weather = report.get_old_weather().and_then(parse_weather);
        let new_name = new_weather.map(Weather::name).unwrap_or("");
        let old_name = old_weather.map(Weather::name).unwrap_or("");
        status_report.println_indent_style(
            indent + 1,
            TextStyle::NONE,
            &format!(
                "The weather is changed to {new_name}. It will return to {old_name} at the end of your {}turn or the end of the drive",
                if game.home_playing { "opponent's " } else { "" },
            ),
        );
        let mechanic = ffb_engine::mechanic::game_mechanic_for(game.rules);
        let description = new_weather.map(|w| mechanic.weather_description(w)).unwrap_or_default();
        status_report.println_indent_style(indent + 1, TextStyle::EXPLANATION, &description);
    }
}

impl ReportMessage for WeatherMageResultMessage {
    type Report = ReportWeatherMageResult;

    fn report_id(&self) -> ReportId {
        ReportId::WEATHER_MAGE_RESULT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        match report.get_effect() {
            Some(WeatherMageEffect::NO_CHANGE) => {
                status_report.println_indent_style(
                    indent + 1,
                    TextStyle::NONE,
                    "The mage fails to influence the weather enough to cause any changes",
                );
            }
            Some(WeatherMageEffect::CHANGED) => {
                self.report_changed_weather(status_report, game, report);
            }
            _ => {
                status_report.println_indent_style(indent + 1, TextStyle::EXPLANATION, "There was only one option");
                self.report_changed_weather(status_report, game, report);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(),
            name: format!("Player {id}"),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: "Team".into(),
            race: String::new(),
            roster_id: String::new(),
            coach: String::new(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home", vec![make_player("p1")]), make_team("away", vec![]), Rules::Bb2020)
    }

    #[test]
    fn no_change_reports_fails_to_influence() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportWeatherMageResult::new(0, None, Some(WeatherMageEffect::NO_CHANGE), None);
        WeatherMageResultMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("fails to influence the weather")));
    }

    #[test]
    fn changed_reports_new_and_old_weather() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportWeatherMageResult::new(
            1,
            Some("BLIZZARD".into()),
            Some(WeatherMageEffect::CHANGED),
            Some("NICE".into()),
        );
        WeatherMageResultMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("changed to Blizzard")));
        assert!(texts.iter().any(|t| t.contains("return to Nice Weather")));
    }

    #[test]
    fn no_choice_reports_explanation_then_changed_weather() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportWeatherMageResult::new(
            1,
            Some("SWELTERING_HEAT".into()),
            Some(WeatherMageEffect::NO_CHOICE),
            Some("NICE".into()),
        );
        WeatherMageResultMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("There was only one option")));
        assert!(texts.iter().any(|t| t.contains("changed to Sweltering Heat")));
    }

    #[test]
    fn changed_weather_mentions_opponents_turn_when_home_playing() {
        let mut game = make_game();
        game.home_playing = true;
        let mut status_report = StatusReport::new();
        let report = ReportWeatherMageResult::new(
            1,
            Some("BLIZZARD".into()),
            Some(WeatherMageEffect::CHANGED),
            Some("NICE".into()),
        );
        WeatherMageResultMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("opponent's turn or the end of the drive")));
    }
}
