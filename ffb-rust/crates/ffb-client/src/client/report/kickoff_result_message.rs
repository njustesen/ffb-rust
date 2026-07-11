use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_kickoff_result::ReportKickoffResult;
use ffb_model::util::array_tool::ArrayTool;

/// 1:1 translation of `KickoffResultMessage.java`.
pub struct KickoffResultMessage;

impl ReportMessage for KickoffResultMessage {
    type Report = ReportKickoffResult;

    fn report_id(&self) -> ReportId {
        ReportId::KICKOFF_RESULT
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        status_report.set_indent(0);
        let kickoff_roll = report.get_kickoff_roll();
        let status = if ArrayTool::is_provided_int(kickoff_roll) {
            format!("Kick-off Event Roll [ {} ][ {} ]", kickoff_roll[0], kickoff_roll[1])
        } else {
            "Chosen kick-off event".to_string()
        };
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        let status = format!("Kick-off event is {}", report.get_kickoff_result().name());
        status_report.println_indent(status_report.get_indent() + 1, &status);
        // java: report.getKickoffResult().getDescription() — ffb-model's KickoffResult enum
        // does not carry the rule-text description strings from the Java kickoff table
        // implementations, so there is no derivable value to render here.
        let description = "";
        status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::EXPLANATION, description);
        status_report.set_indent(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::KickoffResult;
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
    fn rolled_event_prints_roll_values() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportKickoffResult::new(KickoffResult::Blitz, vec![3, 4]);
        KickoffResultMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.as_deref() == Some("Kick-off Event Roll [ 3 ][ 4 ]")));
        assert!(texts.iter().any(|t| t.as_deref() == Some("Kick-off event is Blitz")));
    }

    #[test]
    fn chosen_event_has_no_roll() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportKickoffResult::new(KickoffResult::QuickSnap, vec![]);
        KickoffResultMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.as_deref() == Some("Chosen kick-off event")));
    }

    #[test]
    fn indent_is_left_at_1_after_render() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportKickoffResult::new(KickoffResult::Blitz, vec![1, 2]);
        KickoffResultMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.get_indent(), 1);
    }

    #[test]
    fn get_key_matches_report_id() {
        assert_eq!(KickoffResultMessage.get_key(), "kickoffResult");
    }
}
