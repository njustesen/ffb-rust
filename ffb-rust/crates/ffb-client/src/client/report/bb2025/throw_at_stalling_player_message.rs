use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_throw_at_stalling_player::ReportThrowAtStallingPlayer;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `ThrowAtStallingPlayerMessage.java`.
pub struct ThrowAtStallingPlayerMessage;

impl ReportMessage for ThrowAtStallingPlayerMessage {
    type Report = ReportThrowAtStallingPlayer;

    fn report_id(&self) -> ReportId {
        ReportId::THROW_AT_STALLING_PLAYER
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = report.get_player_id().and_then(|id| game.player(id));
        if report.get_roll() > 0 {
            status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Throw a Rock Roll [ {} ]", report.get_roll()));

            print_player(status_report, game, indent + 1, true, player);
            let message = if report.is_successful() {
                " is hit by a rock."
            } else {
                " is not punished for stalling."
            };
            status_report.println_indent(indent + 1, message);
        } else {
            print_player(status_report, game, indent + 1, true, player);
            status_report.println_indent(indent + 1, " stalled but the crowd can not be bothered.");
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
        let home = make_team("home", vec![make_player("p1", "Staller")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn report_id_is_throw_at_stalling_player() {
        assert_eq!(ThrowAtStallingPlayerMessage.report_id(), ReportId::THROW_AT_STALLING_PLAYER);
    }

    #[test]
    fn no_roll_reports_crowd_not_bothered() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowAtStallingPlayer::new(Some("p1".into()), 0, false);
        ThrowAtStallingPlayerMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " stalled but the crowd can not be bothered."));
        assert!(!texts.iter().any(|t| t.contains("Throw a Rock Roll")));
    }

    #[test]
    fn successful_roll_hits_with_rock() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowAtStallingPlayer::new(Some("p1".into()), 5, true);
        ThrowAtStallingPlayerMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Throw a Rock Roll [ 5 ]"));
        assert!(texts.iter().any(|t| t == " is hit by a rock."));
    }

    #[test]
    fn unsuccessful_roll_not_punished() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowAtStallingPlayer::new(Some("p1".into()), 1, false);
        ThrowAtStallingPlayerMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " is not punished for stalling."));
    }
}
