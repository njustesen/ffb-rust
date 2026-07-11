use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_engine::mechanic::pass_mechanic_for;
use ffb_mechanics::modifiers::pass_modifier_factory::PassModifierFactory;
use ffb_mechanics::pass_result::PassResult;
use ffb_model::enums::PassingDistance;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::report::mixed::report_pass_roll::ReportPassRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `PassRollMessage.java`.
pub struct PassRollMessage;

/// Java: `PassingDistance` values are serialized via `format!("{:?}", dist)` (Rust `Debug`
/// derive prints the bare variant name) when `ReportPassRoll` is constructed in the engine
/// steps (see `step_pass.rs`). Parse that representation back into the enum so
/// `PassMechanic::format_roll_requirement` (which expects a `PassingDistance`) can be called.
fn parse_passing_distance(name: &str) -> Option<PassingDistance> {
    match name {
        "QuickPass" => Some(PassingDistance::QuickPass),
        "ShortPass" => Some(PassingDistance::ShortPass),
        "LongPass" => Some(PassingDistance::LongPass),
        "LongBomb" => Some(PassingDistance::LongBomb),
        "PassToPartner" => Some(PassingDistance::PassToPartner),
        _ => None,
    }
}

impl ReportMessage for PassRollMessage {
    type Report = ReportPassRoll;

    fn report_id(&self) -> ReportId {
        ReportId::PASS_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let mechanic = pass_mechanic_for(game.rules);
        let thrower: Option<&Player> = report.get_player_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent();

        if !report.is_re_rolled() {
            print_player(status_report, game, indent, true, thrower);
            let catcher: Option<&Player> = game
                .pass_coordinate
                .and_then(|coord| game.field_model.player_at(coord))
                .and_then(|id| game.player(id));
            if report.is_hail_mary_pass() {
                if report.is_bomb() {
                    status_report.println_indent_style(indent, TextStyle::BOLD, " throws a Hail Mary bomb:");
                } else {
                    status_report.println_indent_style(indent, TextStyle::BOLD, " throws a Hail Mary pass:");
                }
            } else if let Some(catcher) = catcher {
                if report.is_bomb() {
                    status_report.print_indent_style(indent, TextStyle::BOLD, " throws a bomb at ");
                } else {
                    status_report.print_indent_style(indent, TextStyle::BOLD, " passes the ball to ");
                }
                print_player(status_report, game, indent, true, Some(catcher));
                status_report.println_indent_style(indent, TextStyle::BOLD, ":");
            } else if report.is_bomb() {
                status_report.println_indent_style(indent, TextStyle::BOLD, " throws a bomb to an empty field:");
            } else {
                status_report.println_indent_style(indent, TextStyle::BOLD, " passes the ball to an empty field:");
            }
        }

        let pmf = PassModifierFactory::for_rules(game.rules);
        if pmf
            .for_name("Nerves of Steel")
            .is_some_and(|modifier| report.get_roll_modifiers().contains(&modifier.get_name().to_string()))
        {
            // java: `statusReport.report(new ReportNervesOfSteel(player.getId(), ...))` — the
            // headless StatusReport sink has no reachable equivalent for pushing a *new*,
            // separately-rendered report from inside another message's render() (there is no
            // NervesOfSteelMessage renderer wired up in this batch, and StatusReport::report()
            // requires the caller to already hold the render function for the target report
            // type). Left unimplemented; no fabricated substitute added.
        }

        if let Some(thrower) = thrower {
            let mut status = String::new();
            status.push_str(&mechanic.format_report_roll(report.get_roll(), thrower));
            status_report.println_indent_style(indent + 1, TextStyle::ROLL, &status);

            print_player(status_report, game, indent + 2, false, Some(thrower));
            let result = report.get_result();
            let mut needed_roll: Option<String> = None;
            if result == Some(PassResult::ACCURATE.get_name())
                || (result == Some(PassResult::INACCURATE.get_name()) && report.is_hail_mary_pass())
            {
                if report.is_bomb() {
                    status_report.println_indent(indent + 2, " throws the bomb successfully.");
                } else {
                    status_report.println_indent(indent + 2, " passes the ball.");
                }
                if !report.is_re_rolled() {
                    needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
                }
            } else {
                if result == Some(PassResult::SAVED_FUMBLE.get_name()) {
                    if report.is_bomb() {
                        status_report.println_indent(indent + 2, " holds on to the bomb and puts out the fuse.");
                    } else {
                        status_report.println_indent(indent + 2, " holds on to the ball.");
                    }
                } else if result == Some(PassResult::FUMBLE.get_name()) {
                    if report.is_bomb() {
                        status_report.println_indent(indent + 2, " fumbles the bomb.");
                    } else {
                        status_report.println_indent(indent + 2, " fumbles the ball.");
                    }
                } else if result == Some(PassResult::WILDLY_INACCURATE.get_name()) {
                    status_report.println_indent(indent + 2, " lets the throw deviate.");
                } else {
                    status_report.println_indent(indent + 2, " misses the throw.");
                }
                if !report.is_re_rolled() {
                    needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
                }
            }

            if let Some(mut needed_roll) = needed_roll {
                let mut formatted_modifiers = status_report.format_roll_modifiers(report.get_roll_modifiers());
                if let Some(stat_based_roll_modifier) = report.get_stat_based_roll_modifier() {
                    // java: `statBasedRollModifier.getModifier() + " " + statBasedRollModifier.getReportString()`
                    // — `ReportPassRoll.stat_based_roll_modifier` only carries the resolved
                    // name string (see report_pass_roll.rs), not the original
                    // `StatBasedRollModifier` struct with its numeric `getModifier()` value, so
                    // the magnitude cannot be reconstructed here; only the name is appended.
                    formatted_modifiers.push_str(" + ");
                    formatted_modifiers.push_str(stat_based_roll_modifier);
                }
                if let Some(distance) = report.get_passing_distance().and_then(parse_passing_distance) {
                    needed_roll.push_str(&mechanic.format_roll_requirement(distance, &formatted_modifiers, thrower));
                }
                status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &needed_roll);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::types::FieldCoordinate;

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(),
            name: format!("Player {id}"),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: "Team".into(),
            race: String::new(),
            roster_id: String::new(),
            coach: String::new(),
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
        let mut game = Game::new(
            make_team("home", vec![make_player("thrower")]),
            make_team("away", vec![make_player("catcher")]),
            Rules::Bb2020,
        );
        game.pass_coordinate = Some(FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("catcher", FieldCoordinate::new(5, 5));
        game
    }

    fn texts(status_report: &StatusReport) -> Vec<&str> {
        status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect()
    }

    #[test]
    fn accurate_pass_to_catcher() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportPassRoll::new(
            Some("thrower".into()),
            true,
            4,
            2,
            false,
            vec![],
            Some("ShortPass".into()),
            false,
            Some("ACCURATE".into()),
            false,
            None,
        );
        PassRollMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.iter().any(|s| s.contains("passes the ball to ")));
        assert!(t.iter().any(|s| s.contains(" passes the ball.")));
        assert!(t.iter().any(|s| s.contains("Succeeded on a roll of 2+")));
    }

