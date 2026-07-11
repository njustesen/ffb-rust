use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_safe_throw_roll::ReportSafeThrowRoll;

/// 1:1 translation of `SafeThrowRollMessage.java`.
pub struct SafeThrowRollMessage;

impl ReportMessage for SafeThrowRollMessage {
    type Report = ReportSafeThrowRoll;

    fn report_id(&self) -> ReportId {
        ReportId::SAFE_THROW_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let mut needed_roll: Option<String> = None;
        let player = game.player(game.acting_player.player_id.as_deref().unwrap_or(""));
        let status = format!("Safe Throw Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::ROLL, &status);
        print_player(status_report, game, status_report.get_indent() + 2, false, player);
        if report.is_successful() {
            status_report.println_indent(status_report.get_indent() + 2, " throws safely over any interceptors.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(status_report.get_indent() + 2, "'s Safe Throw fails to stop the interception.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }
        if let (Some(mut needed_roll), Some(player)) = (needed_roll, player) {
            needed_roll.push_str(&format_safe_throw_result(player));
            status_report.println_indent_style(status_report.get_indent() + 2, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

/// java: `AgilityMechanic.formatSafeThrowResult(Player)`. Java calls
/// `formatResult(player.getAgilityWithModifiers(), new RollModifier[0])` — no roll modifiers
/// are involved, so this is a direct 1:1 translation with no data gap.
fn format_safe_throw_result(player: &Player) -> String {
    format!(" (Roll >= {}+)", player.agility_with_modifiers().max(2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    fn add_acting_player(game: &mut Game, agility: i32) {
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Thrower".into();
        player.agility = agility;
        game.team_home.players.push(player);
        game.acting_player.player_id = Some("p1".into());
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(SafeThrowRollMessage.report_id(), ReportId::SAFE_THROW_ROLL);
    }

    #[test]
    fn render_successful_throw_with_needed_roll() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_acting_player(&mut game, 3);
        let report = ReportSafeThrowRoll::new(None, true, 4, 2, false, vec![]);
        SafeThrowRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Safe Throw Roll [ 4 ]"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" throws safely over any interceptors."));
        let last = status_report.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last.text.as_deref(), Some("Succeeded on a roll of 2+ (Roll >= 3+)"));
    }

    #[test]
    fn render_failed_throw_with_needed_roll() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_acting_player(&mut game, 3);
        let report = ReportSafeThrowRoll::new(None, false, 1, 2, false, vec![]);
        SafeThrowRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some("'s Safe Throw fails to stop the interception."));
        let last = status_report.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last.text.as_deref(), Some("Roll a 2+ to succeed (Roll >= 3+)"));
    }

    #[test]
    fn render_re_rolled_omits_needed_roll() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_acting_player(&mut game, 3);
        let report = ReportSafeThrowRoll::new(None, true, 4, 2, true, vec![]);
        SafeThrowRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().all(|r| r.text_style != Some(TextStyle::NEEDED_ROLL)));
    }
}
