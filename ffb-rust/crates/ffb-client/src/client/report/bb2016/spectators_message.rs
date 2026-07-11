use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_spectators::ReportSpectators;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool;

pub struct SpectatorsMessage;

impl ReportMessage for SpectatorsMessage {
    type Report = ReportSpectators;

    fn report_id(&self) -> ReportId {
        ReportId::SPECTATORS
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.set_indent(0);
        let indent = status_report.get_indent();

        let fan_roll_home = report.get_spectator_roll_home();
        let status = format!("Spectator Roll Home Team [ {} ][ {} ]", fan_roll_home[0], fan_roll_home[1]);
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let rolled_total_home = fan_roll_home[0] + fan_roll_home[1];
        let fan_factor_home = game.team_home.fan_factor;
        let status = format!(
            "Rolled Total of {} + {} Fan Factor = {}",
            rolled_total_home, fan_factor_home, rolled_total_home + fan_factor_home
        );
        status_report.println_indent(indent + 1, &status);
        let status = format!("{} fans have come to support ", string_tool::format_thousands(report.get_spectators_home() as i64));
        status_report.print_indent(indent + 1, &status);
        status_report.print_indent_style(indent + 1, TextStyle::HOME, &game.team_home.name.clone());
        status_report.println_indent(indent + 1, ".");

        let fan_roll_away = report.get_spectator_roll_away();
        let status = format!("Spectator Roll Away Team [ {} ][ {} ]", fan_roll_away[0], fan_roll_away[1]);
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let rolled_total_away = fan_roll_away[0] + fan_roll_away[1];
        let fan_factor_away = game.team_away.fan_factor;
        let status = format!(
            "Rolled Total of {} + {} Fan Factor = {}",
            rolled_total_away, fan_factor_away, rolled_total_away + fan_factor_away
        );
        status_report.println_indent(indent + 1, &status);
        let status = format!("{} fans have come to support ", string_tool::format_thousands(report.get_spectators_away() as i64));
        status_report.print_indent(indent + 1, &status);
        status_report.print_indent_style(indent + 1, TextStyle::AWAY, &game.team_away.name.clone());
        status_report.println_indent(indent + 1, ".");

        if report.get_fame_home() > report.get_fame_away() {
            let suffix = if report.get_fame_home() - report.get_fame_away() > 1 {
                " have the whole audience with them (FAME +2)!"
            } else {
                " have a fan advantage (FAME +1) for the game."
            };
            let status = format!("Team {}{}", game.team_home.name, suffix);
            status_report.println_indent_style(indent, TextStyle::HOME_BOLD, &status);
        } else if report.get_fame_away() > report.get_fame_home() {
            let suffix = if report.get_fame_away() - report.get_fame_home() > 1 {
                " have the whole audience with them (FAME +2)!"
            } else {
                " have a fan advantage (FAME +1) for the game."
            };
            let status = format!("Team {}{}", game.team_away.name, suffix);
            status_report.println_indent_style(indent, TextStyle::AWAY_BOLD, &status);
        } else {
            status_report.println_indent_style(indent, TextStyle::BOLD, "Both teams have equal fan support (FAME 0).");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, fan_factor: i32) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home", 3), make_team("away", 2), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_spectators() {
        assert_eq!(SpectatorsMessage.get_key(), "spectators");
    }

    #[test]
    fn home_fame_advantage_reports_home_bold() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSpectators::new(vec![4, 4], 20000, 2, vec![2, 3], 10000, 0);
        SpectatorsMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Team Team home have the whole audience with them (FAME +2)!")));
    }

    #[test]
    fn equal_fame_reports_equal_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSpectators::new(vec![4, 4], 20000, 1, vec![2, 3], 10000, 1);
        SpectatorsMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Both teams have equal fan support (FAME 0).")));
    }

    #[test]
    fn reports_spectator_counts() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSpectators::new(vec![4, 4], 20000, 0, vec![2, 3], 10000, 0);
        SpectatorsMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("20,000 fans have come to support ")));
    }
}
