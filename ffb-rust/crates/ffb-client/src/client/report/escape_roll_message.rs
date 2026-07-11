use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_escape_roll::ReportEscapeRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `EscapeRollMessage.java`. Java types this message against the
/// abstract `ReportSkillRoll`; the Rust model represents that as the concrete
/// `ReportEscapeRoll` wrapper (`base: ReportSkillRoll`) since `ReportSkillRoll` itself
/// carries no `ReportId`/`IReport` impl.
pub struct EscapeRollMessage;

impl ReportMessage for EscapeRollMessage {
    type Report = ReportEscapeRoll;

    fn report_id(&self) -> ReportId {
        ReportId::ESCAPE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let thrown_player = report.base.get_player_id().and_then(|id| game.player(id));
        let status = format!("Escape Roll [ {} ]", report.base.get_roll());
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        print_player(status_report, game, status_report.get_indent() + 1, false, thrown_player);
        let status = if report.base.is_successful() {
            " manages to wriggle free.".to_string()
        } else {
            format!(
                " disappears in {} team-mate's stomach.",
                thrown_player.map(|p| p.gender.genitive()).unwrap_or("")
            )
        };
        status_report.println_indent(status_report.get_indent() + 1, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
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

    fn add_player(game: &mut Game, home: bool, id: &str) {
        let mut player = Player::default();
        player.id = id.to_string();
        player.name = id.to_string();
        if home {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(EscapeRollMessage.report_id(), ReportId::ESCAPE_ROLL);
    }

    #[test]
    fn successful_escape_reports_wriggle_free() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "p1");
        let report = ReportEscapeRoll::new(Some("p1".into()), true, 4, 2, false, vec![]);
        EscapeRollMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Escape Roll [ 4 ]"));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::ROLL));
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("manages to wriggle free")));
    }

    #[test]
    fn unsuccessful_escape_reports_eaten_with_gender() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, false, "p2");
        let report = ReportEscapeRoll::new(Some("p2".into()), false, 1, 3, false, vec![]);
        EscapeRollMessage.render(&mut status_report, &game, &report);

        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("team-mate's stomach")));
    }

    #[test]
    fn prints_player_at_indent_plus_one() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        add_player(&mut game, true, "p3");
        let report = ReportEscapeRoll::new(Some("p3".into()), true, 5, 2, false, vec![]);
        EscapeRollMessage.render(&mut status_report, &game, &report);

        // run 0 is the roll status line, run 1 is its println terminator, run 2 is the player name print
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("p3"));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::HOME));
    }
}
