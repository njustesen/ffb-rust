use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_winnings::ReportWinnings;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool::format_thousands;

/// 1:1 translation of `WinningsMessage.java`.
pub struct WinningsMessage;

impl ReportMessage for WinningsMessage {
    type Report = ReportWinnings;

    fn report_id(&self) -> ReportId {
        ReportId::WINNINGS
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent() + 1;

        status_report.print_indent_style(indent, TextStyle::HOME_BOLD, &game.team_home.name.clone());
        status_report.println_indent_style(
            indent,
            TextStyle::NONE,
            &format!(" earns {} gold.", format_thousands(report.get_winnings_home() as i64)),
        );

        status_report.print_indent_style(indent, TextStyle::AWAY_BOLD, &game.team_away.name.clone());
        status_report.println_indent_style(
            indent,
            TextStyle::NONE,
            &format!(" earns {} gold.", format_thousands(report.get_winnings_away() as i64)),
        );
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
    fn renders_home_and_away_winnings() {
        let game = make_game();
        let report = ReportWinnings::new(50_000, 30_000);
        let mut status_report = StatusReport::new();
        WinningsMessage.render(&mut status_report, &game, &report);
        // run0 = home name (plain print), run1 = home earnings text, run2 = println terminator,
        // run3 = away name, run4 = away earnings text, run5 = terminator.
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Team home"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::HOME_BOLD));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" earns 50,000 gold."));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some("Team away"));
        assert_eq!(status_report.rendered_runs[3].text_style, Some(TextStyle::AWAY_BOLD));
        assert_eq!(status_report.rendered_runs[4].text.as_deref(), Some(" earns 30,000 gold."));
    }

    #[test]
    fn small_amounts_have_no_separator() {
        let game = make_game();
        let report = ReportWinnings::new(42, 0);
        let mut status_report = StatusReport::new();
        WinningsMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" earns 42 gold."));
        assert_eq!(status_report.rendered_runs[4].text.as_deref(), Some(" earns 0 gold."));
    }

    #[test]
    fn large_amount_has_multiple_separators() {
        let game = make_game();
        let report = ReportWinnings::new(2_130_000, 999_000);
        let mut status_report = StatusReport::new();
        WinningsMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" earns 2,130,000 gold."));
        assert_eq!(status_report.rendered_runs[4].text.as_deref(), Some(" earns 999,000 gold."));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(WinningsMessage.report_id(), ReportId::WINNINGS);
        assert_eq!(WinningsMessage.get_key(), "winnings");
    }
}
