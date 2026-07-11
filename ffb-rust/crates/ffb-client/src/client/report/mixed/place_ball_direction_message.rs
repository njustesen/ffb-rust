use crate::client::report::report_message_base::{map_to_local, print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_place_ball_direction::ReportPlaceBallDirection;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `PlaceBallDirectionMessage.java`.
pub struct PlaceBallDirectionMessage;

impl ReportMessage for PlaceBallDirectionMessage {
    type Report = ReportPlaceBallDirection;

    fn report_id(&self) -> ReportId {
        ReportId::PLACE_BALL_DIRECTION
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = report.get_player_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent();
        print_player(status_report, game, indent, false, player);

        // java: report.getDirection() is called unconditionally (would NPE on null); a
        // missing direction has no equivalent name here so it's rendered as an empty string.
        let direction_name = report.get_direction().map(map_to_local).map(|d| d.name()).unwrap_or_default();
        let builder = format!(" places the ball {direction_name}.");
        status_report.println_indent_style(indent, TextStyle::NONE, &builder);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Direction, Rules};
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
    fn north_direction() {
        let game = make_game();
        let report = ReportPlaceBallDirection::new(Some("p1".into()), Some(Direction::North));
        let mut status_report = StatusReport::new();
        PlaceBallDirectionMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Grobnik"));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" places the ball North."));
    }

    #[test]
    fn southeast_direction() {
        let game = make_game();
        let report = ReportPlaceBallDirection::new(Some("p1".into()), Some(Direction::Southeast));
        let mut status_report = StatusReport::new();
        PlaceBallDirectionMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" places the ball Southeast."));
    }

    #[test]
    fn missing_direction_renders_empty_name() {
        let game = make_game();
        let report = ReportPlaceBallDirection::new(Some("p1".into()), None);
        let mut status_report = StatusReport::new();
        PlaceBallDirectionMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" places the ball ."));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(PlaceBallDirectionMessage.report_id(), ReportId::PLACE_BALL_DIRECTION);
        assert_eq!(PlaceBallDirectionMessage.get_key(), "placedBallDirection");
    }
}
