use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_modified_pass_result::ReportModifiedPassResult;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `ModifiedPassResultMessage.java`.
pub struct ModifiedPassResultMessage;

impl ReportMessage for ModifiedPassResultMessage {
    type Report = ReportModifiedPassResult;

    fn report_id(&self) -> ReportId {
        ReportId::MODIFIED_PASS_RESULT
    }

    fn render(&self, status_report: &mut StatusReport, _game: &Game, report: &Self::Report) {
        // java: Skill.getName() approximated with SkillId::class_name() — full display-name
        // formatting not modeled. Java's `report.getSkill()` is called unconditionally
        // (would NPE on null); here a missing skill_id renders as an empty name.
        let skill_name = report.get_skill_id().map(|s| s.class_name()).unwrap_or_default();
        status_report.println_indent_style(
            status_report.get_indent() + 1,
            TextStyle::EXPLANATION,
            &format!(
                "Using {skill_name} would change the result to {}",
                report.get_pass_result()
            ),
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
    fn renders_skill_and_pass_result() {
        let game = make_game();
        let report = ReportModifiedPassResult::new(Some(SkillId::Pass), "Fumble".into());
        let mut status_report = StatusReport::new();
        ModifiedPassResultMessage.render(&mut status_report, &game, &report);
        assert_eq!(
            status_report.rendered_runs[0].text.as_deref(),
            Some("Using Pass would change the result to Fumble")
        );
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::EXPLANATION));
    }

    #[test]
    fn missing_skill_renders_empty_name() {
        let game = make_game();
        let report = ReportModifiedPassResult::new(None, "Accurate".into());
        let mut status_report = StatusReport::new();
        ModifiedPassResultMessage.render(&mut status_report, &game, &report);
        assert_eq!(
            status_report.rendered_runs[0].text.as_deref(),
            Some("Using  would change the result to Accurate")
        );
    }

    #[test]
    fn different_pass_result_value() {
        let game = make_game();
        let report = ReportModifiedPassResult::new(Some(SkillId::Pass), "Inaccurate".into());
        let mut status_report = StatusReport::new();
        ModifiedPassResultMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs[0].text.as_deref().unwrap().ends_with("Inaccurate"));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(ModifiedPassResultMessage.report_id(), ReportId::MODIFIED_PASS_RESULT);
        assert_eq!(ModifiedPassResultMessage.get_key(), "modifiedPassResult");
    }
}
