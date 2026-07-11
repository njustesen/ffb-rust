use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_skill_wasted::ReportSkillWasted;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `SkillWastedMessage.java`.
pub struct SkillWastedMessage;

impl ReportMessage for SkillWastedMessage {
    type Report = ReportSkillWasted;

    fn report_id(&self) -> ReportId {
        ReportId::SKILL_WASTED
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        if let Some(skill) = report.get_skill() {
            let player = report.get_player_id().and_then(|id| game.player(id));
            let indent = status_report.get_indent();
            // java: `Skill.getName()` approximated via `SkillId::class_name()`.
            let skill_name = skill.class_name();
            let status = if let Some(player) = player {
                print_player(status_report, game, indent, false, Some(player));
                format!(" wastes {skill_name}.")
            } else {
                format!("{skill_name} is wasted.")
            };
            status_report.println_indent(indent, &status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId};
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
        let home = make_team("home", vec![make_player("p1", "Wastey")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn skill_none_renders_nothing() {
        let game = make_game();
        let report = ReportSkillWasted::new(Some("p1".into()), None);
        let mut status_report = StatusReport::new();
        SkillWastedMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn skill_with_player() {
        let game = make_game();
        let report = ReportSkillWasted::new(Some("p1".into()), Some(SkillId::Dodge));
        let mut status_report = StatusReport::new();
        SkillWastedMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Wastey"));
        assert_eq!(
            status_report.rendered_runs[1].text.as_deref(),
            Some(format!(" wastes {}.", SkillId::Dodge.class_name())).as_deref()
        );
    }

    #[test]
    fn skill_without_player() {
        let game = make_game();
        let report = ReportSkillWasted::new(None, Some(SkillId::Dodge));
        let mut status_report = StatusReport::new();
        SkillWastedMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs.len(), 2); // text run + println terminator
        assert_eq!(
            status_report.rendered_runs[0].text.as_deref(),
            Some(format!("{} is wasted.", SkillId::Dodge.class_name())).as_deref()
        );
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(SkillWastedMessage.report_id(), ReportId::SKILL_WASTED);
        assert_eq!(SkillWastedMessage.get_key(), "skillWasted");
    }
}
