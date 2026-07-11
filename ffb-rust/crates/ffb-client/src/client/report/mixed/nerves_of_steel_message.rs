use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_nerves_of_steel::ReportNervesOfSteel;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `NervesOfSteelMessage.java`.
pub struct NervesOfSteelMessage;

impl ReportMessage for NervesOfSteelMessage {
    type Report = ReportNervesOfSteel;

    fn report_id(&self) -> ReportId {
        ReportId::NERVES_OF_STEEL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = report.get_player_id().and_then(|id| game.player(id));
        let ball_action = report.get_ball_action();

        if let Some(player) = player {
            let indent = status_report.get_indent();
            print_player(status_report, game, indent, false, Some(player));
            status_report.print_indent(indent, " is using Nerves of Steel to ");
            if report.is_bomb() {
                status_report.println_indent(indent, "throw the bomb.");
            } else {
                status_report.println_indent(indent, &format!("{} the ball.", ball_action.unwrap_or_default()));
            }
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
    fn bomb_throw() {
        let game = make_game();
        let report = ReportNervesOfSteel::new(Some("p1".into()), Some("PASS".into()), true);
        let mut status_report = StatusReport::new();
        NervesOfSteelMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Grobnik"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("throw the bomb."));
    }

    #[test]
    fn ball_action_used_when_not_bomb() {
        let game = make_game();
        let report = ReportNervesOfSteel::new(Some("p1".into()), Some("pass".into()), false);
        let mut status_report = StatusReport::new();
        NervesOfSteelMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("pass the ball."));
    }

    #[test]
    fn no_player_renders_nothing() {
        let game = make_game();
        let report = ReportNervesOfSteel::new(Some("unknown".into()), Some("pass".into()), false);
        let mut status_report = StatusReport::new();
        NervesOfSteelMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(NervesOfSteelMessage.report_id(), ReportId::NERVES_OF_STEEL);
        assert_eq!(NervesOfSteelMessage.get_key(), "nervesOfSteel");
    }
}
