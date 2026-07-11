use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_mechanics::pass_result::PassResult;
use ffb_model::enums::PassingDistance;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_throw_team_mate_roll::ReportThrowTeamMateRoll;
use ffb_model::report::report_id::ReportId;

/// java: `report.getPassingDistance()` returns a `PassingDistance` enum object in Java;
/// here the report stores only its (Java-side) SCREAMING_SNAKE_CASE enum-constant name as a
/// string, so it is parsed back into `ffb_model::enums::pass::PassingDistance` here.
fn parse_passing_distance(name: &str) -> Option<PassingDistance> {
    match name {
        "QUICK_PASS" => Some(PassingDistance::QuickPass),
        "SHORT_PASS" => Some(PassingDistance::ShortPass),
        "LONG_PASS" => Some(PassingDistance::LongPass),
        "LONG_BOMB" => Some(PassingDistance::LongBomb),
        "PASS_TO_PARTNER" => Some(PassingDistance::PassToPartner),
        _ => None,
    }
}

/// 1:1 translation of `ThrowTeamMateRollMessage.java`.
pub struct ThrowTeamMateRollMessage;

impl ReportMessage for ThrowTeamMateRollMessage {
    type Report = ReportThrowTeamMateRoll;

    fn report_id(&self) -> ReportId {
        ReportId::THROW_TEAM_MATE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let thrower = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let thrown_player = report.get_thrown_player_id().and_then(|id| game.player(id));
        let can_throw = thrower.map(|p| p.passing > 0).unwrap_or(false);

        if !report.is_re_rolled() {
            print_player(status_report, game, indent, true, thrower);
            if report.is_kick() {
                status_report.print_indent_style(indent, TextStyle::BOLD, " tries to kick ");
            } else {
                status_report.print_indent_style(indent, TextStyle::BOLD, " tries to throw ");
            }
            print_player(status_report, game, indent, true, thrown_player);
            status_report.println_indent_style(indent, TextStyle::BOLD, ":");
        }

        let status = if report.is_kick() {
            format!("Kick Team-Mate Roll [ {} ]", report.get_roll())
        } else {
            format!("Throw Team-Mate Roll [ {} ]", report.get_roll())
        };
        status_report.println_indent_style(indent + 1, TextStyle::ROLL, &status);

        print_player(status_report, game, indent + 2, false, thrower);
        if report.is_successful() {
            if report.is_kick() {
                status_report.print_indent(indent + 2, " kicks ");
            } else {
                status_report.print_indent(indent + 2, " throws ");
            }
            print_player(status_report, game, indent + 2, false, thrown_player);
            let mut status = String::from(" ");
            if report.get_pass_result() == Some(PassResult::ACCURATE.get_name()) {
                status.push_str("superbly");
            } else {
                status.push_str("successfully");
            }
            status.push('.');
            status_report.println_indent(indent + 2, &status);
        } else if report.get_pass_result() == Some(PassResult::WILDLY_INACCURATE.get_name()) {
            status_report.print_indent(indent + 2, " lets ");
            print_player(status_report, game, indent + 2, false, thrown_player);
            status_report.println_indent(indent + 2, " deviate.");
        } else {
            status_report.print_indent(indent + 2, " fumbles ");
            print_player(status_report, game, indent + 2, false, thrown_player);
            status_report.println_indent(indent + 2, ".");
        }

        let mut needed_roll: Option<String> = None;
        if report.is_successful() && !report.is_re_rolled() && can_throw {
            let mut s = format!("Succeeded on a roll of {}", report.get_minimum_roll());
            if report.is_kick() {
                s.push_str("+ to avoid a fumble or terrible kick");
            } else {
                s.push_str("+ to avoid a fumble or terrible throw");
            }
            needed_roll = Some(s);
        }
        if !report.is_successful() && !report.is_re_rolled() && can_throw {
            let mut s = format!("Roll a {}", report.get_minimum_roll());
            if report.is_kick() {
                s.push_str("+ to have at least a successful kick");
            } else {
                s.push_str("+ to have at least a successful throw");
            }
            needed_roll = Some(s);
        }
        if let Some(mut needed_roll) = needed_roll {
            needed_roll.push_str(" (Roll ");
            if let Some(passing_distance) = report.get_passing_distance().and_then(parse_passing_distance) {
                needed_roll.push_str(" - ");
                needed_roll.push_str(&format!("{} {}", passing_distance.modifier_2020(), passing_distance.name()));
                needed_roll.push_str(&status_report.format_roll_modifiers(report.get_roll_modifiers()));
                needed_roll.push_str(" > 1).");
            }
            status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &needed_roll);
        }

        status_report.set_indent(status_report.get_indent() + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, passing: i32) -> Player {
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
            passing,
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
            make_team("home", vec![make_player("thrower", 3)]),
            make_team("away", vec![make_player("thrown", 3)]),
            Rules::Bb2020,
        );
        game.acting_player.player_id = Some("thrower".into());
        game
    }

    #[test]
    fn kick_success_reports_superbly_and_bumps_indent() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportThrowTeamMateRoll::new(
            Some("thrower".into()), true, 6, 2, false, vec![],
            Some("SHORT_PASS".into()), Some("thrown".into()), Some("ACCURATE".into()), true,
        );
        ThrowTeamMateRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&" kicks "));
        assert!(texts.iter().any(|t| t.contains("superbly")));
        assert_eq!(status_report.get_indent(), 1);
    }

    #[test]
    fn throw_wildly_inaccurate_reports_deviate() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportThrowTeamMateRoll::new(
            Some("thrower".into()), false, 1, 2, false, vec![],
            Some("SHORT_PASS".into()), Some("thrown".into()), Some("WILDLY_INACCURATE".into()), false,
        );
        ThrowTeamMateRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&" lets "));
        assert!(texts.contains(&" deviate."));
    }

    #[test]
    fn throw_fumble_reports_fumbles() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportThrowTeamMateRoll::new(
            Some("thrower".into()), false, 1, 2, false, vec![],
            Some("SHORT_PASS".into()), Some("thrown".into()), Some("FUMBLE".into()), false,
        );
        ThrowTeamMateRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&" fumbles "));
    }

    #[test]
    fn re_rolled_skips_intro_line() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportThrowTeamMateRoll::new(
            Some("thrower".into()), true, 6, 2, true, vec![],
            Some("SHORT_PASS".into()), Some("thrown".into()), Some("ACCURATE".into()), false,
        );
        ThrowTeamMateRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(!texts.iter().any(|t| t.contains("tries to throw")));
        // re-rolled → no "needed roll" line either.
        assert!(!texts.iter().any(|t| t.contains("Succeeded on a roll of")));
    }

    #[test]
    fn success_needed_roll_line_included_when_not_rerolled_and_can_throw() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportThrowTeamMateRoll::new(
            Some("thrower".into()), true, 6, 2, false, vec![],
            Some("SHORT_PASS".into()), Some("thrown".into()), Some("ACCURATE".into()), false,
        );
        ThrowTeamMateRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("Succeeded on a roll of 2")));
        assert!(texts.iter().any(|t| t.contains("to avoid a fumble or terrible throw")));
    }
}
