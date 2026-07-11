use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_blitz_roll::ReportBlitzRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `BlitzRollMessage.java`.
pub struct BlitzRollMessage;

impl ReportMessage for BlitzRollMessage {
    type Report = ReportBlitzRoll;

    fn report_id(&self) -> ReportId {
        ReportId::BLITZ_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.println_indent_style(
            status_report.get_indent(),
            TextStyle::ROLL,
            &format!("Charge! Roll [ {} ]", report.get_roll()),
        );
        let team_id = report.get_team_id().unwrap_or("");
        let team_style = if game.team_home.id == team_id { TextStyle::HOME } else { TextStyle::AWAY };
        let team_name = game.team_by_id(team_id).map(|t| t.name.clone()).unwrap_or_default();
        status_report.print_indent_style(status_report.get_indent() + 1, team_style, &team_name);
        status_report.println_indent_style(
            status_report.get_indent() + 1,
            TextStyle::NONE,
            &format!(" may select {} open players to perform actions.", report.get_amount()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {id}"),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
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
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn home_team_uses_home_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportBlitzRoll::new(Some("home".into()), 2, 5);
        BlitzRollMessage.render(&mut status_report, &game, &report);
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Team home")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::HOME));
    }

    #[test]
    fn away_team_uses_away_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportBlitzRoll::new(Some("away".into()), 3, 4);
        BlitzRollMessage.render(&mut status_report, &game, &report);
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Team away")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn roll_and_amount_text() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportBlitzRoll::new(Some("home".into()), 4, 6);
        BlitzRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Charge! Roll [ 6 ]"));
        assert!(texts.iter().any(|t| t.contains("select 4 open players")));
    }

    #[test]
    fn report_id_is_blitz_roll() {
        assert_eq!(BlitzRollMessage.report_id(), ReportId::BLITZ_ROLL);
    }
}
