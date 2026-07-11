use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_mechanics::pass_result::PassResult;
use ffb_model::enums::PassingDistance;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_pass_roll::ReportPassRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `PassRollMessage.java` (bb2025).
pub struct PassRollMessage;

/// Java: `PassResult` is read directly off `report.getResult()` as an enum; the Rust
/// `ReportPassRoll::result` field only retains its `getName()` string, so this parses it
/// back. Only the values this message actually branches on are handled.
fn parse_pass_result(name: &str) -> Option<PassResult> {
    [
        PassResult::FUMBLE,
        PassResult::SAVED_FUMBLE,
        PassResult::WILDLY_INACCURATE,
        PassResult::INACCURATE,
        PassResult::ACCURATE,
    ]
    .into_iter()
    .find(|r| r.get_name() == name)
}

/// Java: `report.getPassingDistance()` is a `PassingDistance` enum value; the Rust
/// `ReportPassRoll::passing_distance` field only retains a string (populated from Rust's
/// `{:?}` Debug format at the report-creation call sites, e.g.
/// `crates/ffb-engine/src/step/bb2025/pass/step_pass.rs`), so this parses it back. Falls
/// back to `PassingDistance::from_name`'s display-name matching too, in case the string
/// ever comes from that format instead.
fn parse_passing_distance(name: &str) -> Option<PassingDistance> {
    PassingDistance::from_name(name).or_else(|| {
        [
            PassingDistance::QuickPass,
            PassingDistance::ShortPass,
            PassingDistance::LongPass,
            PassingDistance::LongBomb,
            PassingDistance::PassToPartner,
        ]
        .into_iter()
        .find(|d| format!("{d:?}") == name)
    })
}

impl ReportMessage for PassRollMessage {
    type Report = ReportPassRoll;

