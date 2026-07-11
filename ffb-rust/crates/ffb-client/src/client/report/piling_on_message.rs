use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_piling_on::ReportPilingOn;

/// 1:1 translation of `PilingOnMessage.java`.
pub struct PilingOnMessage;

impl ReportMessage for PilingOnMessage {
    type Report = ReportPilingOn;

    fn report_id(&self) -> ReportId {
        ReportId::PILING_ON
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        if let Some(player) = game.player(report.get_player_id()) {
            if let Some(skill_id) = player.skill_id_with_property(NamedProperties::CAN_PILE_ON_OPPONENT) {
                let indent = status_report.get_indent() + 1;
                print_player(status_report, game, indent, false, Some(player));
                // java: skill.getName() — Player only retains a SkillId here (not a full Skill
                // object with a data-driven display name), so the PascalCase class name is used
                // as the closest available approximation.
                let skill_name = skill_id.class_name();
                let status = if !report.is_used() {
                    format!(" does not use {skill_name}.")
                } else {
                    let re_roll_kind = if report.is_re_roll_injury() { "Injury" } else { "Armor" };
                    format!(" uses {skill_name} to re-roll {re_roll_kind}.")
                };
                status_report.println_indent(indent, &status);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::SkillId;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false }
    }

    fn make_player_with_piling_on(id: &str) -> Player {
        let mut p = Player::default();
        p.id = id.to_string();
        p.name = format!("Player {id}");
        p.add_skill(SkillId::PilingOn);
        p
    }

    fn make_game_with_piling_on_player(id: &str) -> Game {
        let mut home = make_team("home");
        home.players.push(make_player_with_piling_on(id));
        Game::new(home, make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn player_without_skill_renders_nothing() {
        let mut home = make_team("home");
        let mut p = Player::default();
        p.id = "p1".into();
        home.players.push(p);
        let game = Game::new(home, make_team("away"), Rules::Bb2025);
        let mut status_report = StatusReport::new();
        let report = ReportPilingOn::new("p1".into(), true, false);
        PilingOnMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn used_re_roll_injury_reports_injury() {
        let game = make_game_with_piling_on_player("p1");
        let mut status_report = StatusReport::new();
        let report = ReportPilingOn::new("p1".into(), true, true);
        PilingOnMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " uses PilingOn to re-roll Injury."));
    }

    #[test]
    fn used_re_roll_armor_reports_armor() {
        let game = make_game_with_piling_on_player("p1");
        let mut status_report = StatusReport::new();
        let report = ReportPilingOn::new("p1".into(), true, false);
        PilingOnMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " uses PilingOn to re-roll Armor."));
    }

    #[test]
    fn not_used_reports_does_not_use() {
        let game = make_game_with_piling_on_player("p1");
        let mut status_report = StatusReport::new();
        let report = ReportPilingOn::new("p1".into(), false, false);
        PilingOnMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == " does not use PilingOn."));
    }

    #[test]
    fn unknown_player_id_renders_nothing() {
        let game = make_game_with_piling_on_player("p1");
        let mut status_report = StatusReport::new();
        let report = ReportPilingOn::new("unknown".into(), true, false);
        PilingOnMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn get_key_matches_report_id() {
        assert_eq!(PilingOnMessage.get_key(), "pilingOn");
    }
}
