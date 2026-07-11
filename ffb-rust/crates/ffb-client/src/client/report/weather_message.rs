use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_engine::mechanic::game_mechanic_for;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_weather::ReportWeather;

pub struct WeatherMessage;

impl ReportMessage for WeatherMessage {
    type Report = ReportWeather;

    fn report_id(&self) -> ReportId {
        ReportId::WEATHER
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let mechanic = game_mechanic_for(game.rules);
        let roll = report.get_weather_roll();
        let indent = status_report.get_indent();
        let status = format!("Weather Roll [ {} ][ {} ] ", roll[0], roll[1]);
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let weather = report.get_weather();
        let status = format!("Weather is {}", weather.name());
        status_report.println_indent(indent + 1, &status);
        status_report.println_indent_style(indent + 1, TextStyle::EXPLANATION, &mechanic.weather_description(weather));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, Weather};
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
        }
    }

    fn make_game(rules: Rules) -> Game {
        Game::new(make_team("home"), make_team("away"), rules)
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(WeatherMessage.report_id(), ReportId::WEATHER);
    }

    #[test]
    fn nice_weather_renders_roll_and_description() {
        let game = make_game(Rules::Bb2025);
        let report = ReportWeather::new(Weather::Nice, vec![3, 4]);
        let mut status_report = StatusReport::new();
        WeatherMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Weather Roll [ 3 ][ 4 ] ".to_string())));
        assert!(texts.contains(&Some("Weather is Nice Weather".to_string())));
    }

    #[test]
    fn explanation_style_is_used_for_weather_description() {
        let game = make_game(Rules::Bb2025);
        let report = ReportWeather::new(Weather::Blizzard, vec![1, 6]);
        let mut status_report = StatusReport::new();
        WeatherMessage.render(&mut status_report, &game, &report);
        let has_explanation = status_report
            .rendered_runs
            .iter()
            .any(|r| r.text_style == Some(TextStyle::EXPLANATION));
        assert!(has_explanation);
    }

    #[test]
    fn sweltering_heat_renders_correct_name() {
        let game = make_game(Rules::Bb2025);
        let report = ReportWeather::new(Weather::SwelteringHeat, vec![2, 2]);
        let mut status_report = StatusReport::new();
        WeatherMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Weather is Sweltering Heat".to_string())));
    }
}
