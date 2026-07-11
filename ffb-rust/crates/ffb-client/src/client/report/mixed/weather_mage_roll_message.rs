use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_weather_mage_roll::ReportWeatherMageRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `WeatherMageRollMessage.java`.
pub struct WeatherMageRollMessage;

impl ReportMessage for WeatherMageRollMessage {
    type Report = ReportWeatherMageRoll;

    fn report_id(&self) -> ReportId {
        ReportId::WEATHER_MAGE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let roll = report.get_weather_roll();
        let indent = status_report.get_indent();
        let status = format!("Weather Roll [ {} ][ {} ] ", roll[0], roll[1]);
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        status_report.println_indent_style(indent + 1, TextStyle::NONE, "The weather mage works his magic");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {id}"),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: format!("Coach {id}"),
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
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2020)
    }

    #[test]
    fn renders_both_dice_values() {
        let game = make_game();
        let report = ReportWeatherMageRoll::new(vec![3, 5]);
        let mut status_report = StatusReport::new();
        WeatherMageRollMessage.render(&mut status_report, &game, &report);
        // run0 = weather roll text, run1 = println terminator, run2 = second line text.
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Weather Roll [ 3 ][ 5 ] "));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::ROLL));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("The weather mage works his magic"));
    }

    #[test]
    fn renders_different_dice_values() {
        let game = make_game();
        let report = ReportWeatherMageRoll::new(vec![1, 6]);
        let mut status_report = StatusReport::new();
        WeatherMageRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Weather Roll [ 1 ][ 6 ] "));
    }

    #[test]
    fn indent_is_respected() {
        let game = make_game();
        let report = ReportWeatherMageRoll::new(vec![2, 2]);
        let mut status_report = StatusReport::new();
        status_report.set_indent(1);
        WeatherMageRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs.len(), 4);
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(WeatherMageRollMessage.report_id(), ReportId::WEATHER_MAGE_ROLL);
        assert_eq!(WeatherMageRollMessage.get_key(), "weatherMageRoll");
    }
}
