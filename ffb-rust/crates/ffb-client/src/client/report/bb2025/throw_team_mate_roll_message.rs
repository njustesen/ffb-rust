use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PassingDistance;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_throw_team_mate_roll::ReportThrowTeamMateRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `ThrowTeamMateRollMessage.java`.
pub struct ThrowTeamMateRollMessage;

impl ReportMessage for ThrowTeamMateRollMessage {
    type Report = ReportThrowTeamMateRoll;

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let mut needed_roll: Option<String> = None;
        let thrower = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let thrown_player = report.get_thrown_player_id().and_then(|id| game.player(id));
        let can_throw = thrower.is_some_and(|p| p.passing > 0);

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

        let roll_label = if report.is_kick() { "Kick Team-Mate Roll" } else { "Throw Team-Mate Roll" };
        status_report.println_indent_style(indent + 1, TextStyle::ROLL, &format!("{roll_label} [ {} ]", report.get_roll()));
        print_player(status_report, game, indent + 2, false, thrower);
        if report.is_successful() {
            if report.is_kick() {
                status_report.print_indent(indent + 2, " kicks ");
            } else {
                status_report.print_indent(indent + 2, " throws ");
            }
            print_player(status_report, game, indent + 2, false, thrown_player);
            let outcome = if report.get_pass_result() == Some("ACCURATE") { "superbly" } else { "with a subpar result" };
            status_report.println_indent(indent + 2, &format!(" {outcome}."));
        } else {
            status_report.print_indent(indent + 2, " fumbles ");
            print_player(status_report, game, indent + 2, false, thrown_player);
            status_report.println_indent(indent + 2, ".");
        }

        if report.is_successful() && !report.is_re_rolled() && can_throw {
            let suffix = if report.is_kick() { "+ to avoid a Fumbled Kick" } else { "+ to avoid a Fumbled Throw" };
            needed_roll = Some(format!("Succeeded on a roll of {}{suffix}", report.get_minimum_roll()));
        }
        if !report.is_successful() && !report.is_re_rolled() && can_throw {
            let suffix = if report.is_kick() { "+ to make at least a Subpar Kick" } else { "+ to make at least a Subpar Throw" };
            needed_roll = Some(format!("Roll a {}{suffix}", report.get_minimum_roll()));
        }
        if let Some(needed_roll) = needed_roll {
            let passing_distance = report.get_passing_distance().and_then(PassingDistance::from_name);
            let (modifier2020, name) = passing_distance.map(|d| (d.modifier_2020(), d.name())).unwrap_or((0, ""));
            let modifiers_str = status_report.format_roll_modifiers(report.get_roll_modifiers());
            let full = format!("{needed_roll} (Roll  - {modifier2020} {name}{modifiers_str} > 1).");
            status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &full);
        }
        status_report.set_indent(status_report.get_indent() + 1);
    }

    fn report_id(&self) -> ReportId {
        ReportId::THROW_TEAM_MATE_ROLL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, Rules};
    use ffb_model::enums::PlayerAction;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str, passing: i32) -> Player {
        Player { id: id.into(), name: name.into(), player_type: PlayerType::default(), passing, ..Default::default() }
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
        let home = make_team("home", vec![make_player("thrower", "Thrower", 3)]);
        let away = make_team("away", vec![make_player("thrown", "Thrown", 0)]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.acting_player.set_player("thrower".into(), PlayerAction::Move);
        game
    }

    #[test]
    fn report_id_is_throw_team_mate_roll() {
        assert_eq!(ThrowTeamMateRollMessage.report_id(), ReportId::THROW_TEAM_MATE_ROLL);
    }

    #[test]
    fn successful_accurate_throw_reports_superbly() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowTeamMateRoll::new(
            Some("thrower".into()), true, 4, 3, false, vec![],
            Some("SHORT_PASS".into()), Some("thrown".into()), Some("ACCURATE".into()), false,
        );
        ThrowTeamMateRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Throw Team-Mate Roll [ 4 ]"));
        assert!(texts.iter().any(|t| t == " superbly."));
        assert!(texts.iter().any(|t| t.contains("Succeeded on a roll of 3+ to avoid a Fumbled Throw")));
    }

    #[test]
    fn kick_uses_kick_wording() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowTeamMateRoll::new(
            Some("thrower".into()), true, 4, 3, false, vec![],
            Some("SHORT_PASS".into()), Some("thrown".into()), Some("INACCURATE".into()), true,
        );
        ThrowTeamMateRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Kick Team-Mate Roll [ 4 ]"));
        assert!(texts.iter().any(|t| t == " with a subpar result."));
        assert!(texts.iter().any(|t| t.contains("to avoid a Fumbled Kick")));
    }

    #[test]
    fn fumble_reports_fumbles() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowTeamMateRoll::new(
            Some("thrower".into()), false, 1, 3, false, vec![],
            Some("SHORT_PASS".into()), Some("thrown".into()), None, false,
        );
        ThrowTeamMateRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains(" fumbles ")));
        assert!(texts.iter().any(|t| t.contains("Roll a 3+ to make at least a Subpar Throw")));
    }

    #[test]
    fn re_rolled_skips_intro_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowTeamMateRoll::new(
            Some("thrower".into()), true, 4, 3, true, vec![],
            Some("SHORT_PASS".into()), Some("thrown".into()), Some("ACCURATE".into()), false,
        );
        ThrowTeamMateRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(!texts.iter().any(|t| t.contains("tries to throw")));
        assert!(!texts.iter().any(|t| t.contains("Succeeded on a roll of")));
    }

    #[test]
    fn indent_incremented_after_render() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportThrowTeamMateRoll::new(
            Some("thrower".into()), true, 4, 3, false, vec![],
            Some("SHORT_PASS".into()), Some("thrown".into()), Some("ACCURATE".into()), false,
        );
        let before = status_report.get_indent();
        ThrowTeamMateRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.get_indent(), before + 1);
    }
}
