use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::report::bb2020::report_skill_use_other_player::ReportSkillUseOtherPlayer;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `SkillUseOtherPlayerMessage.java`.
pub struct SkillUseOtherPlayerMessage;

impl ReportMessage for SkillUseOtherPlayerMessage {
    type Report = ReportSkillUseOtherPlayer;

    fn report_id(&self) -> ReportId {
        ReportId::SKILL_USE_OTHER_PLAYER
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = game.player(report.get_player_id());
        let other_player = game.player(report.get_other_player_id());
        let indent = status_report.get_indent();

        print_player(status_report, game, indent, false, player);
        // java: `report.getSkill().getName()` — this Rust report already stores the
        // resolved skill name as a plain string.
        let status = format!(" uses {} of ", report.get_skill());
        status_report.print_indent(indent, &status);
        print_player(status_report, game, indent, false, other_player);

        // java: `SkillUse.getDescription(player)` — ffb-model's ReportSkillUseOtherPlayer
        // already stores the resolved description string in `skill_use`.
        let status = format!(" {}.", report.get_skill_use());
        status_report.println_indent(indent, &status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str, players: Vec<Player>) -> Team {
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
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_player(id: &str, name: &str) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![make_player("p1", "User")]);
        let away = make_team("away", vec![make_player("p2", "Target")]);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn renders_skill_use_between_players() {
        let game = make_game();
        let report = ReportSkillUseOtherPlayer::new("p1".into(), "Guard".into(), "grants a +1 to the block roll".into(), "p2".into());
        let mut status_report = StatusReport::new();
        SkillUseOtherPlayerMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("User"));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" uses Guard of "));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Target"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" grants a +1 to the block roll."));
    }

    #[test]
    fn different_skill_names() {
        let game = make_game();
        let report = ReportSkillUseOtherPlayer::new("p2".into(), "Tackle".into(), "negates Dodge".into(), "p1".into());
        let mut status_report = StatusReport::new();
        SkillUseOtherPlayerMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Target"));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" uses Tackle of "));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("User"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" negates Dodge."));
    }

    #[test]
    fn missing_players_still_renders_skill_text() {
        let game = make_game();
        let report = ReportSkillUseOtherPlayer::new("unknown1".into(), "Guard".into(), "grants a bonus".into(), "unknown2".into());
        let mut status_report = StatusReport::new();
        SkillUseOtherPlayerMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs.len(), 3); // status1 + status2 text + println terminator
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some(" uses Guard of "));
        assert_eq!(status_report.rendered_runs[1].text.as_deref(), Some(" grants a bonus."));
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(SkillUseOtherPlayerMessage.report_id(), ReportId::SKILL_USE_OTHER_PLAYER);
        assert_eq!(SkillUseOtherPlayerMessage.get_key(), "skillUseOtherPlayer");
    }
}
