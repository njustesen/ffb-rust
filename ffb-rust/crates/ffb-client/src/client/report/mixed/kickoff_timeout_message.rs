use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_kickoff_timeout::ReportKickoffTimeout;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `KickoffTimeoutMessage.java`.
pub struct KickoffTimeoutMessage;

impl ReportMessage for KickoffTimeoutMessage {
    type Report = ReportKickoffTimeout;

    fn report_id(&self) -> ReportId {
        ReportId::KICKOFF_TIMEOUT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let status = format!("Timeout in turn {} of ", report.get_turn_number());
        status_report.print_indent(indent, &status);
        if game.home_playing {
            status_report.println_indent_style(indent, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.println_indent_style(indent, TextStyle::AWAY, &game.team_away.name.clone());
        }
        if report.get_turn_modifier() < 0 {
            status_report.println_indent(indent + 1, "The referee adjusts the clock back.");
            let status = format!(
                "Turn Counter is moved {} step backward.",
                report.get_turn_modifier().abs()
            );
            status_report.println_indent(indent + 1, &status);
        } else {
            status_report.println_indent(indent + 1, "The referee does not stop the clock.");
            let status = format!(
                "Turn Counter is moved {} step forward.",
                report.get_turn_modifier().abs()
            );
            status_report.println_indent(indent + 1, &status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::game::Game;
    use ffb_model::model::team::Team;
    use ffb_model::enums::Rules;

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
    fn home_playing_negative_modifier() {
        let mut game = make_game();
        game.home_playing = true;
        let report = ReportKickoffTimeout::new(-2, 5);
        let mut status_report = StatusReport::new();
        KickoffTimeoutMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Timeout in turn 5 of "));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some("Team home"));
        assert_eq!(status_report.rendered_runs[1].text_style, Some(TextStyle::HOME));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some("The referee adjusts the clock back."));
        assert_eq!(status_report.rendered_runs[5].text.as_deref(), Some("Turn Counter is moved 2 step backward."));
    }

    #[test]
    fn away_playing_positive_modifier() {
        let mut game = make_game();
        game.home_playing = false;
        let report = ReportKickoffTimeout::new(3, 1);
        let mut status_report = StatusReport::new();
        KickoffTimeoutMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some("Team away"));
        assert_eq!(status_report.rendered_runs[1].text_style, Some(TextStyle::AWAY));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some("The referee does not stop the clock."));
        assert_eq!(status_report.rendered_runs[5].text.as_deref(), Some("Turn Counter is moved 3 step forward."));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(KickoffTimeoutMessage.report_id(), ReportId::KICKOFF_TIMEOUT);
        assert_eq!(KickoffTimeoutMessage.get_key(), "kickoffTimeout");
    }
}
