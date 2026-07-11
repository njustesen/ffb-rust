use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_jump_up_roll::ReportJumpUpRoll;

/// 1:1 translation of `JumpUpRollMessage.java`.
///
/// Java declares `ReportMessageBase<ReportSkillRoll>` generically, but the runtime type
/// dispatched to this renderer is always the concrete `ReportJumpUpRoll` subclass (matched
/// by `@ReportMessageType(ReportId.JUMP_UP_ROLL)`); the Rust `IReport` trait bound requires
/// pinning `Self::Report` to that concrete wrapper type instead of the abstract base.
pub struct JumpUpRollMessage;

impl ReportMessage for JumpUpRollMessage {
    type Report = ReportJumpUpRoll;

    fn report_id(&self) -> ReportId {
        ReportId::JUMP_UP_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let report = &report.base;
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let mut needed_roll: Option<String> = None;
        let status = format!("Jump Up Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        print_player(status_report, game, status_report.get_indent() + 1, false, player);
        let genitive = player.map(|p| p.gender.genitive()).unwrap_or("");
        if report.is_successful() {
            let status = format!(" jumps up to block {} opponent.", genitive);
            status_report.println_indent(status_report.get_indent() + 1, &status);
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            let status = format!(" doesn't get to {} feet.", genitive);
            status_report.println_indent(status_report.get_indent() + 1, &status);
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }
        if let Some(mut needed_roll) = needed_roll {
            if let Some(player) = player {
                // java: mechanic.formatJumpUpResult(report, player) — ffb-mechanics::AgilityMechanic
                // expects full RollModifier objects (name + magnitude + included flag), but
                // ReportSkillRoll only retains sorted modifier name strings. Approximated using
                // the same " - <name>" convention as StatusReport::format_roll_modifiers, wrapped
                // in the same "(Roll ... >= N+)" shape as AgilityMechanic::format_result.
                let modifiers = status_report.format_roll_modifiers(report.get_roll_modifiers());
                needed_roll.push_str(&format!(" (Roll{} >= {}+)", modifiers, player.agility_with_modifiers().max(2)));
            }
            status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, Rules};
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false }
    }

    fn make_player(id: &str) -> Player {
        let mut p = Player::default();
        p.id = id.to_string();
        p.name = format!("Player {id}");
        p.gender = PlayerGender::Male;
        p.agility = 3;
        p
    }

    fn make_game() -> Game {
        let mut home = make_team("home");
        home.players.push(make_player("p1"));
        let mut game = Game::new(home, make_team("away"), Rules::Bb2025);
        game.acting_player.player_id = Some("p1".to_string());
        game
    }

    #[test]
    fn successful_not_rerolled_prints_needed_roll() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportJumpUpRoll::new(Some("p1".into()), true, 5, 3, false, vec![]);
        JumpUpRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.as_deref() == Some("Jump Up Roll [ 5 ]")));
        assert!(texts.iter().any(|t| t.as_deref() == Some(" jumps up to block his opponent.")));
        assert!(texts.iter().any(|t| t.as_deref().unwrap_or("").starts_with("Succeeded on a roll of 3+")));
    }

    #[test]
    fn failed_not_rerolled_prints_needed_roll() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportJumpUpRoll::new(Some("p1".into()), false, 1, 3, false, vec![]);
        JumpUpRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.as_deref() == Some(" doesn't get to his feet.")));
        assert!(texts.iter().any(|t| t.as_deref().unwrap_or("").starts_with("Roll a 3+ to succeed")));
    }

    #[test]
    fn re_rolled_suppresses_needed_roll_line() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportJumpUpRoll::new(Some("p1".into()), true, 5, 3, true, vec![]);
        JumpUpRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().all(|r| r.text_style != Some(TextStyle::NEEDED_ROLL)));
    }

    #[test]
    fn get_key_matches_report_id() {
        assert_eq!(JumpUpRollMessage.get_key(), "jumpUpRoll");
    }
}
