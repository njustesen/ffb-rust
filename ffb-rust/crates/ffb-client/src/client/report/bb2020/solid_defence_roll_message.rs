use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_solid_defence_roll::ReportSolidDefenceRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `SolidDefenceRollMessage.java`.
pub struct SolidDefenceRollMessage;

impl ReportMessage for SolidDefenceRollMessage {
    type Report = ReportSolidDefenceRoll;

    fn report_id(&self) -> ReportId {
        ReportId::SOLID_DEFENCE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Solid Defence Roll [{}]", report.get_roll()));
        let team_style = if Some(game.team_home.id.as_str()) == report.get_team_id() {
            TextStyle::HOME
        } else {
            TextStyle::AWAY
        };
        if let Some(team) = report.get_team_id().and_then(|id| game.team_by_id(id)) {
            status_report.print_indent_style(indent + 1, team_style, &team.name.clone());
        }
        status_report.println_indent_style(indent + 1, TextStyle::NONE, &format!(" may reorganize {} players", report.get_amount()));
        status_report.println_indent_style(indent + 2, TextStyle::EXPLANATION, "Numbers mark original player positions.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, name: &str) -> Team {
        Team {
            id: id.into(),
            name: name.into(),
            race: String::new(),
            roster_id: String::new(),
            coach: String::new(),
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
            players: Vec::<Player>::new(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home", "Home Team"), make_team("away", "Away Team"), Rules::Bb2020)
    }

    fn texts(status_report: &StatusReport) -> Vec<&str> {
        status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect()
    }

    #[test]
    fn home_team_roll() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportSolidDefenceRoll::new(Some("home".into()), 4, 2);
        SolidDefenceRollMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.contains(&"Solid Defence Roll [4]"));
        assert!(t.contains(&"Home Team"));
        assert!(t.contains(&" may reorganize 2 players"));
        assert!(t.contains(&"Numbers mark original player positions."));
        let home_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Home Team")).unwrap();
        assert_eq!(home_run.text_style, Some(TextStyle::HOME));
    }

    #[test]
    fn away_team_roll_uses_away_style() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportSolidDefenceRoll::new(Some("away".into()), 6, 3);
        SolidDefenceRollMessage.render(&mut status_report, &game, &report);
        let away_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Away Team")).unwrap();
        assert_eq!(away_run.text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn explanation_line_present() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportSolidDefenceRoll::new(Some("home".into()), 1, 0);
        SolidDefenceRollMessage.render(&mut status_report, &game, &report);
        let explanation_run = status_report
            .rendered_runs
            .iter()
            .find(|r| r.text.as_deref() == Some("Numbers mark original player positions."))
            .unwrap();
        assert_eq!(explanation_run.text_style, Some(TextStyle::EXPLANATION));
    }
}
