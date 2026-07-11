use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
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
        let mut needed_roll: Option<String> = None;
        let acting_player_player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let defender = report.get_defender_id().and_then(|id| game.player(id));
        let skill = report.get_skill();
        let has_follow = skill.is_some_and(|s| s.properties().contains(&NamedProperties::CAN_FOLLOW_PLAYER_LEAVING_TACKLEZONES));
        let has_hold = skill.is_some_and(|s| s.properties().contains(&NamedProperties::CAN_HOLD_PLAYERS_LEAVING_TACKLEZONES));

        if !report.is_re_rolled() {
            if has_follow {
                print_player(status_report, game, indent, true, defender);
                status_report.print_indent_style(indent, TextStyle::BOLD, " tries to shadow ");
                print_player(status_report, game, indent, true, acting_player_player);
                status_report.println_indent_style(indent, TextStyle::BOLD, ":");
            }
            if has_hold {
                print_player(status_report, game, indent, true, defender);
                status_report.print_indent_style(indent, TextStyle::BOLD, " tries to hold ");
                print_player(status_report, game, indent, true, acting_player_player);
                let genitive = defender.map(|d| d.gender.genitive()).unwrap_or("");
                let status = format!(" with {genitive} tentacles:");
                status_report.println_indent_style(indent, TextStyle::BOLD, &status);
            }
        }

        if has_follow {
            status_report.println_indent_style(indent + 1, TextStyle::ROLL, &format!("Shadowing Roll [ {} ]", report.get_roll()));
            let genitive = defender.map(|d| d.gender.genitive()).unwrap_or("");
            if report.is_successful() {
                print_player(status_report, game, indent + 2, false, defender);
                status_report.println_indent(indent + 2, &format!(" shadows {genitive} opponent successfully."));
                if !report.is_re_rolled() {
                    needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
                }
            } else {
                print_player(status_report, game, indent + 2, false, defender);
                status_report.println_indent(indent + 2, &format!(" fails to shadow {genitive} opponent."));
                if !report.is_re_rolled() {
                    needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
                }
            }
            if let Some(needed_roll) = &needed_roll {
                status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, needed_roll);
            }
        }

        if has_hold {
            status_report.println_indent_style(indent + 1, TextStyle::ROLL, &format!("Tentacles Roll [ {} ]", report.get_roll()));
            let genitive = defender.map(|d| d.gender.genitive()).unwrap_or("");
            if report.is_successful() {
                print_player(status_report, game, indent + 2, false, defender);
                status_report.println_indent(indent + 2, &format!(" holds {genitive} opponent successfully."));
                if !report.is_re_rolled() {
                    needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
                }
            } else {
                print_player(status_report, game, indent + 2, false, defender);
                status_report.println_indent(indent + 2, &format!(" failed to hold {genitive} opponent."));
                if !report.is_re_rolled() {
                    needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
                }
            }
            if let Some(needed_roll) = &needed_roll {
                let defender_str = defender.map(|d| d.strength_with_modifiers()).unwrap_or(0);
                let acting_str = acting_player_player.map(|p| p.strength_with_modifiers()).unwrap_or(0);
                let full = format!("{needed_roll} (Roll + ST {defender_str} - ST {acting_str} >= 6).");
                status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &full);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, Rules, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str) -> Player {
        Player { id: id.into(), name: name.into(), player_type: PlayerType::default(), strength: 3, ..Default::default() }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players, vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![make_player("attacker", "Attacker")]);
        let away = make_team("away", vec![make_player("defender", "Defender")]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.acting_player.set_player("attacker".into(), ffb_model::enums::PlayerAction::Move);
        game
    }

    #[test]
    fn report_id_is_tentacles_shadowing_roll() {
        assert_eq!(TentaclesShadowingMessage.report_id(), ReportId::TENTACLES_SHADOWING_ROLL);
    }

    #[test]
    fn shadowing_success_reports_roll_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTentaclesShadowingRoll::new(Some(SkillId::Shadowing), Some("defender".into()), 4, true, 3, false);
        TentaclesShadowingMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Shadowing Roll [ 4 ]"));
        assert!(texts.iter().any(|t| t.contains("shadows his opponent successfully.")));
        assert!(texts.iter().any(|t| t == "Succeeded on a roll of 3+"));
    }

    #[test]
    fn shadowing_failure_reports_roll_a_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTentaclesShadowingRoll::new(Some(SkillId::Shadowing), Some("defender".into()), 1, false, 3, false);
        TentaclesShadowingMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("fails to shadow his opponent.")));
        assert!(texts.iter().any(|t| t == "Roll a 3+ to succeed"));
    }

    #[test]
    fn tentacles_success_reports_st_comparison() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTentaclesShadowingRoll::new(Some(SkillId::Tentacles), Some("defender".into()), 6, true, 4, false);
        TentaclesShadowingMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Tentacles Roll [ 6 ]"));
        assert!(texts.iter().any(|t| t.contains("holds his opponent successfully.")));
        assert!(texts.iter().any(|t| t.contains("(Roll + ST 3 - ST 3 >= 6).")));
    }

    #[test]
    fn re_rolled_skips_intro_lines_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportTentaclesShadowingRoll::new(Some(SkillId::Shadowing), Some("defender".into()), 4, true, 3, true);
        TentaclesShadowingMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(!texts.iter().any(|t| t.contains("tries to shadow")));
        assert!(!texts.iter().any(|t| t.contains("Succeeded on a roll of")));
    }
}
