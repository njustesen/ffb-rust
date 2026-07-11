use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_bomb_out_of_bounds::ReportBombOutOfBounds;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `BombOutOfBoundsMessage.java`.
pub struct BombOutOfBoundsMessage;

impl ReportMessage for BombOutOfBoundsMessage {
    type Report = ReportBombOutOfBounds;

    fn report_id(&self) -> ReportId {
        ReportId::BOMB_OUT_OF_BOUNDS
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, _report: &Self::Report) {
        status_report.println_indent_style(status_report.get_indent(), TextStyle::BOLD, "Bomb scattered out of bounds.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(BombOutOfBoundsMessage.report_id(), ReportId::BOMB_OUT_OF_BOUNDS);
    }

    #[test]
    fn renders_bold_message() {
        let game = make_game();
        let report = ReportBombOutOfBounds::new();
        let mut status_report = StatusReport::new();
        BombOutOfBoundsMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Bomb scattered out of bounds."));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::BOLD));
    }

    #[test]
    fn emits_terminator_run() {
        let game = make_game();
        let report = ReportBombOutOfBounds::new();
        let mut status_report = StatusReport::new();
        BombOutOfBoundsMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs.len(), 2);
        assert_eq!(status_report.rendered_runs[1].text, None);
    }

    #[test]
    fn honors_current_indent() {
        let game = make_game();
        let report = ReportBombOutOfBounds::new();
        let mut status_report = StatusReport::new();
        status_report.set_indent(3);
        BombOutOfBoundsMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].paragraph_style, Some(crate::client::paragraph_style::ParagraphStyle::INDENT_3));
    }
}
