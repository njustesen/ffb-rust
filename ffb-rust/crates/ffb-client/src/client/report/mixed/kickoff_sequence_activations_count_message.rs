use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_kickoff_sequence_activations_count::ReportKickoffSequenceActivationsCount;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `KickoffSequenceActivationsCountMessage.java`.
pub struct KickoffSequenceActivationsCountMessage;

impl ReportMessage for KickoffSequenceActivationsCountMessage {
    type Report = ReportKickoffSequenceActivationsCount;

    fn report_id(&self) -> ReportId {
        ReportId::KICKOFF_SEQUENCE_ACTIVATIONS_COUNT
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        let remain = if report.get_available() == 1 { "remains" } else { "remain" };
        let text = format!(
            "Max {} open players can be used - {} used ({} {} open).",
            report.get_limit(),
            report.get_amount(),
            report.get_available(),
            remain
        );
        status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::EXPLANATION, &text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: id.into(),
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
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020)
    }

    #[test]
    fn renders_plural_remain_text() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffSequenceActivationsCount::new(2, 5, 3);
        KickoffSequenceActivationsCountMessage.render(&mut sr, &game, &report);
        assert_eq!(
            sr.rendered_runs[0].text.as_deref(),
            Some("Max 3 open players can be used - 2 used (5 remain open).")
        );
        assert_eq!(sr.rendered_runs[0].text_style, Some(TextStyle::EXPLANATION));
    }

    #[test]
    fn renders_singular_remains_text_when_available_is_one() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffSequenceActivationsCount::new(4, 1, 5);
        KickoffSequenceActivationsCountMessage.render(&mut sr, &game, &report);
        assert_eq!(
            sr.rendered_runs[0].text.as_deref(),
            Some("Max 5 open players can be used - 4 used (1 remains open).")
        );
    }

    #[test]
    fn renders_at_current_indent_plus_one() {
        let mut sr = StatusReport::new();
        sr.set_indent(1);
        let game = make_game();
        let report = ReportKickoffSequenceActivationsCount::new(0, 3, 3);
        KickoffSequenceActivationsCountMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs[0].text.as_deref().unwrap().contains("Max 3"));
    }
}
