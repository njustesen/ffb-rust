use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_inducement::ReportInducement;

pub struct InducementMessage;

impl ReportMessage for InducementMessage {
    type Report = ReportInducement;

    fn report_id(&self) -> ReportId {
        ReportId::INDUCEMENT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        if report.get_team_id().is_empty() {
            return;
        }
        if game.team_home.id == report.get_team_id() {
            status_report.print_indent_style(indent, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(indent, TextStyle::AWAY, &game.team_away.name.clone());
        }
        // java: inducementType.hasUsage(Usage.X) — the resolved ReportInducement only carries
        // the inducement type id string (not the full InducementType/Usage data), so the three
        // branches below are matched against the known bb2016 inducement collection ids
        // ("extraTeamTraining" -> Usage.REROLL, "wanderingApothecaries" -> Usage.APOTHECARY,
        // "igor" -> Usage.REGENERATION) instead.
        match report.get_inducement_type() {
            "extraTeamTraining" => {
                status_report.print_indent(indent, " use ");
                status_report.print_indent_style(indent, TextStyle::BOLD, "Extra Team Training");
                let suffix = if report.get_value() == 1 { " Re-Roll." } else { " Re-Rolls." };
                status_report.println_indent(indent, &format!(" to add {}{}", report.get_value(), suffix));
            }
            "wanderingApothecaries" => {
                status_report.print_indent(indent, " use ");
                status_report.print_indent_style(indent, TextStyle::BOLD, "Wandering Apothecaries");
                let suffix = if report.get_value() == 1 { " Apothecary." } else { " Apothecaries." };
                status_report.println_indent(indent, &format!(" to add {}{}", report.get_value(), suffix));
            }
            "igor" => {
                status_report.print_indent(indent, " use ");
                status_report.print_indent_style(indent, TextStyle::BOLD, "Igor");
                status_report.println_indent(indent, " to re-roll the failed Regeneration.");
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_inducement() {
        assert_eq!(InducementMessage.get_key(), "inducement");
    }

    #[test]
    fn extra_team_training_plural_rerolls() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportInducement::new("home".into(), "extraTeamTraining".into(), 2);
        InducementMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" to add 2 Re-Rolls.")));
    }

    #[test]
    fn wandering_apothecaries_singular() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportInducement::new("away".into(), "wanderingApothecaries".into(), 1);
        InducementMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" to add 1 Apothecary.")));
    }

    #[test]
    fn igor_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportInducement::new("home".into(), "igor".into(), 1);
        InducementMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" to re-roll the failed Regeneration.")));
    }

    #[test]
    fn empty_team_id_produces_no_output() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportInducement::new("".into(), "igor".into(), 1);
        InducementMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }
}
