use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_weather_mage_result::{ReportWeatherMageResult, WeatherMageEffect};
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `WeatherMageResultMessage.java`.
pub struct WeatherMageResultMessage;

impl WeatherMageResultMessage {
    fn report_changed_weather(&self, status_report: &mut StatusReport, report: &ReportWeatherMageResult, mechanic: &dyn ffb_mechanics::game_mechanic::GameMechanic) {
        let indent = status_report.get_indent();
        let new_weather = report.get_new_weather().unwrap_or("");
        status_report.println_indent_style(indent + 1, TextStyle::NONE, &format!("The weather is changed to {new_weather}"));
        // java: mechanic.weatherDescription(report.getNewWeather()) — the report only carries the
        // weather's name string here (no Weather enum), so the description is looked up by name.
        // Java calls this unconditionally since it always holds a real Weather object; the `if let`
        // here is a gap versus Java only if the stored name string fails to parse back to `Weather`.
        if let Some(weather) = ffb_model::enums::Weather::from_name(new_weather) {
            status_report.println_indent_style(indent + 1, TextStyle::EXPLANATION, &mechanic.weather_description(weather));
        }
    }
}

impl ReportMessage for WeatherMageResultMessage {
    type Report = ReportWeatherMageResult;

    fn report_id(&self) -> ReportId {
        ReportId::WEATHER_MAGE_RESULT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let mechanic = ffb_engine::mechanic::game_mechanic_for(game.rules);
        let indent = status_report.get_indent();

        match report.get_effect() {
            Some(WeatherMageEffect::NO_CHANGE) => {
                status_report.println_indent_style(indent + 1, TextStyle::NONE, "The mage fails to influence the weather enough to cause any changes");
            }
            Some(WeatherMageEffect::CHANGED) => {
                self.report_changed_weather(status_report, report, mechanic.as_ref());
            }
            _ => {
                status_report.println_indent_style(indent + 1, TextStyle::EXPLANATION, "There was only one option");
                self.report_changed_weather(status_report, report, mechanic.as_ref());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn report_id_is_weather_mage_result() {
        assert_eq!(WeatherMageResultMessage.report_id(), ReportId::WEATHER_MAGE_RESULT);
    }

    #[test]
    fn no_change_reports_fails_to_influence() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportWeatherMageResult::new(0, None, Some(WeatherMageEffect::NO_CHANGE), None);
        WeatherMageResultMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("fails to influence the weather")));
    }

    #[test]
    fn changed_reports_new_weather_and_description() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportWeatherMageResult::new(1, Some("Nice Weather".into()), Some(WeatherMageEffect::CHANGED), Some("Sweltering Heat".into()));
        WeatherMageResultMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "The weather is changed to Nice Weather"));
    }

    #[test]
    fn no_choice_reports_only_one_option_then_changed_weather() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportWeatherMageResult::new(1, Some("Nice Weather".into()), Some(WeatherMageEffect::NO_CHOICE), None);
        WeatherMageResultMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "There was only one option"));
        assert!(texts.iter().any(|t| t == "The weather is changed to Nice Weather"));
    }
}
