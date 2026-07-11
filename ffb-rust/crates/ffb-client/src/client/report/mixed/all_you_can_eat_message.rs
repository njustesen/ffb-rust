use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PlayerGender;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_all_you_can_eat_roll::ReportAllYouCanEatRoll;
use ffb_model::report::report_id::ReportId;

/// Java: `PlayerGender.getDative()` not yet in ffb-model enum (him/her/them/it).
fn dative(gender: PlayerGender) -> &'static str {
    match gender {
        PlayerGender::Male => "him",
        PlayerGender::Female => "her",
        PlayerGender::Nonbinary => "them",
        PlayerGender::Neutral => "it",
    }
}

/// 1:1 translation of `AllYouCanEatMessage.java`.
pub struct AllYouCanEatMessage;

impl ReportMessage for AllYouCanEatMessage {
    type Report = ReportAllYouCanEatRoll;

    fn report_id(&self) -> ReportId {
        ReportId::ALL_YOU_CAN_EAT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = report.get_player_id().and_then(|id| game.player(id));

        if !report.is_re_rolled() {
            print_player(status_report, game, indent, true, player);
            let gender_word = player.map(|p| dative(p.gender)).unwrap_or("");
            status_report.println_indent_style(indent, TextStyle::BOLD, &format!(" hopes the ref did not spot {}.", gender_word));
        }

        status_report.println_indent_style(indent + 1, TextStyle::ROLL, &format!("All You Can Eat Roll [ {} ]", report.get_roll()));

        print_player(status_report, game, indent, false, player);

        let message = if report.is_successful() { " goes unnoticed." } else { " is spotted." };
        status_report.println_indent_style(indent + 2, TextStyle::NONE, message);

        if !report.is_re_rolled() {
            status_report.println_indent_style(
                indent + 2,
                TextStyle::NEEDED_ROLL,
                &format!("Roll a {}+ to succeed", report.get_minimum_roll()),
            );
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
        let player = Player { id: "p1".into(), name: "Grobnik".into(), gender: PlayerGender::Male, ..Player::default() };
        Game::new(make_team("home", vec![player]), make_team("away", vec![]), Rules::Bb2020)
    }

    #[test]
    fn successful_and_not_re_rolled_shows_greeting_and_success() {
        let game = make_game();
        let report = ReportAllYouCanEatRoll::new(Some("p1".into()), true, 4, 2, false);
        let mut sr = StatusReport::new();
        AllYouCanEatMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Grobnik"));
        assert_eq!(sr.rendered_runs[1].text.as_deref(), Some(" hopes the ref did not spot him."));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some("All You Can Eat Roll [ 4 ]"));
        assert_eq!(sr.rendered_runs[5].text.as_deref(), Some("Grobnik"));
        assert_eq!(sr.rendered_runs[6].text.as_deref(), Some(" goes unnoticed."));
        assert_eq!(sr.rendered_runs[8].text.as_deref(), Some("Roll a 2+ to succeed"));
    }

    #[test]
    fn unsuccessful_shows_spotted() {
        let game = make_game();
        let report = ReportAllYouCanEatRoll::new(Some("p1".into()), false, 1, 2, false);
        let mut sr = StatusReport::new();
        AllYouCanEatMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[6].text.as_deref(), Some(" is spotted."));
    }

    #[test]
    fn re_rolled_skips_greeting_and_needed_roll() {
        let game = make_game();
        let report = ReportAllYouCanEatRoll::new(Some("p1".into()), true, 4, 2, true);
        let mut sr = StatusReport::new();
        AllYouCanEatMessage.render(&mut sr, &game, &report);
        // roll println (2) + player print (1) + success println (2) = 5, no greeting, no needed roll.
        assert_eq!(sr.rendered_runs.len(), 5);
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(AllYouCanEatMessage.report_id(), ReportId::ALL_YOU_CAN_EAT);
        assert_eq!(AllYouCanEatMessage.get_key(), "allYouCanEat");
    }
}
