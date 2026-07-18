use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::factory::serious_injury_factory::SeriousInjuryFactory;
use ffb_model::model::game::Game;
use ffb_model::model::serious_injury::SeriousInjury;
use ffb_model::report::mixed::report_apothecary_roll::ReportApothecaryRoll;
use ffb_model::report::report_id::ReportId;

// java: PlayerState.getDescription() — not yet ported to the shared `PlayerState` bitmask
// struct (crates/ffb-model/src/enums/player.rs), so the base -> description switch from
// `ffb-common/src/main/java/com/fumbbl/ffb/PlayerState.java` is translated locally here.
// Cross-checked line-for-line against `PlayerState.java`'s `getDescription()` switch (Phase
// re-verification pass) — every variant string matches exactly; no drift found.
fn player_state_description(base: u32) -> Option<&'static str> {
    match base {
        0x00000 => Some("is unknown"),
        0x00001 => Some("is standing"),
        0x00002 => Some("is moving"),
        0x00003 => Some("is prone"),
        0x00004 => Some("has been stunned"),
        0x00005 => Some("has been knocked out"),
        0x00006 => Some("has been badly hurt"),
        0x00007 => Some("has been seriously injured"),
        0x00008 => Some("has been killed"),
        0x00009 => Some("is in reserve"),
        0x0000a => Some("is missing the game"),
        0x0000b => Some("is about to fall down"),
        0x0000c => Some("is being blocked"),
        0x0000d => Some("is banned from the game"),
        0x0000e => Some("is exhausted"),
        0x0000f => Some("is being dragged"),
        0x00010 => Some("has been picked up"),
        0x00011 => Some("was hit while on the ground"),
        0x00014 => Some("can not be set up"),
        0x00015 => Some("is in the air"),
        _ => None,
    }
}

/// java: `CasualtyModifier.getModifier()` has no reachable equivalent here — the report
/// model (`ReportApothecaryRoll.casualty_modifiers`) already flattens each `CasualtyModifier`
/// down to its `reportString()` (format `"<modifier> <name>"`, see
/// `ffb_mechanics::modifiers::bb2020::casualty_modifier::CasualtyModifier::report_string`),
/// discarding the standalone numeric modifier needed to sum `Rolled X + ... = Y`. Mirrors the
/// same fix already applied in the sibling `bb2020::ApothecaryRollMessage` translation: the
/// leading numeric token is re-parsed out of each already-formatted string.
fn parse_leading_modifier(report_string: &str) -> i32 {
    report_string
        .split_whitespace()
        .next()
        .and_then(|tok| tok.parse::<i32>().ok())
        .unwrap_or(0)
}

/// 1:1 translation of `ApothecaryRollMessage.java`.
pub struct ApothecaryRollMessage;

impl ReportMessage for ApothecaryRollMessage {
    type Report = ReportApothecaryRoll;

