use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_mechanics::agility_mechanic::AgilityMechanic as AgilityMechanicTrait;
use ffb_mechanics::bb2020::agility_mechanic::AgilityMechanic;
use ffb_mechanics::modifiers::RollModifier;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_hypnotic_gaze_roll::ReportHypnoticGazeRoll;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool;

/// 1:1 translation of `HypnoticGazeRollMessage.java`.
pub struct HypnoticGazeRollMessage;

impl ReportMessage for HypnoticGazeRollMessage {
    type Report = ReportHypnoticGazeRoll;

    fn report_id(&self) -> ReportId {
        ReportId::HYPNOTIC_GAZE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = game
            .acting_player
            .player_id
            .as_deref()
            .and_then(|id| game.player(id));

        if !report.is_re_rolled() {
            let defender = if string_tool::is_provided(report.get_defender_id()) {
                report.get_defender_id().and_then(|id| game.player(id))
            } else {
                game.defender_id.as_deref().and_then(|id| game.player(id))
            };
            print_player(status_report, game, indent, true, player);
            status_report.print_indent_style(indent, TextStyle::BOLD, " gazes upon ");
            print_player(status_report, game, indent, true, defender);
            status_report.println_indent_style(indent, TextStyle::BOLD, ":");
        }

        status_report.println_indent_style(indent + 1, TextStyle::ROLL, &format!("Hypnotic Gaze Roll [ {} ]", report.get_roll()));
        print_player(status_report, game, indent + 2, false, player);

        let mut needed_roll: Option<String> = None;
        if report.is_successful() {
            let genitive = player.map(|p| p.gender.genitive()).unwrap_or("");
            status_report.println_indent(indent + 2, &format!(" hypnotizes {genitive} victim."));
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            let genitive = player.map(|p| p.gender.genitive()).unwrap_or("");
            status_report.println_indent(indent + 2, &format!(" fails to affect {genitive} victim."));
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }

        if let (Some(mut needed_roll), Some(player)) = (needed_roll, player) {
            let mechanic = AgilityMechanic::new();
            // java: `report.getRollModifiers()` returns the original `RollModifier<?>[]`
            // (name, sign, magnitude, isModifierIncluded). The Rust report model
            // (`ReportSkillRoll.roll_modifier_names`) already flattens each modifier down to
            // its sorted name only, discarding sign/magnitude/isModifierIncluded. Each name is
            // rebuilt here as a zero-magnitude, already-included `RollModifier` so
            // `format_roll_modifiers` renders just the name (best available approximation).
            let roll_modifiers: Vec<RollModifier> = report
                .base
                .get_roll_modifiers()
                .iter()
                .map(|name| RollModifier::with_report(name.clone(), name.clone(), 0, true))
                .collect();
            needed_roll.push_str(&mechanic.format_hypnotic_gaze_result(&roll_modifiers, player));
            status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, gender: PlayerGender) -> Player {
        Player {
            id: id.into(),
            name: format!("Player {id}"),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender,
            movement: 6,
            strength: 3,
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
            make_team("home", vec![make_player("gazer", PlayerGender::Male)]),
            make_team("away", vec![make_player("victim", PlayerGender::Female)]),
            Rules::Bb2020,
        );
        game.acting_player.player_id = Some("gazer".into());
        game
    }

    #[test]
    fn first_roll_prints_gazes_upon_header() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportHypnoticGazeRoll::new(Some("gazer".into()), true, 4, 2, false, Some("victim".into()));
        HypnoticGazeRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains(" gazes upon ")));
        assert!(texts.contains(&"Player gazer"));
        assert!(texts.contains(&"Player victim"));
    }

    #[test]
    fn re_rolled_skips_header_and_needed_roll() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportHypnoticGazeRoll::new(Some("gazer".into()), true, 4, 2, true, Some("victim".into()));
        HypnoticGazeRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(!texts.iter().any(|t| t.contains(" gazes upon ")));
        assert!(!texts.iter().any(|t| t.contains("Succeeded on a roll of")));
    }

    #[test]
    fn successful_roll_prints_hypnotizes_with_genitive() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportHypnoticGazeRoll::new(Some("gazer".into()), true, 4, 2, false, Some("victim".into()));
        HypnoticGazeRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains(" hypnotizes his victim.")));
        assert!(texts.iter().any(|t| t.contains("Succeeded on a roll of 2+")));
    }

    #[test]
    fn unsuccessful_roll_prints_fails_to_affect() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportHypnoticGazeRoll::new(Some("gazer".into()), false, 1, 3, false, None);
        HypnoticGazeRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains(" fails to affect his victim.")));
        assert!(texts.iter().any(|t| t.contains("Roll a 3+ to succeed")));
    }

    #[test]
    fn no_defender_id_falls_back_to_game_defender() {
        let mut game = make_game();
        game.defender_id = Some("victim".into());
        let mut status_report = StatusReport::new();
        let report = ReportHypnoticGazeRoll::new(Some("gazer".into()), true, 4, 2, false, None);
        HypnoticGazeRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.contains(&"Player victim"));
    }
}
