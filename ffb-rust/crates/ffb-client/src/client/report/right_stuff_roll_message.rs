use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_right_stuff_roll::ReportRightStuffRoll;

/// 1:1 translation of `RightStuffRollMessage.java`.
pub struct RightStuffRollMessage;

impl ReportMessage for RightStuffRollMessage {
    type Report = ReportRightStuffRoll;

    fn report_id(&self) -> ReportId {
        ReportId::RIGHT_STUFF_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let mut needed_roll: Option<String> = None;
        let status = format!("Landing Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        let thrown_player = report.get_player_id().and_then(|id| game.player(id));
        print_player(status_report, game, status_report.get_indent() + 1, false, thrown_player);
        if report.is_successful() {
            let status = format!(
                " lands on {} feet.",
                thrown_player.map(|p| p.gender.genitive()).unwrap_or("")
            );
            status_report.println_indent(status_report.get_indent() + 1, &status);
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(status_report.get_indent() + 1, " crashes to the ground.");
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }
        if let (Some(mut needed_roll), Some(thrown_player)) = (needed_roll, thrown_player) {
            needed_roll.push_str(&format_right_stuff_result(status_report, report, thrown_player));
            status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

/// java: `AgilityMechanic.formatRightStuffResult(ReportSkillRoll, Player)`. The full
/// `RollModifier` objects (with sign/magnitude) that Java's formatter needs are not retained
/// on `ReportSkillRoll` (only resolved names are — see `report_skill_roll.rs`), so this mirrors
/// `AgilityMechanic.formatResult` using `StatusReport::format_roll_modifiers`'s name-only
/// fallback instead of calling into `ffb-mechanics::AgilityMechanic`.
fn format_right_stuff_result(status_report: &StatusReport, report: &ReportRightStuffRoll, player: &Player) -> String {
    format!(
        " (Roll{} >= {}+)",
        status_report.format_roll_modifiers(report.get_roll_modifiers()),
        player.agility_with_modifiers().max(2)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::player_gender::PlayerGender;
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

    fn add_player(game: &mut Game, agility: i32) -> String {
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Gutter Runner".into();
        player.gender = PlayerGender::Male;
        player.agility = agility;
        player.position_agility = agility;
        game.team_home.players.push(player);
        "p1".into()
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(RightStuffRollMessage.report_id(), ReportId::RIGHT_STUFF_ROLL);
    }

    #[test]
    fn render_successful_lands_on_feet_with_needed_roll() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, 3);
        let report = ReportRightStuffRoll::new(Some("p1".into()), true, 4, 2, false, vec![]);
        RightStuffRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Landing Roll [ 4 ]"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" lands on his feet."));
        let last = status_report.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last.text.as_deref(), Some("Succeeded on a roll of 2+ (Roll >= 3+)"));
        assert_eq!(last.text_style, Some(TextStyle::NEEDED_ROLL));
    }

    #[test]
    fn render_failed_crashes_with_needed_roll() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, 3);
        let report = ReportRightStuffRoll::new(Some("p1".into()), false, 1, 2, false, vec![]);
        RightStuffRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" crashes to the ground."));
        let last = status_report.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last.text.as_deref(), Some("Roll a 2+ to succeed (Roll >= 3+)"));
    }

    #[test]
    fn render_re_rolled_omits_needed_roll() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, 3);
        let report = ReportRightStuffRoll::new(Some("p1".into()), true, 4, 2, true, vec![]);
        RightStuffRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().all(|r| r.text_style != Some(TextStyle::NEEDED_ROLL)));
    }
}
