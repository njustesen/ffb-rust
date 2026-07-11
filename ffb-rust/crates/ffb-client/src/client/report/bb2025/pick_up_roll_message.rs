use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_pickup_roll::ReportPickupRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `PickUpRollMessage.java` (bb2025).
pub struct PickUpRollMessage;

impl ReportMessage for PickUpRollMessage {
    type Report = ReportPickupRoll;

    fn report_id(&self) -> ReportId {
        ReportId::PICK_UP_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = report.get_player_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent();

        if !report.is_re_rolled() {
            print_player(status_report, game, indent, true, player);
            status_report.println_indent_style(indent, TextStyle::BOLD, " tries to pick up the ball:");
        }

        status_report.println_indent_style(indent + 1, TextStyle::ROLL, &format!("Pickup Roll [ {} ]", report.get_roll()));
        print_player(status_report, game, indent + 2, false, player);

        let mut needed_roll: Option<String> = None;
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
            // java: `AgilityMechanic mechanic = ...; neededRoll.append(mechanic.formatPickupResult(report, player))`.
            // `AgilityMechanic::format_pickup_result` on the Rust side (crates/ffb-mechanics/src/
            // agility_mechanic.rs) takes `&[RollModifier]` (name + numeric magnitude) because
            // Java's `formatResult` needs the signed magnitude to print "+N"/"-N" per modifier.
            // `ReportPickupRoll`/`ReportSkillRoll` only retain modifier *names* on the Rust side
            // (`get_roll_modifiers() -> &[String]`, see report_skill_roll.rs), so a real
            // `RollModifier` can't be reconstructed without inventing a magnitude. This inlines
            // the same shape `format_pickup_result` -> `format_result` produces
            // (`crates/ffb-mechanics/src/bb2025/agility_mechanic.rs`) — base roll (2 if
            // Secure the Ball is in use, else agility with modifiers) plus the modifier names —
            // using `StatusReport::format_roll_modifiers` (name-only) in place of the
            // magnitude-aware `AgilityMechanic::format_roll_modifiers`.
            let base_roll = if report.is_secure_the_ball() {
                2
            } else {
                player.map(|p| p.agility_with_modifiers()).unwrap_or(0)
            };
            needed_roll.push_str(&format!(
                " (Roll{}  >= {}+)",
                status_report.format_roll_modifiers(report.base.get_roll_modifiers()),
                base_roll.max(2)
            ));
            status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str, gender: PlayerGender, agility: i32) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            gender,
            player_type: PlayerType::default(),
            agility,
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
        let home = make_team("home", vec![make_player("p1", "Runner", PlayerGender::Male, 3)]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn successful_pickup_reports_success_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPickupRoll::new(Some("p1".into()), true, 4, 3, false, vec![], false);
        PickUpRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("tries to pick up the ball:")));
        assert!(texts.iter().any(|t| t.contains("Pickup Roll [ 4 ]")));
        assert!(texts.iter().any(|t| t.contains("picks up the ball.")));
        assert!(texts.iter().any(|t| t.contains("Succeeded on a roll of 3+")));
    }

    #[test]
    fn failed_pickup_reports_drop_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPickupRoll::new(Some("p1".into()), false, 1, 3, false, vec![], false);
        PickUpRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("drops the ball.")));
        assert!(texts.iter().any(|t| t.contains("Roll a 3+ to succeed")));
    }

    #[test]
    fn re_rolled_pickup_skips_intro_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPickupRoll::new(Some("p1".into()), true, 4, 3, true, vec![], false);
        PickUpRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(!texts.iter().any(|t| t.contains("tries to pick up the ball:")));
        assert!(!texts.iter().any(|t| t.contains("Succeeded on a roll of")));
        assert!(texts.iter().any(|t| t.contains("picks up the ball.")));
    }

    #[test]
    fn secure_the_ball_uses_base_roll_of_two() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPickupRoll::new(Some("p1".into()), true, 2, 2, false, vec![], true);
        PickUpRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains(">= 2+")));
    }

    #[test]
    fn roll_modifiers_are_included_by_name() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPickupRoll::new(Some("p1".into()), false, 1, 3, false, vec!["TackleZone".into()], false);
        PickUpRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("TackleZone")));
    }

    #[test]
    fn report_id_is_pick_up_roll() {
        assert_eq!(PickUpRollMessage.report_id(), ReportId::PICK_UP_ROLL);
    }
}
