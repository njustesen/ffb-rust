use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_regeneration_roll::ReportRegenerationRoll;

/// 1:1 translation of `RegenerationRollMessage.java`.
pub struct RegenerationRollMessage;

impl ReportMessage for RegenerationRollMessage {
    type Report = ReportRegenerationRoll;

    fn report_id(&self) -> ReportId {
        ReportId::REGENERATION_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        if report.get_roll() > 0 {
            let text = format!("Regeneration Roll [ {} ]", report.get_roll());
            status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &text);
            let player = report.get_player_id().and_then(|id| game.player(id));
            print_player(status_report, game, status_report.get_indent() + 1, false, player);
            if report.is_successful() {
                status_report.println_indent(status_report.get_indent() + 1, " regenerates.");
            } else {
                status_report.println_indent(status_report.get_indent() + 1, " does not regenerate.");
            }
        }
    }
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

    fn make_report(roll: i32, successful: bool) -> ReportRegenerationRoll {
        ReportRegenerationRoll::new(Some("p1".into()), successful, roll, 4, false, vec![])
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(RegenerationRollMessage.report_id(), ReportId::REGENERATION_ROLL);
    }

    #[test]
    fn render_successful_regeneration() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Zombie".into();
        game.team_home.players.push(player);
        let report = make_report(5, true);
        RegenerationRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Regeneration Roll [ 5 ]"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::ROLL));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Zombie"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" regenerates."));
    }

    #[test]
    fn render_failed_regeneration() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        let mut player = Player::default();
        player.id = "p1".into();
        game.team_home.players.push(player);
        let report = make_report(2, false);
        RegenerationRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" does not regenerate."));
    }

    #[test]
    fn render_skips_when_roll_is_zero() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = make_report(0, false);
        RegenerationRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }
}
