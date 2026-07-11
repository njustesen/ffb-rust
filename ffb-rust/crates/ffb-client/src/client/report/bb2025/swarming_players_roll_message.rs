use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2025::report_swarming_roll::ReportSwarmingRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `SwarmingPlayersRollMessage.java`.
pub struct SwarmingPlayersRollMessage;

impl ReportMessage for SwarmingPlayersRollMessage {
    type Report = ReportSwarmingRoll;

    fn report_id(&self) -> ReportId {
        ReportId::SWARMING_PLAYERS_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let team = game.team_by_id(report.get_team_id());
        let is_home = team.is_some_and(|t| game.team_home.id == t.id);
        let style = if is_home { TextStyle::HOME_BOLD } else { TextStyle::AWAY_BOLD };
        status_report.println_indent_style(0, TextStyle::ROLL, &format!("Swarming Roll [{}]", report.get_roll()));
        let team_name = team.map(|t| t.name.clone()).unwrap_or_default();
        status_report.print_indent_style(1, style, &team_name);
        status_report.println_indent_style(1, TextStyle::NONE, &format!(" are allowed to place {} swarming players.", report.get_roll()));
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
    fn report_id_is_swarming_players_roll() {
        assert_eq!(SwarmingPlayersRollMessage.report_id(), ReportId::SWARMING_PLAYERS_ROLL);
    }

    #[test]
    fn home_team_uses_home_bold_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSwarmingRoll::new("home".into(), 3);
        SwarmingPlayersRollMessage.render(&mut status_report, &game, &report);
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Team home")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::HOME_BOLD));
    }

    #[test]
    fn away_team_uses_away_bold_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSwarmingRoll::new("away".into(), 5);
        SwarmingPlayersRollMessage.render(&mut status_report, &game, &report);
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Team away")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::AWAY_BOLD));
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " are allowed to place 5 swarming players."));
    }

    #[test]
    fn roll_reported_in_header() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSwarmingRoll::new("home".into(), 2);
        SwarmingPlayersRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Swarming Roll [2]"));
    }
}
