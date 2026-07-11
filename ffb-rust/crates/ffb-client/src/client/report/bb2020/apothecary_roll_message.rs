use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PlayerState;
use ffb_model::factory::serious_injury_factory::SeriousInjuryFactory;
use ffb_model::model::game::Game;
use ffb_model::model::serious_injury::SeriousInjury as ISeriousInjury;
use ffb_model::report::mixed::report_apothecary_roll::ReportApothecaryRoll;
use ffb_model::report::report_id::ReportId;

/// java: `PlayerState.getDescription()` — not yet exposed on the Rust `PlayerState` type.
fn player_state_description(state: PlayerState) -> &'static str {
    use ffb_model::enums::*;
    match state.base() {
        PS_UNKNOWN => "is unknown",
        PS_STANDING => "is standing",
        PS_MOVING => "is moving",
        PS_PRONE => "is prone",
        PS_STUNNED => "has been stunned",
        PS_KNOCKED_OUT => "has been knocked out",
        PS_BADLY_HURT => "has been badly hurt",
        PS_SERIOUS_INJURY => "has been seriously injured",
        PS_RIP => "has been killed",
        PS_RESERVE => "is in reserve",
        PS_MISSING => "is missing the game",
        PS_FALLING => "is about to fall down",
        PS_BLOCKED => "is being blocked",
        PS_BANNED => "is banned from the game",
        PS_EXHAUSTED => "is exhausted",
        PS_BEING_DRAGGED => "is being dragged",
        PS_PICKED_UP => "has been picked up",
        PS_HIT_ON_GROUND => "was hit while on the ground",
        PS_SETUP_PREVENTED => "can not be set up",
        PS_IN_THE_AIR => "is in the air",
        _ => "",
    }
}

/// java: `CasualtyModifier.getModifier()` has no reachable equivalent here — the report
/// model (`ReportApothecaryRoll.casualty_modifiers`) already flattens each
/// `CasualtyModifier` down to its `reportString()` (format `"<modifier> <name>"`, see
/// `ffb_mechanics::modifiers::bb2020::casualty_modifier::CasualtyModifier::report_string`),
/// discarding the standalone numeric modifier needed to sum `Rolled X + ... = Y`. The
/// leading numeric token is re-parsed out of each already-formatted string as the closest
/// available approximation.
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
        let casualty_roll = report.get_casualty_roll();
        if casualty_roll.is_empty() {
            return;
        }
        let indent = status_report.get_indent();
        status_report.println_indent_style(indent, TextStyle::BOLD, "Apothecary used.");
        let player = report.get_player_id().and_then(|id| game.player(id));

        let mut si_factory = SeriousInjuryFactory::new();
        si_factory.initialize(game);
        let serious_injury = report.get_serious_injury().and_then(|name| si_factory.for_name(name));

        let mut status = format!("Casualty Roll [ {}", casualty_roll[0]);
        if serious_injury.is_some_and(|si| si.show_si_roll()) && casualty_roll.len() > 1 {
            status.push_str(&format!(" ][ {}", casualty_roll[1]));
        }
        status.push_str(" ]");
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);

        if !report.get_casualty_modifiers().is_empty() {
            let mut modifiers = 0;
            let mut report_strings: Vec<&String> = Vec::new();
            for modifier in report.get_casualty_modifiers() {
                report_strings.push(modifier);
                modifiers += parse_leading_modifier(modifier);
            }
            report_strings.sort();
            let mut status = format!("Rolled {}", casualty_roll[0]);
            for report_string in &report_strings {
                status.push_str(" + ");
                status.push_str(report_string);
            }
            status.push_str(&format!(" = {}", casualty_roll[0] + modifiers));
            status_report.println_indent_style(indent + 1, TextStyle::NONE, &status);
        }

        if let Some(injury) = report.get_player_state() {
            print_player(status_report, game, indent + 1, false, player);
            let status = format!(" {}.", player_state_description(injury));
            status_report.println_indent(indent + 1, &status);
        }

        if let Some(serious_injury) = serious_injury {
            if let Some(original_injury) = report.get_original_injury().and_then(|name| si_factory.for_name(name)) {
                let player_name = player.map(|p| p.name.as_str()).unwrap_or("");
                let status = format!(
                    "{player_name} would have {} but that stat cannot be reduced any further. So a different injury has been chosen randomly.",
                    original_injury.get_description()
                );
                status_report.println_indent_style(indent + 1, TextStyle::EXPLANATION, &status);
            }
            print_player(status_report, game, indent + 1, false, player);
            let status = format!(" {}.", serious_injury.get_description());
            status_report.println_indent(indent + 1, &status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules, PS_KNOCKED_OUT};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(),
            name: format!("Player {id}"),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
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
        Game::new(
            make_team("home", vec![make_player("p1")]),
            make_team("away", vec![]),
            Rules::Bb2020,
        )
    }

    #[test]
    fn no_casualty_roll_renders_nothing() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportApothecaryRoll::new(Some("p1".into()), vec![], None, None, None, vec![]);
        ApothecaryRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn casualty_roll_without_serious_injury_prints_state_description() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportApothecaryRoll::new(
            Some("p1".into()),
            vec![3],
            Some(PlayerState::new(PS_KNOCKED_OUT)),
            None,
            None,
            vec![],
        );
        ApothecaryRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("Apothecary used.")));
        assert!(texts.iter().any(|t| t.contains("Casualty Roll [ 3 ]")));
        assert!(texts.iter().any(|t| t.contains("has been knocked out.")));
    }

    #[test]
    fn serious_injury_present_prints_description_and_shows_si_roll() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportApothecaryRoll::new(
            Some("p1".into()),
            vec![3, 5],
            Some(PlayerState::new(PS_KNOCKED_OUT)),
            Some("Head Injury (-AV)".into()),
            None,
            vec![],
        );
        ApothecaryRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        // Head Injury shows the SI roll, so both dice appear.
        assert!(texts.iter().any(|t| t.contains("Casualty Roll [ 3 ][ 5 ]")));
        assert!(texts.iter().any(|t| t.contains("suffered a head injury")));
    }

    #[test]
    fn original_injury_present_prints_explanation() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportApothecaryRoll::new(
            Some("p1".into()),
            vec![3, 5],
            Some(PlayerState::new(PS_KNOCKED_OUT)),
            Some("Head Injury (-AV)".into()),
            Some("Neck Injury (-AG)".into()),
            vec![],
        );
        ApothecaryRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("would have suffered a neck injury")));
        assert!(texts.iter().any(|t| t.contains("but that stat cannot be reduced any further")));
    }

    #[test]
    fn casualty_modifiers_present_prints_sum_line() {
        let game = make_game();
        let mut status_report = StatusReport::new();
        let report = ReportApothecaryRoll::new(
            Some("p1".into()),
            vec![3],
            Some(PlayerState::new(PS_KNOCKED_OUT)),
            None,
            None,
            vec!["1 Mighty Blow".into(), "-1 Thick Skull".into()],
        );
        ApothecaryRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.as_deref()).collect();
        assert!(texts.iter().any(|t| t.contains("Rolled 3")));
        assert!(texts.iter().any(|t| t.contains("-1 Thick Skull")));
        assert!(texts.iter().any(|t| t.contains("1 Mighty Blow")));
        // 3 + 1 + (-1) = 3
        assert!(texts.iter().any(|t| t.contains(" = 3")));
    }
}