    #[test]
    fn bomb_fumble() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportPassRoll::new(
            Some("thrower".into()),
            false,
            1,
            2,
            false,
            vec![],
            Some("ShortPass".into()),
            true,
            Some("FUMBLE".into()),
            false,
            None,
        );
        PassRollMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.iter().any(|s| s.contains("throws a bomb at")));
        assert!(t.iter().any(|s| s.contains("fumbles the bomb.")));
        assert!(t.iter().any(|s| s.contains("Roll a 2+ to succeed")));
    }

    #[test]
    fn hail_mary_pass_inaccurate_counts_as_success() {
        let mut game = make_game();
        game.pass_coordinate = None;
        let mut status_report = StatusReport::new();
        let report = ReportPassRoll::new(
            Some("thrower".into()),
            true,
            2,
            2,
            false,
            vec![],
            None,
            false,
            Some("INACCURATE".into()),
            true,
            None,
        );
        PassRollMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.iter().any(|s| s.contains("throws a Hail Mary pass:")));
        assert!(t.iter().any(|s| s.contains(" passes the ball.")));
    }

    #[test]
    fn re_rolled_skips_intro_and_needed_roll() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportPassRoll::new(
            Some("thrower".into()),
            true,
            4,
            2,
            true,
            vec![],
            Some("ShortPass".into()),
            false,
            Some("ACCURATE".into()),
            false,
            None,
        );
        PassRollMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(!t.iter().any(|s| s.contains("passes the ball to ")));
        assert!(!t.iter().any(|s| s.contains("Succeeded on a roll")));
    }

    #[test]
    fn empty_field_pass() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(10, 10));
        let mut status_report = StatusReport::new();
        let report = ReportPassRoll::new(
            Some("thrower".into()),
            true,
            4,
            2,
            false,
            vec![],
            Some("ShortPass".into()),
            false,
            Some("ACCURATE".into()),
            false,
            None,
        );
        PassRollMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.iter().any(|s| s.contains("passes the ball to an empty field:")));
    }
}
