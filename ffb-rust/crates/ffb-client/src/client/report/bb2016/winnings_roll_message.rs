use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_winnings_roll::ReportWinningsRoll;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool;

pub struct WinningsRollMessage;

impl ReportMessage for WinningsRollMessage {
    type Report = ReportWinningsRoll;

    fn report_id(&self) -> ReportId {
        ReportId::WINNINGS_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();

        if report.get_winnings_roll_away() == 0 && report.get_winnings_roll_home() > 0 {
            status_report.print_indent_style(indent, TextStyle::NONE, "Coach ");
            status_report.print_indent_style(indent, TextStyle::HOME, &game.team_home.coach.clone());
            status_report.println_indent_style(indent, TextStyle::NONE, " re-rolls winnings.");
        }
        if report.get_winnings_roll_home() == 0 && report.get_winnings_roll_away() > 0 {
            status_report.print_indent_style(indent, TextStyle::NONE, "Coach ");
            status_report.print_indent_style(indent, TextStyle::AWAY, &game.team_away.coach.clone());
            status_report.println_indent_style(indent, TextStyle::NONE, " re-rolls winnings.");
        }

        if report.get_winnings_roll_home() > 0 {
            let status = format!("Winnings Roll Home Team [ {} ]", report.get_winnings_roll_home());
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
            status_report.print_indent_style(indent + 1, TextStyle::HOME, &game.team_home.name.clone());
            let status = format!(" earn {} goldcoins.", string_tool::format_thousands(report.get_winnings_home() as i64));
            status_report.println_indent_style(indent + 1, TextStyle::NONE, &status);
        }

        if report.get_winnings_roll_away() > 0 {
            let status = format!("Winnings Roll Away Team [ {} ]", report.get_winnings_roll_away());
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
            status_report.print_indent_style(indent + 1, TextStyle::AWAY, &game.team_away.name.clone());
            let status = format!(" earn {} in gold.", string_tool::format_thousands(report.get_winnings_away() as i64));
            status_report.println_indent_style(indent + 1, TextStyle::NONE, &status);
        }

        if report.get_winnings_roll_home() == 0 && report.get_winnings_roll_away() == 0 {
            if report.get_winnings_home() > 0 {
                status_report.println_indent_style(indent, TextStyle::BOLD, "Winnings: Concession of Away Team");
                status_report.print_indent_style(indent + 1, TextStyle::HOME, &game.team_home.name.clone());
                status_report.print_indent_style(indent + 1, TextStyle::NONE, " win ");
                status_report.print_indent_style(indent + 1, TextStyle::NONE, &report.get_winnings_home().to_string());
                status_report.println_indent_style(indent + 1, TextStyle::NONE, " in gold.");
                status_report.print_indent_style(indent + 1, TextStyle::AWAY, &game.team_away.name.clone());
                status_report.println_indent_style(indent + 1, TextStyle::NONE, " get nothing.");
            }
            if report.get_winnings_away() > 0 {
                status_report.println_indent_style(indent, TextStyle::BOLD, "Winnings: Concession of Home Team");
                status_report.print_indent_style(indent + 1, TextStyle::AWAY, &game.team_away.name.clone());
                status_report.print_indent_style(indent + 1, TextStyle::NONE, " win ");
                status_report.print_indent_style(indent + 1, TextStyle::NONE, &report.get_winnings_away().to_string());
                status_report.println_indent_style(indent + 1, TextStyle::NONE, " in gold.");
                status_report.print_indent_style(indent + 1, TextStyle::HOME, &game.team_home.name.clone());
                status_report.println_indent_style(indent + 1, TextStyle::NONE, " get nothing.");
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
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: format!("Coach{id}"),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_winnings_roll() {
        assert_eq!(WinningsRollMessage.get_key(), "winningsRoll");
    }

    #[test]
    fn home_roll_reports_earnings() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportWinningsRoll::new(5, 20000, 0, 0);
        WinningsRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" earn 20,000 goldcoins.")));
    }

    #[test]
    fn no_rolls_reports_concession_of_away() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportWinningsRoll::new(0, 15000, 0, 0);
        WinningsRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Winnings: Concession of Away Team")));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" get nothing.")));
    }

    #[test]
    fn away_reroll_reports_coach_name() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportWinningsRoll::new(0, 0, 6, 25000);
        WinningsRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Coachaway")));
    }
}
