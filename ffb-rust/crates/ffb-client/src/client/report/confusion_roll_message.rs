use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::factory::skill_factory::SkillFactory;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::report_confusion_roll::ReportConfusionRoll;
use ffb_model::report::report_id::ReportId;

/// Java `Skill.getConfusionMessage()` — the base class returns `"is confused"`, and only
/// specific skill subclasses override it (`WildAnimal` -> "roars in rage", bb2020/bb2025
/// `BoneHead`/`ReallyStupid` -> "is distracted"). The Rust `Skill::get_confusion_message()`
/// stub always returns the base-class default, so this local lookup (keyed off the resolved
/// `SkillId`, matching the same subclasses) is a directly traceable translation of the
/// override table.
fn confusion_message_for(skill_id: ffb_model::enums::SkillId) -> &'static str {
    use ffb_model::enums::SkillId;
    match skill_id {
        SkillId::WildAnimal => "roars in rage",
        SkillId::BoneHead | SkillId::ReallyStupid => "is distracted",
        _ => "is confused",
    }
}

/// 1:1 translation of `ConfusionRollMessage.java`.
pub struct ConfusionRollMessage;

impl ReportMessage for ConfusionRollMessage {
    type Report = ReportConfusionRoll;

    fn report_id(&self) -> ReportId {
        ReportId::CONFUSION_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let Some(confusion_skill) = report.get_confusion_skill() else {
            return;
        };
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let skill_id = SkillFactory::new().for_name(confusion_skill);

        let status = format!("{} Roll [ {} ]", confusion_skill, report.base.get_roll());
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        print_player(status_report, game, status_report.get_indent() + 1, false, player);

        let mut needed_roll: Option<String> = None;
        if report.base.is_successful() {
            status_report.println_indent(status_report.get_indent() + 1, " is able to act normally.");
            if !report.base.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.base.get_minimum_roll()));
            }
        } else {
            let confusion_message = skill_id.map(confusion_message_for).unwrap_or("is confused");
            status_report.println_indent(status_report.get_indent() + 1, &format!(" {confusion_message}."));
            if !report.base.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.base.get_minimum_roll()));
            }
        }

        if let Some(mut needed_roll) = needed_roll {
            if let Some(skill_id) = skill_id {
                let properties = skill_id.properties();
                if properties.contains(&NamedProperties::NEEDS_TO_ROLL_FOR_ACTION_BUT_KEEPS_TACKLEZONE) {
                    if report.base.get_minimum_roll() > 2 {
                        needed_roll.push_str(" (Player does not attack)");
                    } else {
                        needed_roll.push_str(" (Player does attack)");
                    }
                }
                if properties.contains(&NamedProperties::NEEDS_TO_ROLL_HIGH_TO_AVOID_CONFUSION) {
                    if report.base.get_minimum_roll() > 2 {
                        needed_roll.push_str(" (Really Stupid player without assistance)");
                    } else {
                        needed_roll.push_str(" (Really Stupid player gets help from team-mates)");
                    }
                }
            }
            needed_roll.push('.');
            status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        let mut game = Game::new(make_team("home"), make_team("away"), Rules::Bb2025);
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Confused".into();
        player.gender = PlayerGender::Male;
        game.team_home.players.push(player);
        game.acting_player.player_id = Some("p1".into());
        game
    }

    #[test]
    fn no_confusion_skill_renders_nothing() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportConfusionRoll::new(Some("p1".into()), true, 4, 2, false, None);
        ConfusionRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn successful_roll_acts_normally() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportConfusionRoll::new(Some("p1".into()), true, 4, 2, false, Some("Bone Head".into()));
        ConfusionRollMessage.render(&mut status_report, &game, &report);
        let normal = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" is able to act normally."));
        assert!(normal.is_some());
        let needed = status_report.rendered_runs.iter().find(|r| r.text_style == Some(TextStyle::NEEDED_ROLL));
        assert_eq!(needed.unwrap().text.as_deref(), Some("Succeeded on a roll of 2+."));
    }

    #[test]
    fn failed_bone_head_uses_distracted_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportConfusionRoll::new(Some("p1".into()), false, 1, 2, false, Some("Bone Head".into()));
        ConfusionRollMessage.render(&mut status_report, &game, &report);
        let msg = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" is distracted."));
        assert!(msg.is_some());
    }

    #[test]
    fn failed_wild_animal_uses_roars_in_rage_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportConfusionRoll::new(Some("p1".into()), false, 1, 2, false, Some("Wild Animal".into()));
        ConfusionRollMessage.render(&mut status_report, &game, &report);
        let msg = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some(" roars in rage."));
        assert!(msg.is_some());
    }

    #[test]
    fn really_stupid_appends_property_note() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportConfusionRoll::new(Some("p1".into()), false, 1, 3, false, Some("Really Stupid".into()));
        ConfusionRollMessage.render(&mut status_report, &game, &report);
        let needed = status_report.rendered_runs.iter().find(|r| r.text_style == Some(TextStyle::NEEDED_ROLL));
        assert!(needed.unwrap().text.as_deref().unwrap().contains("Really Stupid player without assistance"));
    }

    #[test]
    fn rerolled_skips_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportConfusionRoll::new(Some("p1".into()), false, 1, 2, true, Some("Bone Head".into()));
        ConfusionRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().all(|r| r.text_style != Some(TextStyle::NEEDED_ROLL)));
    }

    #[test]
    fn report_id_is_confusion_roll() {
        assert_eq!(ConfusionRollMessage.report_id(), ReportId::CONFUSION_ROLL);
    }
}
