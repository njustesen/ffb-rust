use crate::client::report::report_message_base::{print_team_name, ReportMessage};
use crate::client::status_report::StatusReport;
use ffb_model::enums::LeaderState;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_leader::ReportLeader;

/// 1:1 translation of `LeaderMessage.java`.
pub struct LeaderMessage;

impl ReportMessage for LeaderMessage {
    type Report = ReportLeader;

    fn report_id(&self) -> ReportId {
        ReportId::LEADER
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.set_indent(0);
        if report.get_leader_state() == LeaderState::Available {
            print_team_name(status_report, game, false, report.get_team_id());
            status_report.print_indent(status_report.get_indent() + 1, " gain a Leader re-roll.");
        } else {
            status_report.print_indent(status_report.get_indent() + 1, "Leader re-roll removed from ");
            print_team_name(status_report, game, false, report.get_team_id());
        }
        status_report.println_indent(status_report.get_indent() + 1, ".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn available_prints_team_name_then_gain_message() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportLeader::new("home".into(), LeaderState::Available);
        LeaderMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert_eq!(texts[0], "home");
        assert!(texts.iter().any(|t| t == " gain a Leader re-roll."));
        assert!(texts.iter().any(|t| t == "."));
    }

    #[test]
    fn used_prints_removed_message_before_team_name() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportLeader::new("away".into(), LeaderState::Used);
        LeaderMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert_eq!(texts[0], "Leader re-roll removed from ");
        assert!(texts.iter().any(|t| t == "away"));
    }

    #[test]
    fn indent_reset_to_zero() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        status_report.set_indent(5);
        let report = ReportLeader::new("home".into(), LeaderState::Available);
        LeaderMessage.render(&mut status_report, &game, &report);
        // render prints at indent 0 (getIndent()) then 1 (getIndent() + 1)
        assert_eq!(status_report.rendered_runs[0].paragraph_style, Some(crate::client::paragraph_style::ParagraphStyle::INDENT_1));
    }

    #[test]
    fn get_key_matches_report_id() {
        assert_eq!(LeaderMessage.get_key(), "leader");
    }
}
