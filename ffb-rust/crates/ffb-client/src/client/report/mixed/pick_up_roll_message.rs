use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_pickup_roll::ReportPickupRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `PickUpRollMessage.java`.
pub struct PickUpRollMessage;

impl ReportMessage for PickUpRollMessage {
    type Report = ReportPickupRoll;

    fn report_id(&self) -> ReportId {
        ReportId::PICK_UP_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = report.get_player_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent();
        let mut needed_roll: Option<String> = None;

        if !report.is_re_rolled() {
            print_player(status_report, game, indent, true, player);
            status_report.println_indent_style(indent, TextStyle::BOLD, " tries to pick up the ball:");
        }

        status_report.println_indent_style(indent + 1, TextStyle::ROLL, &format!("Pickup Roll [ {} ]", report.get_roll()));
        print_player(status_report, game, indent + 2, false, player);
        if report.is_successful() {
            status_report.println_indent(indent + 2, " picks up the ball.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(indent + 2, " drops the ball.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }

        if let Some(mut needed_roll) = needed_roll {
            // java: AgilityMechanic.formatPickupResult(report, player) resolves the edition's
            // AgilityMechanic via game.getRules().getFactory(Factory.MECHANIC) and formats the
            // full RollModifier set (name + signed numeric value). The Rust report data model
            // only retains resolved modifier name strings (ReportSkillRoll.roll_modifier_names),
            // so the exact numeric formatting can't be reconstructed here; approximated with
            // StatusReport::format_roll_modifiers (name-only " - <name>" suffixes).
            needed_roll.push_str(&status_report.format_roll_modifiers(report.get_roll_modifiers()));
            status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &needed_roll);
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
    fn first_attempt_success_reports_intro_and_needed_roll() {
        let game = make_game();
        let report = ReportPickupRoll::new(Some("p1".into()), true, 5, 3, false, vec![]);
        let mut status_report = StatusReport::new();
        PickUpRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Grobnik"));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" tries to pick up the ball:"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some("Pickup Roll [ 5 ]"));
        assert_eq!(status_report.rendered_runs[5].text.as_deref(), Some("Grobnik"));
        assert_eq!(status_report.rendered_runs[6].text.as_deref(), Some(" picks up the ball."));
        assert_eq!(status_report.rendered_runs[8].text.as_deref(), Some("Succeeded on a roll of 3+"));
        assert_eq!(status_report.rendered_runs[8].text_style, Some(TextStyle::NEEDED_ROLL));
    }

    #[test]
    fn first_attempt_failure_reports_drop_and_needed_roll() {
        let game = make_game();
        let report = ReportPickupRoll::new(Some("p1".into()), false, 1, 4, false, vec![]);
        let mut status_report = StatusReport::new();
        PickUpRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[6].text.as_deref(), Some(" drops the ball."));
        assert_eq!(status_report.rendered_runs[8].text.as_deref(), Some("Roll a 4+ to succeed"));
    }

    #[test]
    fn re_rolled_skips_intro_and_needed_roll() {
        let game = make_game();
        let report = ReportPickupRoll::new(Some("p1".into()), true, 5, 3, true, vec![]);
        let mut status_report = StatusReport::new();
        PickUpRollMessage.render(&mut status_report, &game, &report);
        // No intro line, no needed-roll line: only the "Pickup Roll" header + player + result.
        assert_eq!(status_report.rendered_runs.len(), 5);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Pickup Roll [ 5 ]"));
    }

    #[test]
    fn roll_modifiers_appended_to_needed_roll() {
        let game = make_game();
        let report = ReportPickupRoll::new(Some("p1".into()), true, 5, 3, false, vec!["TackleZone".into()]);
        let mut status_report = StatusReport::new();
        PickUpRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[8].text.as_deref(), Some("Succeeded on a roll of 3+ - TackleZone"));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(PickUpRollMessage.report_id(), ReportId::PICK_UP_ROLL);
        assert_eq!(PickUpRollMessage.get_key(), "pickUpRoll");
    }
}
