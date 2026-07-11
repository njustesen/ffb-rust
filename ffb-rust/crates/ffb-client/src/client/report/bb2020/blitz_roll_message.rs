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
        let indent = status_report.get_indent();
        status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Blitz Roll [ {} ]", report.get_roll()));
        let team_id = report.get_team_id().unwrap_or("");
        let team_style = if game.team_home.id == team_id { TextStyle::HOME } else { TextStyle::AWAY };
        let team_name = game.team_by_id(team_id).map(|t| t.name.as_str()).unwrap_or("");
        status_report.print_indent_style(indent + 1, team_style, team_name);
        status_report.println_indent_style(
            indent + 1,
            TextStyle::NONE,
            &format!(" may activate {} open players", report.get_amount()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, name: &str, players: Vec<Player>) -> Team {
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
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(
            make_team("home", "Home Team", vec![]),
            make_team("away", "Away Team", vec![]),
            Rules::Bb2020,
        )
    }

    #[test]
    fn home_team_blitz_roll_prints_roll_and_amount() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportBlitzRoll::new(Some("home".into()), 2, 4);
        BlitzRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("Blitz Roll [ 4 ]")));
        assert!(texts.contains(&"Home Team"));
        assert!(texts.iter().any(|t| t.contains("may activate 2 open players")));
    }

    #[test]
    fn away_team_uses_away_text_style() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportBlitzRoll::new(Some("away".into()), 3, 6);
        BlitzRollMessage.render(&mut status_report, &game, &report);
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Away Team")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn home_team_uses_home_text_style() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportBlitzRoll::new(Some("home".into()), 1, 5);
        BlitzRollMessage.render(&mut status_report, &game, &report);
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Home Team")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::HOME));
    }
}
