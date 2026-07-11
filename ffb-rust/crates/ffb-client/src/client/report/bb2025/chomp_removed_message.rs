use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_chomp_removed::ReportChompRemoved;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `ChompRemovedMessage.java`.
pub struct ChompRemovedMessage;

impl ReportMessage for ChompRemovedMessage {
    type Report = ReportChompRemoved;

    fn report_id(&self) -> ReportId {
        ReportId::CHOMP_REMOVED
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = game.player(report.get_player());
        let indent = status_report.get_indent() + 1;
        print_player(status_report, game, indent, false, player);
        let mut status = String::from(" got unchomped ");
        if report.is_successful() {
            status.push_str("and is free to move again.");
        } else {
            status.push_str("but is still held by another player.");
        }
        status_report.println_indent(indent, &status);
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
        let home = make_team("home", vec![make_player("p1", "Chomped Player")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn successful_unchomp_is_free_to_move() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportChompRemoved::new("p1".into(), true);
        ChompRemovedMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("is free to move again.")));
    }

    #[test]
    fn unsuccessful_unchomp_is_still_held() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportChompRemoved::new("p1".into(), false);
        ChompRemovedMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("but is still held by another player.")));
    }

    #[test]
    fn prints_player_name() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportChompRemoved::new("p1".into(), true);
        ChompRemovedMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Chomped Player"));
    }

    #[test]
    fn report_id_is_chomp_removed() {
        assert_eq!(ChompRemovedMessage.report_id(), ReportId::CHOMP_REMOVED);
    }
}
