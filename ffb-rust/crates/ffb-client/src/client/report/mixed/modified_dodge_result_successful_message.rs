use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_modified_dodge_result_successful::ReportModifiedDodgeResultSuccessful;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `ModifiedDodgeResultSuccessfulMessage.java`.
pub struct ModifiedDodgeResultSuccessfulMessage;

impl ReportMessage for ModifiedDodgeResultSuccessfulMessage {
    type Report = ReportModifiedDodgeResultSuccessful;

    fn report_id(&self) -> ReportId {
        ReportId::MODIFIED_DODGE_RESULT_SUCCESSFUL
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        // java: Skill.getName() approximated with SkillId::class_name() — full display-name
        // formatting not modeled. Java's `report.getSkill()` is called unconditionally
        // (would NPE on null); here a missing skill_id renders as an empty name.
        let skill_name = report.get_skill_id().map(|s| s.class_name()).unwrap_or_default();
        status_report.println_indent_style(
            status_report.get_indent() + 1,
            TextStyle::EXPLANATION,
            &format!("Using {skill_name} would result in a successful dodge"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::enums::SkillId;
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
    fn renders_skill_name() {
        let game = make_game();
        let report = ReportModifiedDodgeResultSuccessful::new(Some(SkillId::Dodge));
        let mut status_report = StatusReport::new();
        ModifiedDodgeResultSuccessfulMessage.render(&mut status_report, &game, &report);
        assert_eq!(
            status_report.rendered_runs[0].text.as_deref(),
            Some("Using Dodge would result in a successful dodge")
        );
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::EXPLANATION));
    }

    #[test]
    fn indent_offset_by_one() {
        let mut game = make_game();
        game.testing = true; // no-op, kept to differentiate fixture from other tests
        let mut status_report = StatusReport::new();
        status_report.set_indent(2);
        let report = ReportModifiedDodgeResultSuccessful::new(Some(SkillId::Dodge));
        ModifiedDodgeResultSuccessfulMessage.render(&mut status_report, &game, &report);
        use crate::client::paragraph_style::ParagraphStyle;
        assert_eq!(status_report.rendered_runs[0].paragraph_style, Some(ParagraphStyle::INDENT_3));
    }

    #[test]
    fn missing_skill_renders_empty_name() {
        let game = make_game();
        let report = ReportModifiedDodgeResultSuccessful::new(None);
        let mut status_report = StatusReport::new();
        ModifiedDodgeResultSuccessfulMessage.render(&mut status_report, &game, &report);
        assert_eq!(
            status_report.rendered_runs[0].text.as_deref(),
            Some("Using  would result in a successful dodge")
        );
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(ModifiedDodgeResultSuccessfulMessage.report_id(), ReportId::MODIFIED_DODGE_RESULT_SUCCESSFUL);
        assert_eq!(ModifiedDodgeResultSuccessfulMessage.get_key(), "modifiedDodgeResultSuccessful");
    }
}
