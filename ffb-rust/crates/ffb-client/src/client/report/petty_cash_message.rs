use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_petty_cash::ReportPettyCash;
use ffb_model::util::string_tool::format_thousands;

/// 1:1 translation of `PettyCashMessage.java`.
pub struct PettyCashMessage;

impl ReportMessage for PettyCashMessage {
    type Report = ReportPettyCash;

    fn report_id(&self) -> ReportId {
        ReportId::PETTY_CASH
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        if !status_report.petty_cash_report_received {
            status_report.petty_cash_report_received = true;
            status_report.println_indent_style(status_report.get_indent(), TextStyle::BOLD, "Transfer Petty Cash");
        }
        status_report.print_indent(status_report.get_indent() + 1, "Team ");
        let team: &Team = if game.team_home.id == report.get_team_id() {
            status_report.print_indent_style(status_report.get_indent() + 1, TextStyle::HOME, &game.team_home.name.clone());
            &game.team_home
        } else {
            status_report.print_indent_style(status_report.get_indent() + 1, TextStyle::AWAY, &game.team_away.name.clone());
            &game.team_away
        };
        let mut status = " transfers ".to_string();
        if report.get_gold() > 0 {
            status.push_str(&format_thousands(report.get_gold() as i64));
            status.push_str(" gold");
        } else {
            status.push_str("nothing");
        }
        status.push_str(" from the Treasury into Petty Cash.");
        status_report.println_indent(status_report.get_indent() + 1, &status);
        if report.get_gold() > team.treasury {
            let status = format!(
                "They received an extra {} gold for being the underdog.",
                format_thousands((report.get_gold() - team.treasury) as i64)
            );
            status_report.println_indent(status_report.get_indent() + 1, &status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    fn make_team(id: &str, treasury: i32) -> Team {
        Team { id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false }
    }

    fn make_game(home_treasury: i32, away_treasury: i32) -> Game {
        Game::new(make_team("home", home_treasury), make_team("away", away_treasury), Rules::Bb2025)
    }

    #[test]
    fn first_report_prints_header_once() {
        let game = make_game(1000, 1000);
        let mut status_report = StatusReport::new();
        let report = ReportPettyCash::new("home".into(), 500);
        PettyCashMessage.render(&mut status_report, &game, &report);
        assert!(status_report.petty_cash_report_received);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Transfer Petty Cash"));
    }

    #[test]
    fn second_report_does_not_repeat_header() {
        let game = make_game(1000, 1000);
        let mut status_report = StatusReport::new();
        status_report.petty_cash_report_received = true;
        let report = ReportPettyCash::new("home".into(), 500);
        PettyCashMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(!texts.iter().any(|t| t == "Transfer Petty Cash"));
    }

    #[test]
    fn zero_gold_says_nothing() {
        let game = make_game(1000, 1000);
        let mut status_report = StatusReport::new();
        let report = ReportPettyCash::new("away".into(), 0);
        PettyCashMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " transfers nothing from the Treasury into Petty Cash."));
    }

    #[test]
    fn gold_over_treasury_reports_underdog_bonus() {
        let game = make_game(100, 1000);
        let mut status_report = StatusReport::new();
        let report = ReportPettyCash::new("home".into(), 500);
        PettyCashMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "They received an extra 400 gold for being the underdog."));
    }

    #[test]
    fn gold_formats_with_thousands_separator() {
        let game = make_game(1_000_000, 1000);
        let mut status_report = StatusReport::new();
        let report = ReportPettyCash::new("home".into(), 50_000);
        PettyCashMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " transfers 50,000 gold from the Treasury into Petty Cash."));
    }

    #[test]
    fn get_key_matches_report_id() {
        assert_eq!(PettyCashMessage.get_key(), "pettyCash");
    }
}
