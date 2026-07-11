use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_getting_even_roll::ReportGettingEvenRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `GettingEvenRollMessage.java`.
pub struct GettingEvenRollMessage;

impl ReportMessage for GettingEvenRollMessage {
    type Report = ReportGettingEvenRoll;

    fn report_id(&self) -> ReportId {
        ReportId::GETTING_EVEN_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let mut needed_roll: Option<String> = None;
        let player = report.get_player_id().and_then(|id| game.player(id));
        status_report.println_indent_style(
            status_report.get_indent(),
            TextStyle::ROLL,
            &format!("Getting Even Roll [ {} ]", report.get_roll()),
        );
        let indent = status_report.get_indent() + 1;
        print_player(status_report, game, indent, false, player);

        if report.is_successful() {
            status_report.println_indent(indent, &format!(" gains hatred towards players of type '{}'.", report.get_keyword()));
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(indent, &format!(" remains peaceful towards players of type '{}'.", report.get_keyword()));
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }
        if let Some(needed_roll) = needed_roll {
            status_report.println_indent_style(indent, TextStyle::NEEDED_ROLL, &needed_roll);
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
        let home = make_team("home", vec![make_player("p1", "Hater")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn successful_gains_hatred_with_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportGettingEvenRoll::new(Some("p1".into()), true, 5, 4, false, "Elf".into());
        GettingEvenRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("gains hatred towards players of type 'Elf'.")));
        assert!(texts.iter().any(|t| t.contains("Succeeded on a roll of 4+")));
    }

    #[test]
    fn unsuccessful_remains_peaceful_with_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportGettingEvenRoll::new(Some("p1".into()), false, 2, 4, false, "Elf".into());
        GettingEvenRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("remains peaceful towards players of type 'Elf'.")));
        assert!(texts.iter().any(|t| t.contains("Roll a 4+ to succeed")));
    }

    #[test]
    fn re_rolled_suppresses_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportGettingEvenRoll::new(Some("p1".into()), true, 5, 4, true, "Elf".into());
        GettingEvenRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(!texts.iter().any(|t| t.contains("Succeeded on a roll of")));
    }

    #[test]
    fn needed_roll_uses_needed_roll_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportGettingEvenRoll::new(Some("p1".into()), true, 5, 4, false, "Elf".into());
        GettingEvenRollMessage.render(&mut status_report, &game, &report);
        let run = status_report.rendered_runs.iter().find(|r| r.text.as_deref().is_some_and(|t| t.contains("Succeeded"))).unwrap();
        assert_eq!(run.text_style, Some(TextStyle::NEEDED_ROLL));
    }

    #[test]
    fn report_id_is_getting_even_roll() {
        assert_eq!(GettingEvenRollMessage.report_id(), ReportId::GETTING_EVEN_ROLL);
    }
}
