use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_trap_door::ReportTrapDoor;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `TrapDoorMessage.java`.
pub struct TrapDoorMessage;

impl ReportMessage for TrapDoorMessage {
    type Report = ReportTrapDoor;

    fn report_id(&self) -> ReportId {
        ReportId::TRAP_DOOR
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = report.get_player_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent();
        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Trapdoor Roll [ {} ]", report.get_roll()));
        print_player(status_report, game, indent, false, player);
        if report.is_escaped() {
            status_report.println_indent_style(indent, TextStyle::NONE, " escapes the trapdoor.");
        } else {
            status_report.println_indent_style(indent, TextStyle::NONE, " falls down the trapdoor.");
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

    fn make_player(id: &str, name: &str) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![make_player("p1", "Faller")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn escaped_trapdoor() {
        let game = make_game();
        let report = ReportTrapDoor::new(Some("p1".into()), 4, true);
        let mut status_report = StatusReport::new();
        TrapDoorMessage.render(&mut status_report, &game, &report);
        // run0 = roll text, run1 = println terminator, run2 = player name, run3 = outcome text.
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Trapdoor Roll [ 4 ]"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Faller"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" escapes the trapdoor."));
    }

    #[test]
    fn falls_down_trapdoor() {
        let game = make_game();
        let report = ReportTrapDoor::new(Some("p1".into()), 2, false);
        let mut status_report = StatusReport::new();
        TrapDoorMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" falls down the trapdoor."));
    }

    #[test]
    fn missing_player_still_reports_roll_and_outcome() {
        let game = make_game();
        let report = ReportTrapDoor::new(None, 6, true);
        let mut status_report = StatusReport::new();
        TrapDoorMessage.render(&mut status_report, &game, &report);
        // no player run since print_player is a no-op for None: roll text + terminator + outcome text + terminator.
        assert_eq!(status_report.rendered_runs.len(), 4);
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some(" escapes the trapdoor."));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(TrapDoorMessage.report_id(), ReportId::TRAP_DOOR);
        assert_eq!(TrapDoorMessage.get_key(), "trapDoor");
    }
}
