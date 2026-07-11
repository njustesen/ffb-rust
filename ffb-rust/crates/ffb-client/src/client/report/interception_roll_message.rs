use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_mechanics::agility_mechanic::AgilityMechanic as AgilityMechanicTrait;
use ffb_mechanics::bb2016::agility_mechanic::AgilityMechanic as AgilityMechanicBb2016;
use ffb_mechanics::bb2020::agility_mechanic::AgilityMechanic as AgilityMechanicBb2020;
use ffb_mechanics::bb2025::agility_mechanic::AgilityMechanic as AgilityMechanicBb2025;
use ffb_mechanics::wording::Wording;
use ffb_model::enums::Rules;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_interception_roll::ReportInterceptionRoll;

/// Java: `((AgilityMechanic) game.getFactory(Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name()))`
/// — selects the rules-edition-specific `AgilityMechanic` implementation.
fn interception_wording(rules: Rules, ignore_agility: bool) -> Wording {
    match rules {
        Rules::Bb2016 => AgilityMechanicBb2016.interception_wording(ignore_agility),
        Rules::Bb2020 => AgilityMechanicBb2020.interception_wording(ignore_agility),
        Rules::Bb2025 | Rules::Common => AgilityMechanicBb2025.interception_wording(ignore_agility),
    }
}

/// Java: `AgilityMechanic.formatInterceptionResult(report, player)`. The Rust
/// `ReportInterceptionRoll`/`ReportSkillRoll` model only carries resolved modifier-name
/// strings (no retained `RollModifier` objects with numeric magnitude), so this mirrors
/// the mechanic's `formatResult` formula using `StatusReport.format_roll_modifiers` for the
/// modifier text instead of calling the mechanic directly.
// java: AgilityMechanic.formatInterceptionResult(report, player) — approximated locally,
// see comment above; no RollModifier objects are reachable from ReportInterceptionRoll.
fn format_interception_result(status_report: &StatusReport, roll_modifier_names: &[String], player: Option<&Player>) -> String {
    let agility = player.map(|p| p.agility_with_modifiers()).unwrap_or(0);
    format!(
        " (Roll{} >= {}+)",
        status_report.format_roll_modifiers(roll_modifier_names),
        agility.max(2)
    )
}

/// 1:1 translation of `InterceptionRollMessage.java`.
pub struct InterceptionRollMessage;

impl ReportMessage for InterceptionRollMessage {
    type Report = ReportInterceptionRoll;

    fn report_id(&self) -> ReportId {
        ReportId::INTERCEPTION_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let mut needed_roll: Option<String> = None;
        let player = report.base.get_player_id().and_then(|id| game.player(id));
        let wording = interception_wording(game.rules, report.is_ignore_agility());

        if !report.base.is_re_rolled() {
            print_player(status_report, game, status_report.get_indent(), true, player);
            let target = if report.is_bomb() { "bomb" } else { "ball" };
            status_report.println_indent_style(
                status_report.get_indent(),
                TextStyle::BOLD,
                &format!(" tries to {} the {}:", wording.get_verb(), target),
            );
        }

        let status = format!("{} Roll [ {} ]", wording.get_noun(), report.base.get_roll());
        status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::ROLL, &status);
        print_player(status_report, game, status_report.get_indent() + 2, false, player);
        if report.base.is_successful() {
            let target = if report.is_bomb() { "bomb" } else { "ball" };
            status_report.println_indent(
                status_report.get_indent() + 2,
                &format!(" {} the {}.", wording.get_inflection(), target),
            );
            if !report.base.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.base.get_minimum_roll()));
            }
        } else {
            let target = if report.is_bomb() { "bomb" } else { "ball" };
            status_report.println_indent(
                status_report.get_indent() + 2,
                &format!(" fails to {} the {}.", wording.get_verb(), target),
            );
            if !report.base.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.base.get_minimum_roll()));
            }
        }

        if let Some(mut needed_roll) = needed_roll {
            if !report.is_ignore_agility() {
                needed_roll.push_str(&format_interception_result(status_report, report.base.get_roll_modifiers(), player));
            }
            status_report.println_indent_style(status_report.get_indent() + 2, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(InterceptionRollMessage.report_id(), ReportId::INTERCEPTION_ROLL);
    }

    #[test]
    fn successful_ball_interception_reports_verb_and_inflection() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", 3);
        let report = ReportInterceptionRoll::new(Some("p1".into()), true, 5, 3, false, vec![], false, false);
        InterceptionRollMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("tries to intercept the ball:")));
        assert!(texts.iter().any(|t| t.contains("intercepts the ball.")));
        assert!(texts.iter().any(|t| t.contains("Succeeded on a roll of 3+")));
    }

    #[test]
    fn failed_bomb_interception_reports_fails_to_intercept() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", 3);
        let report = ReportInterceptionRoll::new(Some("p1".into()), false, 1, 4, false, vec![], true, false);
        InterceptionRollMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("tries to intercept the bomb:")));
        assert!(texts.iter().any(|t| t.contains("fails to intercept the bomb.")));
        assert!(texts.iter().any(|t| t.contains("Roll a 4+ to succeed")));
    }

    #[test]
    fn re_rolled_skips_initial_attempt_line_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", 3);
        let report = ReportInterceptionRoll::new(Some("p1".into()), true, 5, 3, true, vec![], false, false);
        InterceptionRollMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(!texts.iter().any(|t| t.contains("tries to intercept")));
        assert!(!texts.iter().any(|t| t.contains("Succeeded on a roll of")));
    }

    #[test]
    fn ignore_agility_skips_needed_roll_formula_suffix() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1", 3);
        let report = ReportInterceptionRoll::new(Some("p1".into()), false, 1, 4, false, vec![], false, true);
        InterceptionRollMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        let needed = texts.iter().find(|t| t.contains("Roll a 4+ to succeed")).unwrap();
        assert!(!needed.contains("(Roll"));
    }
}
