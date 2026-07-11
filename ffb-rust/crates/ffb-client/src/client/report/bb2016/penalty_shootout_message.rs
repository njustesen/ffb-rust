use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_penalty_shootout::ReportPenaltyShootout;
use ffb_model::report::report_id::ReportId;

pub struct PenaltyShootoutMessage;

impl ReportMessage for PenaltyShootoutMessage {
    type Report = ReportPenaltyShootout;

    fn report_id(&self) -> ReportId {
        ReportId::PENALTY_SHOOTOUT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let penalty_score_home = report.get_roll_home() + report.get_re_rolls_left_home();
        status_report.print_indent_style(0, TextStyle::ROLL, &format!("Penalty Shootout Roll Home [{}]", report.get_roll_home()));
        status_report.print_indent_style(0, TextStyle::ROLL, &format!(" + {} ReRolls", report.get_re_rolls_left_home()));
        status_report.println_indent_style(0, TextStyle::ROLL, &format!(" = {}", penalty_score_home));
        let penalty_score_away = report.get_roll_away() + report.get_re_rolls_left_away();
        status_report.print_indent_style(0, TextStyle::ROLL, &format!("Penalty Shootout Roll Away [{}]", report.get_roll_away()));
        status_report.print_indent_style(0, TextStyle::ROLL, &format!(" + {} ReRolls", report.get_re_rolls_left_away()));
        status_report.println_indent_style(0, TextStyle::ROLL, &format!(" = {}", penalty_score_away));
        if penalty_score_home > penalty_score_away {
            status_report.print_indent_style(1, TextStyle::HOME, &game.team_home.name.clone());
            status_report.println_indent_style(1, TextStyle::NONE, " win the penalty shootout.");
        } else {
            status_report.print_indent_style(1, TextStyle::AWAY, &game.team_away.name.clone());
            status_report.println_indent_style(1, TextStyle::NONE, " win the penalty shootout.");
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
    fn get_key_is_penalty_shootout() {
        assert_eq!(PenaltyShootoutMessage.get_key(), "penaltyShootout");
    }

    #[test]
    fn home_wins_when_higher_score() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPenaltyShootout::new(5, 1, 3, 0);
        PenaltyShootoutMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Team home")));
    }

    #[test]
    fn away_wins_when_higher_or_equal_score() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPenaltyShootout::new(2, 0, 4, 0);
        PenaltyShootoutMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Team away")));
    }

    #[test]
    fn reports_roll_totals() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPenaltyShootout::new(5, 1, 3, 0);
        PenaltyShootoutMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Penalty Shootout Roll Home [5]"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some(" = 6"));
    }
}