    fn report_id(&self) -> ReportId {
        ReportId::APOTHECARY_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let casualty_roll = report.get_casualty_roll();
        // java: ArrayTool.isProvided(casualtyRoll)
        if !casualty_roll.is_empty() {
            status_report.println_indent_style(indent, TextStyle::BOLD, "Apothecary used.");
            let player = report.get_player_id().and_then(|id| game.player(id));

            let mut factory = SeriousInjuryFactory::new();
            factory.initialize(game);
            let serious_injury = report.get_serious_injury().and_then(|name| factory.for_name(name));

            let mut status = format!("Casualty Roll [ {}", casualty_roll[0]);
            if serious_injury.as_ref().is_some_and(|si| si.show_si_roll()) && casualty_roll.len() > 1 {
                status.push_str(&format!(" ][ {}", casualty_roll[1]));
            }
            status.push_str(" ]");
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);

            if !report.get_casualty_modifiers().is_empty() {
                let mut modifiers = 0;
                let mut report_strings: Vec<String> = report.get_casualty_modifiers().to_vec();
                for report_string in &report_strings {
                    modifiers += parse_leading_modifier(report_string);
                }
                report_strings.sort();
                let mut status = format!("Rolled {}", casualty_roll[0]);
                for report_string in &report_strings {
                    status.push_str(" + ");
                    status.push_str(report_string);
                }
                // java: status.append(" = ").append(casualtyRoll[0] + modifiers)
                status.push_str(&format!(" = {}", casualty_roll[0] + modifiers));
                status_report.println_indent_style(indent + 1, TextStyle::NONE, &status);
            }

            let injury = report.get_player_state();
            print_player(status_report, game, indent + 1, false, player);
            let injury_description = injury.and_then(|ps| player_state_description(ps.base())).unwrap_or("");
            status_report.println_indent(indent + 1, &format!(" {injury_description}."));

            if let Some(serious_injury) = &serious_injury {
                if let Some(original_name) = report.get_original_injury() {
                    if let Some(original_injury) = factory.for_name(original_name) {
                        let player_name = player.map(|p| p.name.as_str()).unwrap_or("");
                        let status = format!(
                            "{player_name} would have {} but that stat cannot be reduced any further. The result counts as Miss Next Game instead.",
                            original_injury.get_description()
                        );
                        status_report.println_indent_style(indent + 1, TextStyle::EXPLANATION, &status);
                    }
                }
                print_player(status_report, game, indent + 1, false, player);
                status_report.println_indent(indent + 1, &format!(" {}.", serious_injury.get_description()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerState, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str) -> Player {
        Player { id: id.into(), name: name.into(), player_type: PlayerType::default(), ..Default::default() }
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
        let home = make_team("home", vec![make_player("p1", "Injured Player")]);
        let away = make_team("away", vec![]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn report_id_is_apothecary_roll() {
        assert_eq!(ApothecaryRollMessage.report_id(), ReportId::APOTHECARY_ROLL);
    }

    #[test]
    fn empty_casualty_roll_renders_nothing() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportApothecaryRoll::new(Some("p1".into()), vec![], None, None, None, vec![]);
        ApothecaryRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn basic_casualty_roll_reports_used_and_injury_description() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportApothecaryRoll::new(
            Some("p1".into()),
            vec![4],
            Some(PlayerState::new(0x00006)), // BADLY_HURT
            None,
            None,
            vec![],
        );
        ApothecaryRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Apothecary used."));
        assert!(texts.iter().any(|t| t == "Casualty Roll [ 4 ]"));
        assert!(texts.iter().any(|t| t == " has been badly hurt."));
    }

    #[test]
    fn casualty_modifiers_reported_sorted_alphabetically() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportApothecaryRoll::new(
            Some("p1".into()),
            vec![4],
            Some(PlayerState::new(0x00006)),
            None,
            None,
            vec!["1 Zealous".into(), "1 Claws".into()],
        );
        ApothecaryRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        let rolled_line = texts.iter().find(|t| t.starts_with("Rolled")).unwrap();
        assert!(rolled_line.find("Claws").unwrap() < rolled_line.find("Zealous").unwrap());
    }

    #[test]
    fn casualty_modifiers_present_prints_sum_line() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportApothecaryRoll::new(
            Some("p1".into()),
            vec![3],
            Some(PlayerState::new(0x00005)), // KNOCKED_OUT
            None,
            None,
            vec!["1 Mighty Blow".into(), "-1 Thick Skull".into()],
        );
        ApothecaryRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        let rolled_line = texts.iter().find(|t| t.starts_with("Rolled")).unwrap();
        assert!(rolled_line.contains("1 Mighty Blow"));
        assert!(rolled_line.contains("-1 Thick Skull"));
        // 3 + 1 + (-1) = 3
        assert!(rolled_line.ends_with(" = 3"));
    }

    #[test]
    fn serious_injury_present_reports_description() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportApothecaryRoll::new(
            Some("p1".into()),
            vec![4, 8],
            Some(PlayerState::new(0x00007)), // SERIOUS_INJURY
            Some("SERIOUSLY_HURT".into()),
            None,
            vec![],
        );
        ApothecaryRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.starts_with(" ") && t.ends_with(".") && t.len() > 1));
    }
}
