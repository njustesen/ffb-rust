use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::bb2016::report_tentacles_shadowing_roll::ReportTentaclesShadowingRoll;
use ffb_model::report::report_id::ReportId;

pub struct TentaclesShadowingMessage;

impl ReportMessage for TentaclesShadowingMessage {
    type Report = ReportTentaclesShadowingRoll;

    fn report_id(&self) -> ReportId {
        ReportId::TENTACLES_SHADOWING_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let acting_player_id = game.acting_player.player_id.as_deref();
        let acting_player = acting_player_id.and_then(|id| game.player(id));
        let defender = game.player(report.get_defender_id());
        let mut needed_roll: Option<String> = None;

        let skill = SkillId::from_class_name(report.get_skill());
        let has_follow = skill.is_some_and(|s| s.properties().contains(&NamedProperties::CAN_FOLLOW_PLAYER_LEAVING_TACKLEZONES));
        let has_hold = skill.is_some_and(|s| s.properties().contains(&NamedProperties::CAN_HOLD_PLAYERS_LEAVING_TACKLEZONES));

        if !report.is_re_rolled() {
            if has_follow {
                print_player(status_report, game, indent, true, defender);
                status_report.print_indent_style(indent, TextStyle::BOLD, " tries to shadow ");
                print_player(status_report, game, indent, true, acting_player);
                status_report.println_indent_style(indent, TextStyle::BOLD, ":");
            }
            if has_hold {
                print_player(status_report, game, indent, true, defender);
                status_report.print_indent_style(indent, TextStyle::BOLD, " tries to hold ");
                print_player(status_report, game, indent, true, acting_player);
                let gender = defender.map(|p| p.gender).unwrap_or_default();
                let status = format!(" with {} tentacles:", gender.genitive());
                status_report.println_indent_style(indent, TextStyle::BOLD, &status);
            }
        }
        let mut rolled_total = 0;
        let roll = report.get_roll();
        if !roll.is_empty() {
            rolled_total = roll[0] + roll[1];
        }
        if has_follow {
            if rolled_total > 0 {
                let status = format!("Shadowing Escape Roll [ {} ][ {} ] = {}", roll[0], roll[1], rolled_total);
                status_report.println_indent_style(indent + 1, TextStyle::ROLL, &status);
            }
            if report.is_successful() {
                print_player(status_report, game, indent + 2, false, acting_player);
                let gender = acting_player.map(|p| p.gender).unwrap_or_default();
                let status = format!(" escapes {} opponent.", gender.genitive());
                status_report.println_indent(indent + 2, &status);
                if !report.is_re_rolled() {
                    needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
                }
            } else {
                print_player(status_report, game, indent + 2, false, defender);
                let gender = defender.map(|p| p.gender).unwrap_or_default();
                let status = format!(" shadows {} opponent successfully.", gender.genitive());
                status_report.println_indent(indent + 2, &status);
                if !report.is_re_rolled() {
                    needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
                }
            }
            if let Some(mut needed_roll) = needed_roll.take() {
                if let (Some(acting_player), Some(defender)) = (acting_player, defender) {
                    needed_roll.push_str(&format!(
                        " (MA {} - MA {} + Roll > 7).",
                        acting_player.movement_with_modifiers(),
                        defender.movement_with_modifiers()
                    ));
                }
                status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &needed_roll);
            }
        }
        if has_hold {
            if rolled_total > 0 {
                let status = format!("Tentacles Escape Roll [ {} ][ {} ] = {}", roll[0], roll[1], rolled_total);
                status_report.println_indent_style(indent + 1, TextStyle::ROLL, &status);
            }
            let mut needed_roll: Option<String> = None;
            if report.is_successful() {
                print_player(status_report, game, indent + 2, false, acting_player);
                let gender = acting_player.map(|p| p.gender).unwrap_or_default();
                let status = format!(" escapes {} opponent.", gender.genitive());
                status_report.println_indent(indent + 2, &status);
                if !report.is_re_rolled() {
                    needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
                }
            } else {
                print_player(status_report, game, indent + 2, false, defender);
                let gender = defender.map(|p| p.gender).unwrap_or_default();
                let status = format!(" holds {} opponent successfully.", gender.genitive());
                status_report.println_indent(indent + 2, &status);
                if !report.is_re_rolled() {
                    needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
                }
            }
            if let Some(mut needed_roll) = needed_roll {
                if let Some(defender) = defender {
                    needed_roll.push_str(&format!(
                        " (ST {} - ST {} + Roll > 5).",
                        game.acting_player.strength,
                        defender.strength_with_modifiers()
                    ));
                }
                status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &needed_roll);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        let mut home = make_team("home");
        home.players.push(Player {
            id: "runner".into(), name: "Runner".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 7, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        let mut away = make_team("away");
        away.players.push(Player {
            id: "shadower".into(), name: "Shadower".into(), nr: 2, position_id: "assassin".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Female,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        let mut game = Game::new(home, away, Rules::Bb2016);
        game.acting_player.player_id = Some("runner".into());
        game.acting_player.strength = 3;
        game
    }

    #[test]
    fn get_key_is_tentacles_shadowing_roll() {
        assert_eq!(TentaclesShadowingMessage.get_key(), "tentaclesShadowingRoll");
    }

    #[test]
    fn shadowing_skill_successful_escape() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTentaclesShadowingRoll::new("Shadowing".into(), "shadower".into(), vec![4, 5], true, 4, false);
        TentaclesShadowingMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" escapes his opponent.")));
    }

    #[test]
    fn tentacles_skill_failed_hold() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTentaclesShadowingRoll::new("Tentacles".into(), "shadower".into(), vec![1, 1], false, 5, false);
        TentaclesShadowingMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" holds her opponent successfully.")));
    }

    #[test]
    fn re_rolled_skips_intro_line() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTentaclesShadowingRoll::new("Shadowing".into(), "shadower".into(), vec![4, 5], true, 4, true);
        TentaclesShadowingMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Shadowing Escape Roll [ 4 ][ 5 ] = 9"));
    }
}
