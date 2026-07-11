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
        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Throw a Rock Roll [ {} ]", report.get_roll()));
        let player = report.get_player_id().and_then(|id| game.player(id));
        print_player(status_report, game, indent + 1, true, player);
        let message = if report.is_successful() {
            " is hit by a rock."
        } else {
            " is not punished for stalling."
        };
        status_report.println_indent(indent + 1, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(),
            name: "Bob".into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: "Team".into(),
            race: String::new(),
            roster_id: String::new(),
            coach: String::new(),
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
        Game::new(
            make_team("home", vec![make_player("p1")]),
            make_team("away", vec![]),
            Rules::Bb2020,
        )
    }

    #[test]
    fn successful_reports_hit_by_rock() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportThrowAtStallingPlayer::new(Some("p1".into()), 6, true);
        ThrowAtStallingPlayerMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&" is hit by a rock."));
    }

    #[test]
    fn unsuccessful_reports_not_punished() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportThrowAtStallingPlayer::new(Some("p1".into()), 1, false);
        ThrowAtStallingPlayerMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&" is not punished for stalling."));
    }

    #[test]
    fn roll_line_uses_current_indent() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        status_report.set_indent(2);
        let report = ReportThrowAtStallingPlayer::new(Some("p1".into()), 6, true);
        ThrowAtStallingPlayerMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Throw a Rock Roll [ 6 ]"));
    }
}
