use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_pick_me_up::ReportPickMeUp;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `PickMeUpMessage.java`.
pub struct PickMeUpMessage;

impl ReportMessage for PickMeUpMessage {
    type Report = ReportPickMeUp;

    fn report_id(&self) -> ReportId {
        ReportId::PICK_ME_UP
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = report.get_player_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent();

        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Pick-me-up Roll [ {} ]", report.get_roll()));
        print_player(status_report, game, indent, false, player);
        if report.is_success() {
            status_report.println_indent_style(indent, TextStyle::NONE, " is picked up.");
        } else {
            status_report.println_indent_style(indent, TextStyle::NONE, " is not picked up.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, players: Vec<Player>) -> Team {
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
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        let player = Player { id: "p1".into(), name: "Grobnik".into(), ..Player::default() };
        Game::new(make_team("home", vec![player]), make_team("away", vec![]), Rules::Bb2020)
    }

    #[test]
    fn success() {
        let game = make_game();
        let report = ReportPickMeUp::new(Some("p1".into()), 5, true);
        let mut status_report = StatusReport::new();
        PickMeUpMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Pick-me-up Roll [ 5 ]"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Grobnik"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" is picked up."));
    }

    #[test]
    fn failure() {
        let game = make_game();
        let report = ReportPickMeUp::new(Some("p1".into()), 1, false);
        let mut status_report = StatusReport::new();
        PickMeUpMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" is not picked up."));
    }

    #[test]
    fn no_player_id() {
        let game = make_game();
        let report = ReportPickMeUp::new(None, 2, true);
        let mut status_report = StatusReport::new();
        PickMeUpMessage.render(&mut status_report, &game, &report);
        // No player-name run emitted; only roll header + result sentence (2 runs each).
        assert_eq!(status_report.rendered_runs.len(), 4);
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(PickMeUpMessage.report_id(), ReportId::PICK_ME_UP);
        assert_eq!(PickMeUpMessage.get_key(), "pickMeUp");
    }
}
