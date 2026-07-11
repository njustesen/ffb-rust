use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::bb2025::report_steady_footing_roll::ReportSteadyFootingRoll;

/// 1:1 translation of `SteadyFootingRollMessage.java`.
pub struct SteadyFootingRollMessage;

impl ReportMessage for SteadyFootingRollMessage {
    type Report = ReportSteadyFootingRoll;

    fn report_id(&self) -> ReportId {
        ReportId::STEADY_FOOTING_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let mut needed_roll: Option<String> = None;
        let player = report.get_player_id().and_then(|id| game.player(id));
        let status = if report.get_roll() > 0 {
            format!("Steady Footing Roll [ {} ]", report.get_roll())
        } else {
            "New Steady Footing Result".to_string()
        };
        let indent = status_report.get_indent();
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        print_player(status_report, game, indent + 1, false, player);
        if report.is_successful() {
            let genitive = player.map(|p| p.gender.genitive()).unwrap_or("");
            let status = format!(" stays on {genitive}  feet.");
            status_report.println_indent(indent + 1, &status);
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(indent + 1, " fails to keep standing.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }
        if let Some(needed_roll) = needed_roll {
            status_report.println_indent_style(indent + 1, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            gender: PlayerGender::Male,
            player_type: PlayerType::default(),
            ..Default::default()
        }
    }

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
        let home = make_team("home", vec![make_player("p1", "Steady")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn report_id_is_steady_footing_roll() {
        assert_eq!(SteadyFootingRollMessage.report_id(), ReportId::STEADY_FOOTING_ROLL);
    }

    #[test]
    fn successful_roll_not_rerolled_shows_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSteadyFootingRoll::new(Some("p1".into()), true, 4, 3, false);
        SteadyFootingRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Steady Footing Roll [ 4 ]"));
        assert!(texts.iter().any(|t| t == " stays on his  feet."));
        assert!(texts.iter().any(|t| t == "Succeeded on a roll of 3+"));
    }

    #[test]
    fn failed_roll_rerolled_hides_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSteadyFootingRoll::new(Some("p1".into()), false, 2, 3, true);
        SteadyFootingRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " fails to keep standing."));
        assert!(!texts.iter().any(|t| t.contains("Roll a")));
    }

    #[test]
    fn zero_roll_uses_new_result_header() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSteadyFootingRoll::new(Some("p1".into()), false, 0, 3, false);
        SteadyFootingRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "New Steady Footing Result"));
        assert!(texts.iter().any(|t| t == "Roll a 3+ to succeed"));
    }
}
