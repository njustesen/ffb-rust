use crate::client::report::report_message_base::{print_team_name, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_mascot_used::ReportMascotUsed;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `MascotUsedMessage.java`.
pub struct MascotUsedMessage;

impl ReportMessage for MascotUsedMessage {
    type Report = ReportMascotUsed;

    fn report_id(&self) -> ReportId {
        ReportId::MASCOT_USED
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.println_indent_style(
            status_report.get_indent(),
            TextStyle::ROLL,
            &format!("Mascot Roll [ {} ]", report.get_roll()),
        );
        print_team_name(status_report, game, false, report.get_team_id());
        let mut builder = String::from(" used their Team Mascot");
        if report.is_successful() {
            builder.push_str(" successfully.");
        } else if report.is_fallback() {
            builder.push_str(" but it failed so they used a regular re-roll instead.");
        } else {
            builder.push_str(" but it failed.");
        }
        status_report.println_indent(status_report.get_indent(), &builder);

        if !report.is_successful() {
            status_report.println_indent_style(
                status_report.get_indent() + 1,
                TextStyle::NONE,
                &format!("(Roll >= {} to succeed)", report.get_minimum_roll()),
            );
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
    fn successful_mascot_use() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportMascotUsed::new("home".into(), 4, 5, true, false);
        MascotUsedMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("used their Team Mascot successfully.")));
        assert!(!texts.iter().any(|t| t.contains("Roll >=")));
    }

    #[test]
    fn failed_with_fallback_reroll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportMascotUsed::new("home".into(), 4, 2, false, true);
        MascotUsedMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("but it failed so they used a regular re-roll instead.")));
        assert!(texts.iter().any(|t| t.contains("Roll >= 4 to succeed")));
    }

    #[test]
    fn failed_without_fallback() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportMascotUsed::new("away".into(), 5, 1, false, false);
        MascotUsedMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " used their Team Mascot but it failed."));
    }

    #[test]
    fn report_id_is_mascot_used() {
        assert_eq!(MascotUsedMessage.report_id(), ReportId::MASCOT_USED);
    }
}
