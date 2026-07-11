use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2020::report_officious_ref_roll::ReportOfficiousRefRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `OfficiousRefRollMessage.java`.
pub struct OfficiousRefRollMessage;

impl ReportMessage for OfficiousRefRollMessage {
    type Report = ReportOfficiousRefRoll;

    fn report_id(&self) -> ReportId {
        ReportId::OFFICIOUS_REF_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        status_report.println_indent_style(
            indent,
            TextStyle::ROLL,
            &format!("Officious Ref Effect Roll [ {} ]", report.get_roll()),
        );
        let player = game.player(report.get_player_id());
        print_player(status_report, game, indent + 1, false, player);
        let message = if report.get_roll() == 1 { " is sent off." } else { " is stunned" };
        status_report.println_indent_style(indent + 1, TextStyle::NONE, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
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
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game_with_player() -> Game {
        let mut home = make_team("home");
        home.players.push(Player { id: "p1".into(), name: "Bob".into(), ..Player::default() });
        Game::new(home, make_team("away"), Rules::Bb2020)
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(OfficiousRefRollMessage.report_id(), ReportId::OFFICIOUS_REF_ROLL);
    }

    #[test]
    fn roll_of_one_sends_player_off() {
        let mut sr = StatusReport::new();
        let game = make_game_with_player();
        let report = ReportOfficiousRefRoll::new(1, "p1".to_string());
        OfficiousRefRollMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" is sent off.")));
    }

    #[test]
    fn other_roll_stuns_player() {
        let mut sr = StatusReport::new();
        let game = make_game_with_player();
        let report = ReportOfficiousRefRoll::new(4, "p1".to_string());
        OfficiousRefRollMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" is stunned")));
    }

    #[test]
    fn roll_value_is_reported() {
        let mut sr = StatusReport::new();
        let game = make_game_with_player();
        let report = ReportOfficiousRefRoll::new(6, "p1".to_string());
        OfficiousRefRollMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Officious Ref Effect Roll [ 6 ]")));
    }
}
