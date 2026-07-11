use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_swarming_roll::ReportSwarmingRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `SwarmingPlayersRollMessage.java`.
pub struct SwarmingPlayersRollMessage;

impl SwarmingPlayersRollMessage {
    fn render_legacy(&self, status_report: &mut StatusReport, game: &Game, report: &ReportSwarmingRoll) {
        let is_home = report.get_team_id() == Some(game.team_home.id.as_str());
        let style = if is_home { TextStyle::HOME_BOLD } else { TextStyle::AWAY_BOLD };
        status_report.println_indent_style(0, TextStyle::ROLL, &format!("Swarming Roll [{}]", report.get_amount()));
        if let Some(team) = report.get_team_id().and_then(|id| game.team_by_id(id)) {
            status_report.print_indent_style(1, style, &team.name.clone());
        }
        status_report.println_indent_style(1, TextStyle::NONE, &format!(" are allowed to place {} swarming players.", report.get_amount()));
    }
}

impl ReportMessage for SwarmingPlayersRollMessage {
    type Report = ReportSwarmingRoll;

    fn report_id(&self) -> ReportId {
        ReportId::SWARMING_PLAYERS_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        if report.get_limit() < 0 {
            self.render_legacy(status_report, game, report);
        } else {
            let is_home = report.get_team_id() == Some(game.team_home.id.as_str());
            let style = if is_home { TextStyle::HOME_BOLD } else { TextStyle::AWAY_BOLD };
            status_report.println_indent_style(0, TextStyle::ROLL, &format!("Swarming Roll [{}]", report.get_roll()));
            if let Some(team) = report.get_team_id().and_then(|id| game.team_by_id(id)) {
                status_report.print_indent_style(1, style, &team.name.clone());
            }
            status_report.println_indent_style(1, TextStyle::NONE, &format!(" have {} swarming players on the pitch.", report.get_limit()));
            if report.get_amount() == 0 {
                status_report.println_indent_style(1, TextStyle::NONE, "They are not allowed to place any swarming players.");
            } else {
                status_report.println_indent_style(1, TextStyle::NONE, &format!("They are allowed to place {} swarming players.", report.get_amount()));
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
    fn legacy_path_used_when_limit_negative() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportSwarmingRoll::new(Some("home".into()), 3, 5, -1);
        SwarmingPlayersRollMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.contains(&"Swarming Roll [3]"));
        assert!(t.contains(&" are allowed to place 3 swarming players."));
    }

    #[test]
    fn new_path_zero_amount() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportSwarmingRoll::new(Some("away".into()), 0, 6, 4);
        SwarmingPlayersRollMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.contains(&"Swarming Roll [6]"));
        assert!(t.contains(&" have 4 swarming players on the pitch."));
        assert!(t.contains(&"They are not allowed to place any swarming players."));
        let away_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Away Team")).unwrap();
        assert_eq!(away_run.text_style, Some(TextStyle::AWAY_BOLD));
    }

    #[test]
    fn new_path_positive_amount() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportSwarmingRoll::new(Some("home".into()), 2, 6, 4);
        SwarmingPlayersRollMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.contains(&"They are allowed to place 2 swarming players."));
        let home_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Home Team")).unwrap();
        assert_eq!(home_run.text_style, Some(TextStyle::HOME_BOLD));
    }
}
