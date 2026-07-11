use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_prayer_end::ReportPrayerEnd;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `PrayerEndMessage.java`.
pub struct PrayerEndMessage;

impl ReportMessage for PrayerEndMessage {
    type Report = ReportPrayerEnd;

    fn report_id(&self) -> ReportId {
        ReportId::PRAYER_END
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        status_report.println_indent_style(
            indent,
            TextStyle::ROLL,
            &format!("Prayer effect ended: {}", report.get_prayer().unwrap_or_default()),
        );
        // java: Prayer.getDescription() not modeled on ReportPrayerEnd (only the name string
        // survives translation) — using the name as a placeholder.
        status_report.println_indent_style(
            indent + 2,
            TextStyle::EXPLANATION,
            &format!("Effect was: {}", report.get_prayer().unwrap_or_default()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, name: &str) -> Team {
        Team {
            id: id.into(),
            name: name.into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
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
        Game::new(make_team("home", "Home Team"), make_team("away", "Away Team"), Rules::Bb2020)
    }

    #[test]
    fn renders_prayer_name_on_both_lines() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerEnd::new(Some("PRAYER_OF_DEATH".into()));
        PrayerEndMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Prayer effect ended: PRAYER_OF_DEATH"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::ROLL));
    }

    #[test]
    fn second_line_uses_indent_plus_two_and_explanation_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerEnd::new(Some("HAND_OF_GOD".into()));
        PrayerEndMessage.render(&mut status_report, &game, &report);
        // rendered_runs: [line1, terminator, line2, terminator]
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Effect was: HAND_OF_GOD"));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::EXPLANATION));
    }

    #[test]
    fn missing_prayer_name_renders_empty_string() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerEnd::new(None);
        PrayerEndMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Prayer effect ended: "));
    }
}
