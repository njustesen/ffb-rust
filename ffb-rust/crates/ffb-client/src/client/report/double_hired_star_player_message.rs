use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_double_hired_star_player::ReportDoubleHiredStarPlayer;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `DoubleHiredStarPlayerMessage.java`.
pub struct DoubleHiredStarPlayerMessage;

impl ReportMessage for DoubleHiredStarPlayerMessage {
    type Report = ReportDoubleHiredStarPlayer;

    fn report_id(&self) -> ReportId {
        ReportId::DOUBLE_HIRED_STAR_PLAYER
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let status = format!(
            "Star Player {} takes money from both teams and plays for neither.",
            report.get_star_player_name()
        );
        status_report.println_indent_style(status_report.get_indent(), TextStyle::BOLD, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::paragraph_style::ParagraphStyle;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(),
            name: id.to_string(),
            race: "human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
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
            special_rules: Vec::new(),
            players: Vec::new(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(DoubleHiredStarPlayerMessage.report_id(), ReportId::DOUBLE_HIRED_STAR_PLAYER);
    }

    #[test]
    fn renders_bold_message_with_star_player_name() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportDoubleHiredStarPlayer::new("Griff Oberwald".into());
        DoubleHiredStarPlayerMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs.len(), 2);
        assert_eq!(
            status_report.rendered_runs[0].text.as_deref(),
            Some("Star Player Griff Oberwald takes money from both teams and plays for neither.")
        );
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::BOLD));
        assert_eq!(status_report.rendered_runs[0].paragraph_style, Some(ParagraphStyle::INDENT_0));
    }

    #[test]
    fn renders_at_current_indent() {
        let mut status_report = StatusReport::new();
        status_report.set_indent(2);
        let game = make_game();
        let report = ReportDoubleHiredStarPlayer::new("Morg 'n' Thorg".into());
        DoubleHiredStarPlayerMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].paragraph_style, Some(ParagraphStyle::INDENT_2));
        assert!(status_report.rendered_runs[0].text.as_deref().unwrap().contains("Morg 'n' Thorg"));
    }

    #[test]
    fn different_star_player_name_reflected_in_text() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportDoubleHiredStarPlayer::new("Eldril Sidewinder".into());
        DoubleHiredStarPlayerMessage.render(&mut status_report, &game, &report);

        assert!(status_report.rendered_runs[0]
            .text
            .as_deref()
            .unwrap()
            .starts_with("Star Player Eldril Sidewinder"));
    }
}
