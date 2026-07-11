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
        let team_id = report.get_team_id().unwrap_or("");
        let team_style = if game.team_home.id == team_id { TextStyle::HOME } else { TextStyle::AWAY };
        let team_name = game.team_by_id(team_id).map(|t| t.name.clone()).unwrap_or_default();
        status_report.print_indent_style(indent + 1, team_style, &team_name);
        if report.get_amount() > 1 {
            status_report.println_indent_style(indent + 1, TextStyle::NONE, &format!(" may select up to {} players to setup again", report.get_amount()));
        } else if report.get_amount() == 1 {
            status_report.println_indent_style(indent + 1, TextStyle::NONE, &format!(" may select {} player to setup again", report.get_amount()));
        } else {
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " have no eligible players, moving on to kick-off");
        }
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
    fn report_id_is_solid_defence_roll() {
        assert_eq!(SolidDefenceRollMessage.report_id(), ReportId::SOLID_DEFENCE_ROLL);
    }

    #[test]
    fn multiple_players_may_setup_again() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSolidDefenceRoll::new(Some("home".into()), 5, 2);
        SolidDefenceRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " may select up to 2 players to setup again"));
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Team home")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::HOME));
    }

    #[test]
    fn single_player_may_setup_again() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSolidDefenceRoll::new(Some("away".into()), 3, 1);
        SolidDefenceRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " may select 1 player to setup again"));
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Team away")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn no_eligible_players() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSolidDefenceRoll::new(Some("home".into()), 1, 0);
        SolidDefenceRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " have no eligible players, moving on to kick-off"));
    }
}
