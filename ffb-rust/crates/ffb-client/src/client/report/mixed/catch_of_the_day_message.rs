use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_catch_of_the_day_roll::ReportCatchOfTheDayRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `CatchOfTheDayMessage.java`.
pub struct CatchOfTheDayMessage;

impl ReportMessage for CatchOfTheDayMessage {
    type Report = ReportCatchOfTheDayRoll;

    fn report_id(&self) -> ReportId {
        ReportId::CATCH_OF_THE_DAY
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = report.get_player_id().and_then(|id| game.player(id));

        if !report.is_re_rolled() {
            print_player(status_report, game, indent, true, player);
            status_report.println_indent_style(indent, TextStyle::BOLD, " tries to get the ball from the ground:");
        }

        status_report.println_indent_style(indent + 1, TextStyle::ROLL, &format!("Catch of the Day Roll [ {} ]", report.get_roll()));

        print_player(status_report, game, indent + 2, false, player);
        let mut needed_roll: Option<String> = None;
        if report.is_successful() {
            status_report.println_indent(indent + 2, " gets the ball.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(indent + 2, " fails to get the ball.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }
        if let Some(needed_roll) = needed_roll {
            status_report.println_indent(indent + 2, &needed_roll);
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
            coach: "Coach".into(),
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
        let player = Player { id: "p1".into(), name: "Scavenger".into(), ..Player::default() };
        Game::new(make_team("home", vec![player]), make_team("away", vec![]), Rules::Bb2020)
    }

    #[test]
    fn success_without_re_roll_shows_intro_and_needed_roll() {
        let game = make_game();
        let report = ReportCatchOfTheDayRoll::new(Some("p1".into()), true, 4, 2, false);
        let mut sr = StatusReport::new();
        CatchOfTheDayMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Scavenger"));
        assert_eq!(sr.rendered_runs[1].text.as_deref(), Some(" tries to get the ball from the ground:"));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some("Catch of the Day Roll [ 4 ]"));
        assert_eq!(sr.rendered_runs[5].text.as_deref(), Some("Scavenger"));
        assert_eq!(sr.rendered_runs[6].text.as_deref(), Some(" gets the ball."));
        assert_eq!(sr.rendered_runs[8].text.as_deref(), Some("Succeeded on a roll of 2+"));
    }

    #[test]
    fn failure_shows_fails_to_get_ball() {
        let game = make_game();
        let report = ReportCatchOfTheDayRoll::new(Some("p1".into()), false, 1, 2, false);
        let mut sr = StatusReport::new();
        CatchOfTheDayMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[6].text.as_deref(), Some(" fails to get the ball."));
        assert_eq!(sr.rendered_runs[8].text.as_deref(), Some("Roll a 2+ to succeed"));
    }

    #[test]
    fn re_rolled_skips_intro_and_needed_roll() {
        let game = make_game();
        let report = ReportCatchOfTheDayRoll::new(Some("p1".into()), true, 4, 2, true);
        let mut sr = StatusReport::new();
        CatchOfTheDayMessage.render(&mut sr, &game, &report);
        // roll println (2) + player print (1) + success println (2) = 5, no intro, no needed roll.
        assert_eq!(sr.rendered_runs.len(), 5);
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(CatchOfTheDayMessage.report_id(), ReportId::CATCH_OF_THE_DAY);
        assert_eq!(CatchOfTheDayMessage.get_key(), "catchOfTheDay");
    }
}
