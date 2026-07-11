use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_look_into_my_eyes_roll::ReportLookIntoMyEyesRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `LookIntoMyEyesRollMessage.java`.
pub struct LookIntoMyEyesRollMessage;

impl ReportMessage for LookIntoMyEyesRollMessage {
    type Report = ReportLookIntoMyEyesRoll;

    fn report_id(&self) -> ReportId {
        ReportId::LOOK_INTO_MY_EYES_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let acting_player = &game.acting_player;
        let player = acting_player.player_id.as_deref().and_then(|id| game.player(id));

        let status = format!("Look Into My Eyes Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);

        print_player(status_report, game, indent + 1, false, player);
        let genitive = player.map(|p| p.gender.genitive()).unwrap_or_default();
        let status = if report.is_successful() {
            format!(" steals the ball from {genitive} opponent.")
        } else {
            format!(" fails to steal the ball from {genitive} opponent.")
        };
        status_report.println_indent(indent + 1, &status);
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
        let mut game = Game::new(make_team("home", vec![player]), make_team("away", vec![]), Rules::Bb2020);
        game.acting_player.player_id = Some("p1".into());
        game
    }

    #[test]
    fn successful_roll() {
        let game = make_game();
        let report = ReportLookIntoMyEyesRoll::new(Some("p1".into()), true, 4, 2, false);
        let mut status_report = StatusReport::new();
        LookIntoMyEyesRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Look Into My Eyes Roll [ 4 ]"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Grobnik"));
        assert!(status_report.rendered_runs[3].text.as_deref().unwrap().contains("steals the ball"));
    }

    #[test]
    fn failed_roll() {
        let game = make_game();
        let report = ReportLookIntoMyEyesRoll::new(Some("p1".into()), false, 1, 2, false);
        let mut status_report = StatusReport::new();
        LookIntoMyEyesRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs[3].text.as_deref().unwrap().contains("fails to steal"));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(LookIntoMyEyesRollMessage.report_id(), ReportId::LOOK_INTO_MY_EYES_ROLL);
        assert_eq!(LookIntoMyEyesRollMessage.get_key(), "lookIntoMyEyesRoll");
    }
}
