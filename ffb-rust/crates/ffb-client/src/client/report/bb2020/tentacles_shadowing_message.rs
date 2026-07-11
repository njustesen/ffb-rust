use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::mixed::report_tentacles_shadowing_roll::ReportTentaclesShadowingRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `TentaclesShadowingMessage.java`.
pub struct TentaclesShadowingMessage;

impl ReportMessage for TentaclesShadowingMessage {
    type Report = ReportTentaclesShadowingRoll;

    fn report_id(&self) -> ReportId {
        ReportId::TENTACLES_SHADOWING_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        // java: `ActingPlayer actingPlayer = game.getActingPlayer();` — the acting player is
        // read via `game.acting_player.player_id`, resolved through `Game::player`.
        let acting_player: Option<&Player> = game
            .acting_player
            .player_id
            .as_deref()
            .and_then(|id| game.player(id));
        let defender: Option<&Player> = report.get_defender_id().and_then(|id| game.player(id));

        let Some(skill) = report.get_skill() else { return };
        let properties = skill.properties();
        let can_follow = properties.contains(&NamedProperties::CAN_FOLLOW_PLAYER_LEAVING_TACKLEZONES);
        let can_hold = properties.contains(&NamedProperties::CAN_HOLD_PLAYERS_LEAVING_TACKLEZONES);

        if !report.is_re_rolled() {
            if can_follow {
                print_player(status_report, game, indent, true, defender);
                status_report.print_indent_style(indent, TextStyle::BOLD, " tries to shadow ");
                print_player(status_report, game, indent, true, acting_player);
                status_report.println_indent_style(indent, TextStyle::BOLD, ":");
            }
            if can_hold {
                if let Some(defender) = defender {
                    print_player(status_report, game, indent, true, Some(defender));
                    status_report.print_indent_style(indent, TextStyle::BOLD, " tries to hold ");
                    print_player(status_report, game, indent, true, acting_player);
                    let status = format!(" with {} tentacles:", defender.gender.genitive());
                    status_report.println_indent_style(indent, TextStyle::BOLD, &status);
                }
            }
        }

        if can_follow {
            if let (Some(defender), Some(acting_player)) = (defender, acting_player) {
                status_report.println_indent_style(indent + 1, TextStyle::ROLL, &format!("Shadowing Roll [ {} ]", report.get_roll()));
                let mut needed_roll: Option<String> = None;
                if report.is_successful() {
                    print_player(status_report, game, indent + 2, false, Some(defender));
                    status_report.println_indent(
                        indent + 2,
                        &format!(" shadows {} opponent successfully.", defender.gender.genitive()),
                    );
                    if !report.is_re_rolled() {
                        needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
                    }
                } else {
                    print_player(status_report, game, indent + 2, false, Some(defender));
                    status_report.println_indent(
                        indent + 2,
                        &format!(" fails to shadow {} opponent.", defender.gender.genitive()),
                    );
                    if !report.is_re_rolled() {
                        needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
                    }
                }
                if let Some(mut needed_roll) = needed_roll {
                    needed_roll.push_str(&format!(
                        " (Roll + MA {} - MA {} >= 6).",
                        defender.movement_with_modifiers(),
                        acting_player.movement_with_modifiers()
                    ));
                    status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &needed_roll);
                }
            }
        }

        if can_hold {
            if let (Some(defender), Some(acting_player)) = (defender, acting_player) {
                status_report.println_indent_style(indent + 1, TextStyle::ROLL, &format!("Tentacles Roll [ {} ]", report.get_roll()));
                let mut needed_roll: Option<String> = None;
                if report.is_successful() {
                    print_player(status_report, game, indent + 2, false, Some(defender));
                    status_report.println_indent(
                        indent + 2,
                        &format!(" holds {} opponent successfully.", defender.gender.genitive()),
                    );
                    if !report.is_re_rolled() {
                        needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
                    }
                } else {
                    print_player(status_report, game, indent + 2, false, Some(defender));
                    status_report.println_indent(
                        indent + 2,
                        &format!(" failed to hold {} opponent.", defender.gender.genitive()),
                    );
                    if !report.is_re_rolled() {
                        needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
                    }
                }
                if let Some(mut needed_roll) = needed_roll {
                    needed_roll.push_str(&format!(
                        " (Roll + ST {} - ST {} >= 6).",
                        defender.strength_with_modifiers(),
                        acting_player.strength_with_modifiers()
                    ));
                    status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &needed_roll);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, gender: PlayerGender, movement: i32, strength: i32) -> Player {
        Player {
            id: id.into(),
            name: format!("Player {id}"),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender,
            movement,
            strength,
            agility: 3,
            passing: 4,
            armour: 8,
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: "Team".into(),
            race: String::new(),
            roster_id: String::new(),
            coach: String::new(),
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

    fn make_game() -> Game {
        let mut game = Game::new(
            make_team("home", vec![make_player("acting", PlayerGender::Male, 7, 3)]),
            make_team("away", vec![make_player("defender", PlayerGender::Female, 6, 4)]),
            Rules::Bb2020,
        );
        game.acting_player.player_id = Some("acting".into());
        game
    }

    fn texts(status_report: &StatusReport) -> Vec<&str> {
        status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect()
    }

    #[test]
    fn shadowing_skill_success() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportTentaclesShadowingRoll::new(Some(SkillId::Shadowing), Some("defender".into()), 6, true, 4, false);
        TentaclesShadowingMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.iter().any(|s| s.contains("tries to shadow")));
        assert!(t.iter().any(|s| s.contains("Shadowing Roll [ 6 ]")));
        assert!(t.iter().any(|s| s.contains("shadows her opponent successfully.")));
        assert!(t.iter().any(|s| s.contains("Succeeded on a roll of 4+")));
        assert!(t.iter().any(|s| s.contains("MA 6 - MA 7 >= 6")));
    }

    #[test]
    fn shadowing_skill_failure() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportTentaclesShadowingRoll::new(Some(SkillId::Shadowing), Some("defender".into()), 2, false, 4, false);
        TentaclesShadowingMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.iter().any(|s| s.contains("fails to shadow her opponent.")));
        assert!(t.iter().any(|s| s.contains("Roll a 4+ to succeed")));
    }

    #[test]
    fn tentacles_skill_success() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportTentaclesShadowingRoll::new(Some(SkillId::Tentacles), Some("defender".into()), 6, true, 4, false);
        TentaclesShadowingMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(t.iter().any(|s| s.contains("tries to hold")));
        assert!(t.iter().any(|s| s.contains("with her tentacles:")));
        assert!(t.iter().any(|s| s.contains("Tentacles Roll [ 6 ]")));
        assert!(t.iter().any(|s| s.contains("holds her opponent successfully.")));
        assert!(t.iter().any(|s| s.contains("ST 4 - ST 3 >= 6")));
    }

    #[test]
    fn re_rolled_skips_intro_and_needed_roll() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportTentaclesShadowingRoll::new(Some(SkillId::Shadowing), Some("defender".into()), 6, true, 4, true);
        TentaclesShadowingMessage.render(&mut status_report, &game, &report);
        let t = texts(&status_report);
        assert!(!t.iter().any(|s| s.contains("tries to shadow")));
        assert!(!t.iter().any(|s| s.contains("Succeeded on a roll")));
        assert!(t.iter().any(|s| s.contains("Shadowing Roll [ 6 ]")));
    }
}
