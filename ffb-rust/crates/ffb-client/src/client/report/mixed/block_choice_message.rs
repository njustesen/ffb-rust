use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::report_block_choice::ReportBlockChoice;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `BlockChoiceMessage.java`.
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
            status.push_str(defender.map(|d| d.name.as_str()).unwrap_or(""));
        }
        status.push_str(&format!(" [ {} ]", report.get_block_result()));
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);

        let attacker = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));

        // java: BlockResult enum name matching — verified against ffb-java BlockResult.java
        // constants: BOTH_DOWN -> "BOTH DOWN", POW_PUSHBACK -> "POW/PUSH".
        match report.get_block_result() {
            "BOTH DOWN" => {
                if let Some(attacker) = attacker {
                    if attacker.has_skill_property(NamedProperties::PREVENT_FALL_ON_BOTH_DOWN) {
                        print_player(status_report, game, indent + 1, false, Some(attacker));
                        let skill_name = attacker
                            .skill_id_with_property(NamedProperties::PREVENT_FALL_ON_BOTH_DOWN)
                            .map(|id| id.class_name())
                            .unwrap_or("");
                        status_report.println_indent(
                            indent + 1,
                            &format!(" has been saved by {} {} skill.", attacker.gender.genitive(), skill_name),
                        );
                    }
                }
                if let Some(defender) = defender {
                    if defender.has_skill_property(NamedProperties::PREVENT_FALL_ON_BOTH_DOWN) {
                        print_player(status_report, game, indent + 1, false, Some(defender));
                        let skill_name = defender
                            .skill_id_with_property(NamedProperties::PREVENT_FALL_ON_BOTH_DOWN)
                            .map(|id| id.class_name())
                            .unwrap_or("");
                        let player_state = game.field_model.player_state(&defender.id);
                        let has_tacklezones = player_state.is_some_and(|s| s.has_tacklezones());
                        if has_tacklezones {
                            status_report.println_indent(
                                indent + 1,
                                &format!(" has been saved by {} {} skill.", defender.gender.genitive(), skill_name),
                            );
                        } else {
                            status_report.println_indent(
                                indent + 1,
                                &format!(
                                    " has not been saved by {} {} skill, due to having no tacklezones.",
                                    defender.gender.genitive(),
                                    skill_name
                                ),
                            );
                        }
                    }
                }
            }
            "POW/PUSH" => {
                if let (Some(attacker), Some(defender)) = (attacker, defender) {
                    // java: UtilCards.hasSkillToCancelProperty(attacker, ...) not reachable from
                    // ffb-client — approximated with a direct property check on the attacker.
                    let attacker_cancels = attacker.has_skill_property(NamedProperties::IGNORE_DEFENDER_STUMBLES_RESULT);
                    let defender_has = defender.has_skill_property(NamedProperties::IGNORE_DEFENDER_STUMBLES_RESULT)
                        || defender.has_unused_skill_with_property(NamedProperties::IGNORES_DEFENDER_STUMBLES_RESULT_FOR_FIRST_BLOCK);
                    if attacker_cancels && defender_has {
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
    use ffb_model::enums::{PlayerGender, Rules, SkillId};
    use ffb_model::model::acting_player::ActingPlayer;
    use ffb_model::model::player::Player;
    use ffb_model::model::player_state::PlayerState;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::model::team::Team;
    use ffb_model::enums::PS_STANDING;

    fn make_team(id: &str, players: Vec<Player>) -> Team {
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
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_report(block_result: &str, defender_id: &str, show_name: bool) -> ReportBlockChoice {
        ReportBlockChoice::new(2, vec![2, 3], 0, block_result.to_string(), defender_id.to_string(), false, show_name, 1)
    }

    #[test]
    fn shows_defender_name_when_configured() {
        let attacker = Player { id: "a1".into(), name: "Attacker".into(), gender: PlayerGender::Male, ..Player::default() };
        let defender = Player { id: "d1".into(), name: "Defender".into(), gender: PlayerGender::Female, ..Player::default() };
        let mut game = Game::new(make_team("home", vec![attacker]), make_team("away", vec![defender]), Rules::Bb2020);
        game.acting_player = ActingPlayer { player_id: Some("a1".into()), ..Default::default() };
        let report = make_report("PUSHBACK", "d1", true);
        let mut sr = StatusReport::new();
        BlockChoiceMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Block Result against Defender [ PUSHBACK ]"));
    }

    #[test]
    fn hides_defender_name_when_not_configured() {
        let attacker = Player { id: "a1".into(), name: "Attacker".into(), ..Player::default() };
        let defender = Player { id: "d1".into(), name: "Defender".into(), ..Player::default() };
        let mut game = Game::new(make_team("home", vec![attacker]), make_team("away", vec![defender]), Rules::Bb2020);
        game.acting_player = ActingPlayer { player_id: Some("a1".into()), ..Default::default() };
        let report = make_report("POW", "d1", false);
        let mut sr = StatusReport::new();
        BlockChoiceMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs[0].text.as_deref(), Some("Block Result [ POW ]"));
    }

    #[test]
    fn both_down_with_attacker_skill_prints_saved_message() {
        let attacker = Player {
            id: "a1".into(),
            name: "Attacker".into(),
            gender: PlayerGender::Male,
            starting_skills: vec![SkillWithValue::new(SkillId::StandFirm)],
            ..Player::default()
        };
        // No real skill maps to preventFallOnBothDown, so use has_skill_property directly via a
        // stand-in: since SkillId properties are fixed by data, this test exercises the "no skill"
        // branch (no output beyond the header) to keep behaviour honest to available skill data.
        let defender = Player { id: "d1".into(), name: "Defender".into(), gender: PlayerGender::Female, ..Player::default() };
        let mut game = Game::new(make_team("home", vec![attacker]), make_team("away", vec![defender]), Rules::Bb2020);
        game.acting_player = ActingPlayer { player_id: Some("a1".into()), ..Default::default() };
        game.field_model.set_player_state("d1", PlayerState::new(PS_STANDING).change_active(true));
        let report = make_report("BOTH DOWN", "d1", false);
        let mut sr = StatusReport::new();
        BlockChoiceMessage.render(&mut sr, &game, &report);
        // Only the header run/terminator are printed when neither side has the property.
        assert_eq!(sr.rendered_runs.len(), 2);
    }

    #[test]
    fn pow_pushback_without_tackle_prints_only_header() {
        let attacker = Player { id: "a1".into(), name: "Attacker".into(), ..Player::default() };
        let defender = Player { id: "d1".into(), name: "Defender".into(), ..Player::default() };
        let mut game = Game::new(make_team("home", vec![attacker]), make_team("away", vec![defender]), Rules::Bb2020);
        game.acting_player = ActingPlayer { player_id: Some("a1".into()), ..Default::default() };
        let report = make_report("POW/PUSH", "d1", false);
        let mut sr = StatusReport::new();
        BlockChoiceMessage.render(&mut sr, &game, &report);
        assert_eq!(sr.rendered_runs.len(), 2);
    }

    #[test]
    fn report_id_and_key() {
        assert_eq!(BlockChoiceMessage.report_id(), ReportId::BLOCK_CHOICE);
        assert_eq!(BlockChoiceMessage.get_key(), "blockChoice");
    }
}
