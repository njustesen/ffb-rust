use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_jump_roll::ReportJumpRoll;

/// Java: `AgilityMechanic.formatJumpResult(report, player)`. The Rust `ReportJumpRoll`
/// (wrapping `ReportSkillRoll`) model only carries resolved modifier-name strings (no
/// retained `RollModifier` objects with numeric magnitude), so this mirrors the
/// mechanic's `formatResult` formula using `StatusReport.format_roll_modifiers` for the
/// modifier text instead of calling the mechanic directly.
// java: AgilityMechanic.formatJumpResult(report, player) — approximated locally, see
// comment above; no RollModifier objects are reachable from ReportJumpRoll.
fn format_jump_result(status_report: &StatusReport, roll_modifier_names: &[String], player: Option<&Player>) -> String {
    let agility = player.map(|p| p.agility_with_modifiers()).unwrap_or(0);
    format!(
        " (Roll{} >= {}+)",
        status_report.format_roll_modifiers(roll_modifier_names),
        agility.max(2)
    )
}

/// 1:1 translation of `JumpRollMessage.java`. Java types this message against the
/// abstract `ReportSkillRoll`; the Rust model represents that as the concrete
/// `ReportJumpRoll` wrapper (`base: ReportSkillRoll`) since `ReportSkillRoll` itself
/// carries no `ReportId`/`IReport` impl.
pub struct JumpRollMessage;

impl ReportMessage for JumpRollMessage {
    type Report = ReportJumpRoll;

    fn report_id(&self) -> ReportId {
        ReportId::JUMP_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let mut needed_roll: Option<String> = None;
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));

        let status = format!("Jump Roll [ {} ]", report.base.get_roll());
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        print_player(status_report, game, status_report.get_indent() + 1, false, player);
        if report.base.is_successful() {
            let status = format!(
                " jumps over {} opponents.",
                player.map(|p| p.gender.genitive()).unwrap_or("")
            );
            status_report.println_indent(status_report.get_indent() + 1, &status);
            if !report.base.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.base.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(status_report.get_indent() + 1, " trips while jumping.");
            if !report.base.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.base.get_minimum_roll()));
            }
        }

        if let Some(mut needed_roll) = needed_roll {
            needed_roll.push_str(&format_jump_result(status_report, report.base.get_roll_modifiers(), player));
            status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(),
            name: id.to_string(),
            race: "human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
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
            special_rules: Vec::new(),
            players: Vec::new(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, agility: i32) {
        let mut player = Player::default();
        player.id = id.to_string();
        player.name = id.to_string();
        player.agility = agility;
        if home {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(JumpRollMessage.report_id(), ReportId::JUMP_ROLL);
    }

    #[test]
    fn successful_jump_reports_jumps_over_opponents() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", 3);
        game.acting_player.player_id = Some("p1".into());
        let report = ReportJumpRoll::new(Some("p1".into()), true, 5, 3, false, vec![]);
        JumpRollMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("jumps over")));
        assert!(texts.iter().any(|t| t.contains("Succeeded on a roll of 3+")));
    }

    #[test]
    fn unsuccessful_jump_reports_trips() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", 3);
        game.acting_player.player_id = Some("p1".into());
        let report = ReportJumpRoll::new(Some("p1".into()), false, 1, 4, false, vec![]);
        JumpRollMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("trips while jumping.")));
        assert!(texts.iter().any(|t| t.contains("Roll a 4+ to succeed")));
    }

    #[test]
    fn re_rolled_skips_needed_roll_line() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", 3);
        game.acting_player.player_id = Some("p1".into());
        let report = ReportJumpRoll::new(Some("p1".into()), true, 5, 3, true, vec![]);
        JumpRollMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(!texts.iter().any(|t| t.contains("Succeeded on a roll of")));
    }

    #[test]
    fn needed_roll_includes_agility_formula() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", 4);
        game.acting_player.player_id = Some("p1".into());
        let report = ReportJumpRoll::new(Some("p1".into()), false, 1, 3, false, vec![]);
        JumpRollMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("(Roll >= 4+)")));
    }
}
