use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_coin_throw::ReportCoinThrow;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `CoinThrowMessage.java`.
pub struct CoinThrowMessage;

impl ReportMessage for CoinThrowMessage {
    type Report = ReportCoinThrow;

    fn report_id(&self) -> ReportId {
        ReportId::COIN_THROW
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        status_report.set_indent(0);
        status_report.println_indent_style(status_report.get_indent(), TextStyle::BOLD, "The referee throws the coin.");
        status_report.print_indent(status_report.get_indent() + 1, "Coach ");
        if game.team_home.coach == report.get_coach() {
            status_report.print_indent_style(status_report.get_indent() + 1, TextStyle::HOME, report.get_coach());
        } else {
            status_report.print_indent_style(status_report.get_indent() + 1, TextStyle::AWAY, report.get_coach());
        }
        let status = format!(" chooses {}", if report.is_coin_choice_heads() { "HEADS." } else { "TAILS." });
        status_report.println_indent(status_report.get_indent() + 1, &status);
        let status = format!("Coin throw is {}", if report.is_coin_throw_heads() { "HEADS." } else { "TAILS." });
        status_report.println_indent(status_report.get_indent() + 1, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::paragraph_style::ParagraphStyle;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, coach: &str) -> Team {
        Team { id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: coach.to_string(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        Game::new(make_team("home", "CoachA"), make_team("away", "CoachB"), Rules::Bb2025)
    }

    #[test]
    fn home_coach_uses_home_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportCoinThrow::new(true, "CoachA".into(), false);
        CoinThrowMessage.render(&mut status_report, &game, &report);
        let coach_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("CoachA"));
        assert_eq!(coach_run.unwrap().text_style, Some(TextStyle::HOME));
    }

    #[test]
    fn away_coach_uses_away_style() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportCoinThrow::new(true, "CoachB".into(), false);
        CoinThrowMessage.render(&mut status_report, &game, &report);
        let coach_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("CoachB"));
        assert_eq!(coach_run.unwrap().text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn coin_choice_and_throw_text() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportCoinThrow::new(false, "CoachA".into(), true);
        CoinThrowMessage.render(&mut status_report, &game, &report);
        let choice = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" chooses HEADS."));
        assert!(choice.is_some());
        let throw = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Coin throw is TAILS."));
        assert!(throw.is_some());
    }

    #[test]
    fn resets_indent_to_zero() {
        let mut status_report = StatusReport::new();
        status_report.set_indent(3);
        let game = make_game();
        let report = ReportCoinThrow::new(true, "CoachA".into(), false);
        CoinThrowMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].paragraph_style, Some(ParagraphStyle::INDENT_0));
    }

    #[test]
    fn report_id_is_coin_throw() {
        assert_eq!(CoinThrowMessage.report_id(), ReportId::COIN_THROW);
    }
}
