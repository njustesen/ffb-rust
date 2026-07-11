use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_old_pro::ReportOldPro;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `OldProMessage.java`.
pub struct OldProMessage;

impl ReportMessage for OldProMessage {
    type Report = ReportOldPro;

    fn report_id(&self) -> ReportId {
        ReportId::OLD_PRO
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.println_indent_style(
            status_report.get_indent() + 1,
            TextStyle::ROLL,
            &format!("Old Pro Roll [ {} ]", report.get_new_value()),
        );
        let player = report.get_player_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent() + 1;
        print_player(status_report, game, indent, false, player);
        let action = if report.is_self_inflicted() { " forced the opponent to re-roll a " } else { " re-rolled a " };
        status_report.println_indent_style(
            indent,
            TextStyle::NONE,
            &format!("{}{} into a {}.", action, report.get_old_value(), report.get_new_value()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {id}"),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: format!("Coach {id}"),
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
        let player = Player { id: "p1".into(), name: "Grobnik".into(), ..Player::default() };
        Game::new(make_team("home", vec![player]), make_team("away", vec![]), Rules::Bb2020)
    }

    #[test]
    fn self_inflicted_true() {
        let game = make_game();
        let report = ReportOldPro::new(Some("p1".into()), 2, 5, true);
        let mut status_report = StatusReport::new();
        OldProMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Old Pro Roll [ 5 ]"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Grobnik"));
        assert_eq!(
            status_report.rendered_runs[3].text.as_deref(),
            Some(" forced the opponent to re-roll a 2 into a 5.")
        );
    }

    #[test]
    fn self_inflicted_false() {
        let game = make_game();
        let report = ReportOldPro::new(Some("p1".into()), 3, 6, false);
        let mut status_report = StatusReport::new();
        OldProMessage.render(&mut status_report, &game, &report);
        assert_eq!(
            status_report.rendered_runs[3].text.as_deref(),
            Some(" re-rolled a 3 into a 6.")
        );
    }

    #[test]
    fn unknown_player_skips_name_run() {
        let game = make_game();
        let report = ReportOldPro::new(Some("unknown".into()), 1, 2, false);
        let mut status_report = StatusReport::new();
        OldProMessage.render(&mut status_report, &game, &report);
        // Only the roll header and the final sentence are emitted; no player-name run.
        assert_eq!(status_report.rendered_runs.len(), 4);
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(OldProMessage.report_id(), ReportId::OLD_PRO);
        assert_eq!(OldProMessage.get_key(), "oldPro");
    }
}
