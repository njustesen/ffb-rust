use crate::client::paragraph_style::ParagraphStyle;
use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_timeout_enforced::ReportTimeoutEnforced;

pub struct TimeoutEnforcedMessage;

impl ReportMessage for TimeoutEnforcedMessage {
    type Report = ReportTimeoutEnforced;

    fn report_id(&self) -> ReportId {
        ReportId::TIMEOUT_ENFORCED
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let status = format!("Coach {} forces a Timeout.", report.get_coach());
        if game.team_home.coach == report.get_coach() {
            status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE), Some(TextStyle::HOME_BOLD), &status);
        } else {
            status_report.println_style(Some(ParagraphStyle::SPACE_ABOVE), Some(TextStyle::AWAY_BOLD), &status);
        }
        status_report.println_style(
            Some(ParagraphStyle::SPACE_BELOW),
            Some(TextStyle::NONE),
            "The turn will end after the Acting Player has finished moving.",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, coach: &str) -> Team {
        Team {
            id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: coach.to_string(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home", "HomeCoach"), make_team("away", "AwayCoach"), Rules::Bb2025)
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(TimeoutEnforcedMessage.report_id(), ReportId::TIMEOUT_ENFORCED);
    }

    #[test]
    fn home_coach_uses_home_bold() {
        let game = make_game();
        let report = ReportTimeoutEnforced::new("HomeCoach".into());
        let mut status_report = StatusReport::new();
        TimeoutEnforcedMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Coach HomeCoach forces a Timeout."));
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::HOME_BOLD));
        assert_eq!(status_report.rendered_runs[0].paragraph_style, Some(ParagraphStyle::SPACE_ABOVE));
    }

    #[test]
    fn away_coach_uses_away_bold() {
        let game = make_game();
        let report = ReportTimeoutEnforced::new("AwayCoach".into());
        let mut status_report = StatusReport::new();
        TimeoutEnforcedMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::AWAY_BOLD));
    }

    #[test]
    fn unknown_coach_falls_through_to_away_bold() {
        let game = make_game();
        let report = ReportTimeoutEnforced::new("SomeoneElse".into());
        let mut status_report = StatusReport::new();
        TimeoutEnforcedMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::AWAY_BOLD));
    }

    #[test]
    fn second_line_uses_space_below_and_none_style() {
        let game = make_game();
        let report = ReportTimeoutEnforced::new("HomeCoach".into());
        let mut status_report = StatusReport::new();
        TimeoutEnforcedMessage.render(&mut status_report, &game, &report);
        assert_eq!(
            status_report.rendered_runs[2].text.as_deref(),
            Some("The turn will end after the Acting Player has finished moving.")
        );
        assert_eq!(status_report.rendered_runs[2].paragraph_style, Some(ParagraphStyle::SPACE_BELOW));
        assert_eq!(status_report.rendered_runs[2].text_style, Some(TextStyle::NONE));
    }
}
