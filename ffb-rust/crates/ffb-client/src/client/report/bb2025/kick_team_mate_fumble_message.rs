use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_kick_team_mate_fumble::ReportKickTeamMateFumble;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `KickTeamMateFumbleMessage.java`.
pub struct KickTeamMateFumbleMessage;

impl ReportMessage for KickTeamMateFumbleMessage {
    type Report = ReportKickTeamMateFumble;

    fn report_id(&self) -> ReportId {
        ReportId::KICK_TEAM_MATE_FUMBLE
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, _report: &Self::Report) {
        status_report.println_indent_style(
            status_report.get_indent() + 2,
            TextStyle::EXPLANATION,
            "Fumbled Kick Team-Mate always causes at least a KO.",
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
    fn prints_explanation_at_indent_plus_two() {
        let mut status_report = StatusReport::new();
        status_report.set_indent(1);
        let game = make_game();
        let report = ReportKickTeamMateFumble::new();
        KickTeamMateFumbleMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Fumbled Kick Team-Mate always causes at least a KO."));
    }

    #[test]
    fn uses_explanation_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickTeamMateFumble::new();
        KickTeamMateFumbleMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::EXPLANATION));
    }

    #[test]
    fn indent_is_base_indent_plus_two() {
        use crate::client::paragraph_style::ParagraphStyle;
        let mut status_report = StatusReport::new();
        status_report.set_indent(0);
        let game = make_game();
        let report = ReportKickTeamMateFumble::new();
        KickTeamMateFumbleMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].paragraph_style, Some(ParagraphStyle::INDENT_2));
    }

    #[test]
    fn report_id_is_kick_team_mate_fumble() {
        assert_eq!(KickTeamMateFumbleMessage.report_id(), ReportId::KICK_TEAM_MATE_FUMBLE);
    }
}
