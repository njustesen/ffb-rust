use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::report_block_choice::ReportBlockChoice;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::util_cards::UtilCards;

pub struct BlockChoiceMessage;

impl ReportMessage for BlockChoiceMessage {
    type Report = ReportBlockChoice;

    fn report_id(&self) -> ReportId {
        ReportId::BLOCK_CHOICE
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let defender = game.player(report.get_defender_id());
        let mut status = String::from("Block Result");
        if report.is_show_name_in_report() {
            status.push_str(" against ");
            if let Some(defender) = defender {
                status.push_str(&defender.name);
            }
        }
        status.push_str(" [ ");
        status.push_str(report.get_block_result());
        status.push_str(" ]");
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let attacker = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        match report.get_block_result() {
            "BOTH DOWN" => {
                if let Some(attacker) = attacker {
                    if attacker.has_skill_property(NamedProperties::PREVENT_FALL_ON_BOTH_DOWN) {
                        print_player(status_report, game, indent + 1, false, Some(attacker));
                        let skill = attacker
                            .skill_id_with_property(NamedProperties::PREVENT_FALL_ON_BOTH_DOWN)
                            .map(|id| id.class_name())
                            .unwrap_or("");
                        let status = format!(
                            " has been saved by {} {} skill.",
                            attacker.gender.genitive(),
                            skill
                        );
                        status_report.println_indent(indent + 1, &status);
                    }
                }
                if let Some(defender) = defender {
                    if defender.has_skill_property(NamedProperties::PREVENT_FALL_ON_BOTH_DOWN) {
                        print_player(status_report, game, indent + 1, false, Some(defender));
                        let skill = defender
                            .skill_id_with_property(NamedProperties::PREVENT_FALL_ON_BOTH_DOWN)
                            .map(|id| id.class_name())
                            .unwrap_or("");
                        let status = format!(
                            " has been saved by {} {} skill.",
                            defender.gender.genitive(),
                            skill
                        );
                        status_report.println_indent(indent + 1, &status);
                    }
                }
            }
            "POW/PUSH" => {
                if let (Some(defender), Some(attacker)) = (defender, attacker) {
                    if UtilCards::has_skill_with_property(defender, NamedProperties::IGNORE_DEFENDER_STUMBLES_RESULT)
                        && UtilCards::has_skill_to_cancel_property(attacker, NamedProperties::IGNORE_DEFENDER_STUMBLES_RESULT)
                    {
                        print_player(status_report, game, indent + 1, false, Some(attacker));
                        status_report.println_indent(indent + 1, " uses Tackle to bring opponent down.");
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false }
    }

    fn make_player(id: &str, skills: Vec<SkillId>) -> Player {
        Player {
            id: id.into(), name: format!("Player {id}"), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            ..Default::default()
        }
    }

    fn make_game() -> Game {
        let mut home = make_team("home");
        home.players.push(make_player("attacker", vec![]));
        let mut away = make_team("away");
        away.players.push(make_player("defender", vec![]));
        let mut game = Game::new(home, away, Rules::Bb2016);
        game.acting_player.player_id = Some("attacker".into());
        game
    }

    #[test]
    fn get_key_is_block_choice() {
        assert_eq!(BlockChoiceMessage.get_key(), "blockChoice");
    }

    #[test]
    fn reports_block_result_without_name() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportBlockChoice::new(1, vec![3], 0, "PUSHBACK".into(), "defender".into(), false, false, 1);
        BlockChoiceMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Block Result [ PUSHBACK ]"));
    }

    #[test]
    fn reports_block_result_with_defender_name() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportBlockChoice::new(1, vec![3], 0, "PUSHBACK".into(), "defender".into(), false, true, 1);
        BlockChoiceMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Block Result against Player defender [ PUSHBACK ]"));
    }

    #[test]
    fn both_down_with_prevent_fall_skill_reports_save() {
        let mut status_report = StatusReport::new();
        let mut home = make_team("home");
        home.players.push(make_player("attacker", vec![SkillId::Block]));
        let mut away = make_team("away");
        away.players.push(make_player("defender", vec![]));
        let mut game = Game::new(home, away, Rules::Bb2016);
        game.acting_player.player_id = Some("attacker".into());
        let report = ReportBlockChoice::new(1, vec![3], 0, "BOTH DOWN".into(), "defender".into(), false, false, 1);
        BlockChoiceMessage.render(&mut status_report, &game, &report);
        // Block has the preventFallOnBothDown property, so the attacker's save line is added;
        // the defender has no such skill, so no second save line follows.
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref().unwrap_or("").contains("has been saved by his Block skill.")));
        assert_eq!(status_report.rendered_runs.len(), 5);
    }
}
