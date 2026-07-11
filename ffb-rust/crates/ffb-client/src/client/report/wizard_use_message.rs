use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::special_effect::SpecialEffect;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_wizard_use::ReportWizardUse;

pub struct WizardUseMessage;

impl ReportMessage for WizardUseMessage {
    type Report = ReportWizardUse;

    fn report_id(&self) -> ReportId {
        ReportId::WIZARD_USE
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        status_report.print_indent_style(indent, TextStyle::BOLD, "The team wizard of ");
        if game.team_home.id == report.get_team_id() {
            status_report.print_indent_style(indent, TextStyle::HOME_BOLD, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(indent, TextStyle::AWAY_BOLD, &game.team_away.name.clone());
        }
        match report.get_wizard_spell() {
            SpecialEffect::LIGHTNING => {
                status_report.println_indent_style(indent, TextStyle::BOLD, " casts a Lightning spell.");
            }
            SpecialEffect::ZAP => {
                status_report.println_indent_style(indent, TextStyle::BOLD, " casts a Zap! spell.");
            }
            _ => {
                status_report.println_indent_style(indent, TextStyle::BOLD, " casts a Fireball spell.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, name: &str) -> Team {
        Team {
            id: id.to_string(), name: name.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home", "Home Wizards"), make_team("away", "Away Wizards"), Rules::Bb2025)
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(WizardUseMessage.report_id(), ReportId::WIZARD_USE);
    }

    #[test]
    fn home_team_lightning() {
        let game = make_game();
        let report = ReportWizardUse::new("home".into(), SpecialEffect::LIGHTNING);
        let mut status_report = StatusReport::new();
        WizardUseMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Home Wizards".to_string())));
        assert!(texts.contains(&Some(" casts a Lightning spell.".to_string())));
    }

    #[test]
    fn away_team_zap() {
        let game = make_game();
        let report = ReportWizardUse::new("away".into(), SpecialEffect::ZAP);
        let mut status_report = StatusReport::new();
        WizardUseMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Away Wizards".to_string())));
        assert!(texts.contains(&Some(" casts a Zap! spell.".to_string())));
    }

    #[test]
    fn fireball_default() {
        let game = make_game();
        let report = ReportWizardUse::new("home".into(), SpecialEffect::FIREBALL);
        let mut status_report = StatusReport::new();
        WizardUseMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some(" casts a Fireball spell.".to_string())));
    }

    #[test]
    fn unknown_team_id_falls_through_to_away() {
        let game = make_game();
        let report = ReportWizardUse::new("nonexistent".into(), SpecialEffect::FIREBALL);
        let mut status_report = StatusReport::new();
        WizardUseMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Away Wizards".to_string())));
    }
}
