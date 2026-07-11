use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_inducement::ReportInducement;

/// 1:1 translation of `InducementMessage.java`.
pub struct InducementMessage;

impl ReportMessage for InducementMessage {
    type Report = ReportInducement;

    fn report_id(&self) -> ReportId {
        ReportId::INDUCEMENT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();

        // java: `StringTool.isProvided(pReport.getTeamId()) && (pReport.getInducementType() != null)`.
        // `ReportInducement` stores both fields as plain (non-`Option`) strings, so "provided"
        // is approximated as "non-empty" for each.
        if !report.get_team_id().is_empty() && !report.get_inducement_type().is_empty() {
            if report.get_team_id() == game.team_home.id {
                status_report.print_indent_style(indent, TextStyle::HOME, &game.team_home.name.clone());
            } else {
                status_report.print_indent_style(indent, TextStyle::AWAY, &game.team_away.name.clone());
            }

            let inducement_type = report.get_inducement_type();

            // java: `InducementType.hasUsage(Usage.X)` approximated by matching known type
            // names since ffb-model's ReportInducement only stores the type name string, not
            // full usage metadata. Java's `inducementType.getDescription()` (used in the
            // REGENERATION branch to print e.g. "Igor"/"Mortuary Assistant"/"Plague Doctor")
            // has no equivalent lookup available here, so the raw type name string is printed
            // instead.
            match inducement_type {
                "extraTeamTraining" => {
                    status_report.print_indent(indent, " use ");
                    status_report.print_indent_style(indent, TextStyle::BOLD, "Extra Team Training");
                    let value = report.get_value();
                    let status = format!(
                        " to add {}{}",
                        value,
                        if value == 1 { " Re-Roll." } else { " Re-Rolls." }
                    );
                    status_report.println_indent(indent, &status);
                }
                "wanderingApothecaries" => {
                    status_report.print_indent(indent, " use ");
                    status_report.print_indent_style(indent, TextStyle::BOLD, "Wandering Apothecaries");
                    let value = report.get_value();
                    let status = format!(
                        " to add {}{}",
                        value,
                        if value == 1 { " Apothecary." } else { " Apothecaries." }
                    );
                    status_report.println_indent(indent, &status);
                }
                "igor" | "mortuaryAssistant" | "plagueDoctor" => {
                    status_report.print_indent(indent, " use ");
                    status_report.print_indent_style(indent, TextStyle::BOLD, inducement_type);
                    status_report.println_indent(indent, " to re-roll the failed Regeneration.");
                }
                _ => {}
            }
        }
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
            name: format!("Team {id}"),
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
    fn renders_extra_team_training_singular() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportInducement::new("home".into(), "extraTeamTraining".into(), 1);
        InducementMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Team home"));
        assert_eq!(sr.rendered_runs[1].text.as_deref(), Some(" use "));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Extra Team Training"));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" to add 1 Re-Roll."));
    }

    #[test]
    fn renders_wandering_apothecaries_plural() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportInducement::new("away".into(), "wanderingApothecaries".into(), 2);
        InducementMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Team away"));
        assert_eq!(sr.rendered_runs[2].text.as_deref(), Some("Wandering Apothecaries"));
        assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" to add 2 Apothecaries."));
    }

    #[test]
    fn renders_regeneration_branch_for_known_types() {
        for type_name in ["igor", "mortuaryAssistant", "plagueDoctor"] {
            let mut sr = StatusReport::new();
            let game = make_game();
            let report = ReportInducement::new("home".into(), type_name.into(), 1);
            InducementMessage.render(&mut sr, &game, &report);
            assert_eq!(sr.rendered_runs[2].text.as_deref(), Some(type_name));
            assert_eq!(sr.rendered_runs[3].text.as_deref(), Some(" to re-roll the failed Regeneration."));
        }
    }

    #[test]
    fn renders_nothing_when_team_id_empty() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportInducement::new("".into(), "extraTeamTraining".into(), 1);
        InducementMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.is_empty());
    }

    #[test]
    fn renders_nothing_for_unknown_inducement_type() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = ReportInducement::new("home".into(), "bribery".into(), 1);
        InducementMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs.len(), 1); // team name only, no usage branch matched
    }
}
