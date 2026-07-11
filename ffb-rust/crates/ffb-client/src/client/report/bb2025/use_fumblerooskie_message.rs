use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_fumblerooskie::ReportFumblerooskie;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `UseFumblerooskieMessage.java`.
pub struct UseFumblerooskieMessage;

impl ReportMessage for UseFumblerooskieMessage {
    type Report = ReportFumblerooskie;

    fn report_id(&self) -> ReportId {
        ReportId::FUMBLEROOSKIE
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = report.get_player_id().and_then(|id| game.player(id));

        print_player(status_report, game, indent, false, player);
        if report.is_used() {
            status_report.println_indent_style(
                indent,
                TextStyle::NONE,
                " will drop the ball using Fumblerooski once he moves from the current square.",
            );
        } else {
            status_report.println_indent_style(indent, TextStyle::NONE, " did not vacate the square and thus keeps the ball.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str) -> Player {
        Player { id: id.into(), name: name.into(), player_type: PlayerType::default(), ..Default::default() }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players, vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![make_player("p1", "Ball Carrier")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn report_id_is_fumblerooskie() {
        assert_eq!(UseFumblerooskieMessage.report_id(), ReportId::FUMBLEROOSKIE);
    }

    #[test]
    fn used_reports_will_drop_the_ball() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportFumblerooskie::new(Some("p1".into()), true);
        UseFumblerooskieMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("will drop the ball using Fumblerooski")));
    }

    #[test]
    fn not_used_reports_keeps_the_ball() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportFumblerooskie::new(Some("p1".into()), false);
        UseFumblerooskieMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("did not vacate the square and thus keeps the ball.")));
    }

    #[test]
    fn prints_player_name() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportFumblerooskie::new(Some("p1".into()), true);
        UseFumblerooskieMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Ball Carrier"));
    }
}
