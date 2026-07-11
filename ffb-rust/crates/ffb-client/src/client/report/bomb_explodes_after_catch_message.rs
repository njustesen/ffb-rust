use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_bomb_explodes_after_catch::ReportBombExplodesAfterCatch;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `BombExplodesAfterCatchMessage.java`.
pub struct BombExplodesAfterCatchMessage;

impl ReportMessage for BombExplodesAfterCatchMessage {
    type Report = ReportBombExplodesAfterCatch;

    fn report_id(&self) -> ReportId {
        ReportId::BOMB_EXPLODES_AFTER_CATCH
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::ROLL, &format!("Bomb Roll [ {} ]", report.get_roll()));
        let catcher = game.player(report.get_catcher_id());
        let team_style = if catcher.is_some_and(|c| game.team_home.has_player(&c.id)) {
            TextStyle::HOME
        } else {
            TextStyle::AWAY
        };
        status_report.print_indent_style(status_report.get_indent() + 2, team_style, catcher.map(|c| c.name.as_str()).unwrap_or(""));
        status_report.print_indent(status_report.get_indent() + 2, " caught the bomb");
        if report.explodes() {
            let genitive = catcher.map(|c| c.gender.genitive()).unwrap_or("");
            status_report.println_indent(status_report.get_indent() + 2, &format!(" but it explodes in {genitive} hands."));
        } else {
            status_report.println_indent(status_report.get_indent() + 2, " and it does not explode");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, Rules};
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

    #[test]
    fn report_id_matches() {
        assert_eq!(BombExplodesAfterCatchMessage.report_id(), ReportId::BOMB_EXPLODES_AFTER_CATCH);
    }

    #[test]
    fn explodes_uses_genitive_and_home_style() {
        let mut game = make_game();
        let mut catcher = Player::default();
        catcher.id = "p1".into();
        catcher.name = "Catcher".into();
        catcher.gender = PlayerGender::Male;
        game.team_home.players.push(catcher);

        let report = ReportBombExplodesAfterCatch::new("p1".into(), true, 5);
        let mut status_report = StatusReport::new();
        BombExplodesAfterCatchMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Bomb Roll [ 5 ]"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Catcher"));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::HOME));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" caught the bomb"));
        assert_eq!(status_report.rendered_runs[4].text.as_deref(), Some(" but it explodes in his hands."));
    }

    #[test]
    fn does_not_explode_away_style() {
        let mut game = make_game();
        let mut catcher = Player::default();
        catcher.id = "p2".into();
        catcher.name = "AwayCatcher".into();
        game.team_away.players.push(catcher);

        let report = ReportBombExplodesAfterCatch::new("p2".into(), false, 2);
        let mut status_report = StatusReport::new();
        BombExplodesAfterCatchMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::AWAY));
        assert_eq!(status_report.rendered_runs[4].text.as_deref(), Some(" and it does not explode"));
    }

    #[test]
    fn honors_current_indent() {
        let mut game = make_game();
        let mut catcher = Player::default();
        catcher.id = "p1".into();
        catcher.name = "Catcher".into();
        game.team_home.players.push(catcher);
        let report = ReportBombExplodesAfterCatch::new("p1".into(), false, 1);
        let mut status_report = StatusReport::new();
        status_report.set_indent(1);
        BombExplodesAfterCatchMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].paragraph_style, Some(crate::client::paragraph_style::ParagraphStyle::INDENT_2));
    }
}
