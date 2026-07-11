use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_penalty_shootout::ReportPenaltyShootout;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `PenaltyShootoutMessage.java`.
pub struct PenaltyShootoutMessage;

impl ReportMessage for PenaltyShootoutMessage {
    type Report = ReportPenaltyShootout;

    fn report_id(&self) -> ReportId {
        ReportId::PENALTY_SHOOTOUT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        status_report.println_indent_style(
            indent,
            TextStyle::ROLL,
            &format!(
                "{} Penalty Shootout Rolls: Home [{}] Away [{}]",
                report.get_roll_count().unwrap_or_default(),
                report.get_roll_home(),
                report.get_roll_away()
            ),
        );
        if let Some(home_team_won_penalty) = report.get_home_team_won_penalty() {
            let coach_style = if home_team_won_penalty { TextStyle::HOME } else { TextStyle::AWAY };
            let team_name = if home_team_won_penalty { game.team_home.name.clone() } else { game.team_away.name.clone() };
            status_report.print_indent_style(indent + 1, coach_style, &team_name);
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " win this penalty");
            status_report.print_indent_style(indent + 1, TextStyle::NONE, "Current score: ");
            status_report.print_indent_style(indent + 1, TextStyle::HOME, &game.team_home.name.clone());
            status_report.print_indent_style(
                indent + 1,
                TextStyle::NONE,
                &format!(" {} - {} ", report.get_score_home(), report.get_score_away()),
            );
            status_report.println_indent_style(indent + 1, TextStyle::AWAY, &game.team_away.name.clone());
        } else {
            status_report.println_indent_style(indent + 1, TextStyle::NONE, "Penalty is rerolled");
        }

        // java: StringTool.isProvided(s) — s != null && !s.isEmpty().
        if report.get_winning_team().is_some_and(|s| !s.is_empty()) {
            let team_id = report.get_winning_team().unwrap();
            let team = game.team_by_id(team_id);
            let team_style = if team.is_some_and(|t| t.id == game.team_home.id) { TextStyle::HOME } else { TextStyle::AWAY };
            if let Some(team) = team {
                status_report.print_indent_style(indent, team_style, &team.name.clone());
            }
            status_report.println_indent_style(indent, TextStyle::NONE, " win sudden death");
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
            coach: format!("Coach {id}"),
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
        Game::new(make_team("home"), make_team("away"), Rules::Bb2020)
    }

    #[test]
    fn home_wins_penalty_with_winning_team() {
        let game = make_game();
        let report = ReportPenaltyShootout::new(4, 1, 3, 0, Some(true), Some("2".into()), Some("home".into()));
        let mut status_report = StatusReport::new();
        PenaltyShootoutMessage.render(&mut status_report, &game, &report);
        assert_eq!(
            status_report.rendered_runs[0].text.as_deref(),
            Some("2 Penalty Shootout Rolls: Home [4] Away [3]")
        );
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Team home"));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::HOME));
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&" win sudden death"));
    }

    #[test]
    fn away_wins_penalty_no_winning_team() {
        let game = make_game();
        let report = ReportPenaltyShootout::new(2, 1, 5, 2, Some(false), Some("1".into()), None);
        let mut status_report = StatusReport::new();
        PenaltyShootoutMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Team away"));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::AWAY));
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(!texts.contains(&" win sudden death"));
    }

    #[test]
    fn no_winner_penalty_rerolled() {
        let game = make_game();
        let report = ReportPenaltyShootout::new(3, 3, 3, 3, None, Some("3".into()), None);
        let mut status_report = StatusReport::new();
        PenaltyShootoutMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Penalty is rerolled"));
    }

    #[test]
    fn empty_winning_team_string_is_not_provided() {
        let game = make_game();
        let report = ReportPenaltyShootout::new(1, 0, 1, 0, Some(true), Some("1".into()), Some(String::new()));
        let mut status_report = StatusReport::new();
        PenaltyShootoutMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(!texts.contains(&" win sudden death"));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(PenaltyShootoutMessage.report_id(), ReportId::PENALTY_SHOOTOUT);
        assert_eq!(PenaltyShootoutMessage.get_key(), "penaltyShootout");
    }
}
