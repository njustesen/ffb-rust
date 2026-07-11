use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_pass_block::ReportPassBlock;

/// 1:1 translation of `PassBlockMessage.java`.
pub struct PassBlockMessage;

impl ReportMessage for PassBlockMessage {
    type Report = ReportPassBlock;

    fn report_id(&self) -> ReportId {
        ReportId::PASS_BLOCK
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        if !report.is_pass_block_available() {
            let text_style = if game.team_home.id == report.get_team_id() { TextStyle::HOME } else { TextStyle::AWAY };
            status_report.println_indent_style(status_report.get_indent(), text_style, "No pass blockers in range to intercept.");
        }
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
    fn available_prints_nothing() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportPassBlock::new("home".into(), true);
        PassBlockMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn unavailable_home_team_uses_home_style() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportPassBlock::new("home".into(), false);
        PassBlockMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::HOME));
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("No pass blockers in range to intercept."));
    }

    #[test]
    fn unavailable_away_team_uses_away_style() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportPassBlock::new("away".into(), false);
        PassBlockMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn get_key_matches_report_id() {
        assert_eq!(PassBlockMessage.get_key(), "passBlock");
    }
}
