use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::report_skill_use::ReportSkillUse;

pub struct SkillUseMessage;

impl ReportMessage for SkillUseMessage {
    type Report = ReportSkillUse;

    fn report_id(&self) -> ReportId {
        ReportId::SKILL_USE
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        // java: `if (report.getSkill() != null)` — the Rust `ReportSkillUse.skill` field is
        // non-optional (`SkillId`), so this branch is always true; kept structurally to mirror
        // the Java control flow.
        let player = report.get_player_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent();
        let mut status = String::new();
        // java: `report.getSkill().getName()` — no display-name lookup exists yet for
        // `SkillId` (only `class_name()`, e.g. "MightyBlow"), so `class_name()` is used as the
        // closest available approximation.
        let skill_name = report.get_skill().class_name();
        if !report.is_used() {
            if let Some(player) = player {
                print_player(status_report, game, indent, false, Some(player));
                status.push_str(" does not use ");
                status.push_str(skill_name);
            } else {
                status.push_str(skill_name);
                status.push_str(" is not used");
            }
            status.push_str(&format!(" {}", report.get_skill_use().get_description()));
            status.push('.');
            status_report.println_indent(indent, &status);
        } else {
            if let Some(player) = player {
                print_player(status_report, game, indent, false, Some(player));
                status.push_str(" uses ");
                status.push_str(skill_name);
            } else {
                status.push_str(skill_name);
                status.push_str(" used");
            }
            status.push_str(&format!(" {}", report.get_skill_use().get_description()));
            status.push('.');
            status_report.println_indent(indent, &status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_use::SkillUse;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, name: &str) {
        let mut player = Player::default();
        player.id = id.to_string();
        player.name = name.to_string();
        game.team_home.players.push(player);
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(SkillUseMessage.report_id(), ReportId::SKILL_USE);
    }

    #[test]
    fn used_skill_with_player_renders_uses() {
        let mut game = make_game();
        add_player(&mut game, "p1", "Griff");
        let report = ReportSkillUse::new(Some("p1".into()), SkillId::Block, true, SkillUse::BRING_DOWN_OPPONENT);
        let mut status_report = StatusReport::new();
        SkillUseMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.contains(&Some("Griff".to_string())));
        assert!(texts.iter().any(|t| t.as_deref().is_some_and(|s| s.starts_with(" uses Block"))));
    }

    #[test]
    fn unused_skill_with_player_renders_does_not_use() {
        let mut game = make_game();
        add_player(&mut game, "p1", "Griff");
        let report = ReportSkillUse::new(Some("p1".into()), SkillId::Block, false, SkillUse::WOULD_NOT_HELP);
        let mut status_report = StatusReport::new();
        SkillUseMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.as_deref().is_some_and(|s| s.starts_with(" does not use Block"))));
    }

    #[test]
    fn no_player_found_renders_skill_name_only() {
        let game = make_game();
        let report = ReportSkillUse::new(Some("unknown".into()), SkillId::Dodge, true, SkillUse::AVOID_PUSH);
        let mut status_report = StatusReport::new();
        SkillUseMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.as_deref().is_some_and(|s| s.starts_with("Dodge used"))));
    }

    #[test]
    fn no_player_id_renders_skill_name_only() {
        let game = make_game();
        let report = ReportSkillUse::new(None, SkillId::Dodge, false, SkillUse::WOULD_NOT_HELP);
        let mut status_report = StatusReport::new();
        SkillUseMessage.render(&mut status_report, &game, &report);
        let texts: Vec<Option<String>> = status_report.rendered_runs.iter().map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.as_deref().is_some_and(|s| s.starts_with("Dodge is not used"))));
    }
}
