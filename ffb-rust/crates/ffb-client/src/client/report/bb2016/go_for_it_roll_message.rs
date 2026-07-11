use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_go_for_it_roll::ReportGoForItRoll;
use ffb_model::report::report_id::ReportId;

pub struct GoForItRollMessage;

impl ReportMessage for GoForItRollMessage {
    type Report = ReportGoForItRoll;

    fn report_id(&self) -> ReportId {
        ReportId::GO_FOR_IT_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let mut needed_roll: Option<String> = None;
        let status = format!("Go For It Roll [ {} ]", report.base.get_roll());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        print_player(status_report, game, indent + 1, false, player);
        if report.base.is_successful() {
            status_report.println_indent(indent + 1, " goes for it!");
            if !report.base.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.base.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(indent + 1, " trips while going for it.");
            if !report.base.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.base.get_minimum_roll()));
            }
        }
        if let Some(mut needed_roll) = needed_roll {
            needed_roll.push_str(" (Roll");
            needed_roll.push_str(&status_report.format_roll_modifiers(report.base.get_roll_modifiers()));
            needed_roll.push_str(" > ");
            needed_roll.push_str(&(report.base.get_minimum_roll() - 1).to_string());
            needed_roll.push_str(").");
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

    fn make_team(id: &str) -> Team {
        Team { id: id.into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        let mut home = make_team("home");
        home.players.push(Player {
            id: "p1".into(), name: "Grubb".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        let mut game = Game::new(home, make_team("away"), Rules::Bb2016);
        game.acting_player.player_id = Some("p1".into());
        game
    }

    #[test]
    fn get_key_is_go_for_it_roll() {
        assert_eq!(GoForItRollMessage.get_key(), "goForItRoll");
    }

    #[test]
    fn successful_roll_reports_goes_for_it_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportGoForItRoll::new(Some("p1".into()), true, 3, 2, false, vec![]);
        GoForItRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" goes for it!"));
        let needed = status_report.rendered_runs.iter().find(|r| r.text_style == Some(TextStyle::NEEDED_ROLL)).unwrap();
        assert_eq!(needed.text.as_deref(), Some("Succeeded on a roll of 2+ (Roll > 1)."));
    }

    #[test]
    fn failed_roll_trips() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportGoForItRoll::new(Some("p1".into()), false, 1, 2, false, vec![]);
        GoForItRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" trips while going for it."));
    }

    #[test]
    fn re_rolled_has_no_needed_roll_line() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportGoForItRoll::new(Some("p1".into()), false, 1, 2, true, vec![]);
        GoForItRollMessage.render(&mut status_report, &game, &report);
        assert!(!status_report.rendered_runs.iter().any(|r| r.text_style == Some(TextStyle::NEEDED_ROLL)));
    }
}