    fn report_id(&self) -> ReportId {
        ReportId::PASS_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let thrower = report.get_player_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent();

        let Some(thrower) = thrower else { return };

        if !report.is_re_rolled() {
            print_player(status_report, game, indent, true, Some(thrower));
            let catcher = game
                .pass_coordinate
                .and_then(|c| game.field_model.player_at(c))
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

        // java: `PassModifierFactory pmf = game.getFactory(Factory.PASS_MODIFIER);
        // report.hasRollModifier(pmf.forName("Nerves of Steel"))` — `ReportPassRoll` only
        // retains roll modifier *names* (Vec<String>), so "Nerves of Steel" presence is
        // checked directly by name instead of resolving a `PassModifier` instance through a
        // (not-yet-ported) `PassModifierFactory`.
        if report.get_roll_modifiers().iter().any(|m| m == "Nerves of Steel") {
            // java: `Player<?> player = game.getActingPlayer().getPlayer();
            // statusReport.report(new ReportNervesOfSteel(player.getId(), ...))` — no bb2025
            // `NervesOfSteelMessage` render function exists yet in the Rust client (only
            // PascalCase placeholder stubs exist for bb2016/mixed under
            // `crates/ffb-client/src/client/report/`), so there is nothing to dispatch the
            // constructed `ReportNervesOfSteel` to yet.
            let _player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        }

        let mechanic = ffb_engine::mechanic::pass_mechanic_for(game.rules);
        status_report.println_indent_style(
            indent + 1,
            TextStyle::ROLL,
            &mechanic.format_report_roll(report.get_roll(), thrower),
        );
        print_player(status_report, game, indent + 2, false, Some(thrower));

        let result = report.get_result().and_then(parse_pass_result);
        let mut needed_roll: Option<String> = None;

        if result == Some(PassResult::ACCURATE) || (result == Some(PassResult::INACCURATE) && report.is_hail_mary_pass()) {
            if report.is_bomb() {
                status_report.println_indent(indent + 2, " throws the bomb successfully.");
            } else {
                status_report.println_indent(indent + 2, " passes the ball.");
            }
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            if result == Some(PassResult::SAVED_FUMBLE) || result == Some(PassResult::FUMBLE) {
                if report.is_bomb() {
                    status_report.println_indent(indent + 2, " fumbles the bomb.");
                } else {
                    status_report.println_indent(indent + 2, " fumbles the ball.");
                }
            } else {
                status_report.println_indent(indent + 2, " misses the throw.");
            }
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }

        if let Some(mut needed_roll) = needed_roll {
            let mut formatted_modifiers = status_report.format_roll_modifiers(report.get_roll_modifiers());
            if let Some(stat_based) = report.get_stat_based_roll_modifier() {
                // java: `formattedModifiers += " + " + statBasedRollModifier.getModifier() + " "
                // + statBasedRollModifier.getReportString()` — `ReportPassRoll` only retains the
                // modifier's name string (see `report_pass_roll.rs` doc comment), not its
                // numeric magnitude or a separate report string, so only the name is appended.
                formatted_modifiers.push_str(&format!(" + {stat_based}"));
            }
            if let Some(distance) = report.get_passing_distance().and_then(parse_passing_distance) {
                needed_roll.push_str(&mechanic.format_roll_requirement(distance, &formatted_modifiers, thrower));
            }
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
    use ffb_model::types::FieldCoordinate;

    fn make_player(id: &str, name: &str, gender: PlayerGender) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            gender,
            player_type: PlayerType::default(),
            passing: 3,
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
        let home = make_team("home", vec![make_player("t1", "Thrower", PlayerGender::Male)]);
        let away = make_team("away", vec![make_player("c1", "Catcher", PlayerGender::Female)]);
        Game::new(home, away, Rules::Bb2025)
    }

    fn make_report(successful: bool, result: &str, roll: i32, minimum_roll: i32, re_rolled: bool) -> ReportPassRoll {
        ReportPassRoll::new(
            Some("t1".into()),
            successful,
            roll,
            minimum_roll,
            re_rolled,
            vec![],
            Some("ShortPass".into()),
            false,
            Some(result.into()),
            false,
            None,
        )
    }

    #[test]
    fn accurate_pass_reports_success_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = make_report(true, "ACCURATE", 4, 3, false);
        PassRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("passes the ball.")));
        assert!(texts.iter().any(|t| t.contains("Succeeded on a roll of 3+")));
    }

    #[test]
    fn fumble_reports_failure_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = make_report(false, "FUMBLE", 1, 3, false);
        PassRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("fumbles the ball.")));
        assert!(texts.iter().any(|t| t.contains("Roll a 3+ to succeed")));
    }

    #[test]
    fn re_rolled_pass_skips_action_line_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = make_report(true, "ACCURATE", 4, 3, true);
        PassRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(!texts.iter().any(|t| t.contains("passes the ball to")));
        assert!(!texts.iter().any(|t| t.contains("Succeeded on a roll of")));
        assert!(texts.iter().any(|t| t.contains("passes the ball.")));
    }

    #[test]
    fn bomb_at_catcher_reports_bomb_wording() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(1, 1));
        game.field_model.set_player_coordinate("c1", FieldCoordinate::new(1, 1));
        let report = ReportPassRoll::new(
            Some("t1".into()),
            true,
            4,
            3,
            false,
            vec![],
            Some("ShortPass".into()),
            true,
            Some("ACCURATE".into()),
            false,
            None,
        );
        PassRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("throws a bomb at ")));
        assert!(texts.iter().any(|t| t.contains("throws the bomb successfully.")));
    }

    #[test]
    fn missing_thrower_renders_nothing() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPassRoll::new(
            Some("nonexistent".into()),
            true,
            4,
            3,
            false,
            vec![],
            Some("ShortPass".into()),
            false,
            Some("ACCURATE".into()),
            false,
            None,
        );
        PassRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn report_id_is_pass_roll() {
        assert_eq!(PassRollMessage.report_id(), ReportId::PASS_ROLL);
    }
}
