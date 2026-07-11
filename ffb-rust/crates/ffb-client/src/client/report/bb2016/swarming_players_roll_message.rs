use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_swarming_roll::ReportSwarmingRoll;
use ffb_model::report::report_id::ReportId;

pub struct SwarmingPlayersRollMessage;

impl ReportMessage for SwarmingPlayersRollMessage {
    type Report = ReportSwarmingRoll;

    fn report_id(&self) -> ReportId {
        ReportId::SWARMING_PLAYERS_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let team = report.get_team_id().and_then(|id| game.team_by_id(id));
        let is_home = team.is_some_and(|t| t.id == game.team_home.id);
        let style = if is_home { TextStyle::HOME_BOLD } else { TextStyle::AWAY_BOLD };
        status_report.println_indent_style(0, TextStyle::ROLL, &format!("Swarming Roll [{}]", report.get_amount()));
        if let Some(team) = team {
            status_report.print_indent_style(1, style, &team.name.clone());
        }
        status_report.println_indent_style(1, TextStyle::NONE, &format!(" are allowed to place {} swarming players.", report.get_amount()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_swarming_players_roll() {
        assert_eq!(SwarmingPlayersRollMessage.get_key(), "swarmingPlayersRoll");
    }

    #[test]
    fn home_team_reports_home_bold_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSwarmingRoll::new(Some("home".into()), 2, 4, 3);
        SwarmingPlayersRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Swarming Roll [2]"));
        assert!(status_report.rendered_runs.iter().any(|r| r.text_style == Some(TextStyle::HOME_BOLD)));
    }

    #[test]
    fn away_team_reports_away_bold_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSwarmingRoll::new(Some("away".into()), 1, 2, 1);
        SwarmingPlayersRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text_style == Some(TextStyle::AWAY_BOLD)));
    }
}
