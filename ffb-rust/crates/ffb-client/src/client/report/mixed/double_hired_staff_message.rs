use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_double_hired_staff::ReportDoubleHiredStaff;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `DoubleHiredStaffMessage.java`.
pub struct DoubleHiredStaffMessage;

impl ReportMessage for DoubleHiredStaffMessage {
    type Report = ReportDoubleHiredStaff;

    fn report_id(&self) -> ReportId {
        ReportId::DOUBLE_HIRED_STAFF
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let status = format!(
            "Inamous Coaching Staff {} takes money from both teams and plays for neither.",
            report.get_staff_name().unwrap_or("")
        );
        status_report.println_indent_style(status_report.get_indent(), TextStyle::BOLD, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: id.into(),
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
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020)
    }

    #[test]
    fn renders_staff_name_in_bold() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportDoubleHiredStaff::new(Some("Mr. Wibble".into()));
        DoubleHiredStaffMessage.render(&mut sr, &game, &report);
        assert_eq!(
            sr.rendered_runs[0].text.as_deref(),
            Some("Inamous Coaching Staff Mr. Wibble takes money from both teams and plays for neither.")
        );
        assert_eq!(sr.rendered_runs[0].text_style, Some(TextStyle::BOLD));
    }

    #[test]
    fn renders_at_current_indent() {
        let mut sr = StatusReport::new();
        sr.set_indent(2);
        let game = make_game();
        let report = ReportDoubleHiredStaff::new(Some("Coach".into()));
        DoubleHiredStaffMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs[0].text.as_deref().unwrap().contains("Coach"));
    }

    #[test]
    fn renders_empty_staff_name_gracefully() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportDoubleHiredStaff::new(None);
        DoubleHiredStaffMessage.render(&mut sr, &game, &report);
        assert_eq!(
            sr.rendered_runs[0].text.as_deref(),
            Some("Inamous Coaching Staff  takes money from both teams and plays for neither.")
        );
    }
}
