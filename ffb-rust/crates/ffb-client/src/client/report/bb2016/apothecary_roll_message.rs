use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PlayerState;
use ffb_model::factory::serious_injury_factory::SeriousInjuryFactory;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_apothecary_roll::ReportApothecaryRoll;
use ffb_model::report::report_id::ReportId;

/// Java: `PlayerState.getDescription()`. Not exposed on the Rust `PlayerState` type, so
/// this small lookup is kept local to the message renderers that need it (also used by
/// `InjuryMessage`).
pub(super) fn player_state_description(state: PlayerState) -> &'static str {
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

/// Java: `SeriousInjury.getDescription()`, resolved via `game.getRules().getFactory(SERIOUS_INJURY)`.
/// The Rust report model stores only the resolved `SeriousInjury` name, so the factory is
/// looked up locally here (also used by `InjuryMessage`).
pub(super) fn serious_injury_description(game: &Game, name: &str) -> Option<String> {
    let mut factory = SeriousInjuryFactory::new();
    factory.initialize(game);
    factory.for_name(name).map(|si| {
        use ffb_model::model::serious_injury::SeriousInjury;
        si.get_description().to_string()
    })
}

pub struct ApothecaryRollMessage;

impl ReportMessage for ApothecaryRollMessage {
    type Report = ReportApothecaryRoll;

    fn report_id(&self) -> ReportId {
        ReportId::APOTHECARY_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let casualty_roll = report.get_casualty_roll();
        if !casualty_roll.is_empty() {
            let indent = status_report.get_indent();
            status_report.println_indent_style(indent, TextStyle::BOLD, "Apothecary used.");
            let player = game.player(report.get_player_id());
            let status = format!("Casualty Roll [ {} ][ {} ]", casualty_roll[0], casualty_roll[1]);
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
            if let Some(injury) = report.get_player_state() {
                print_player(status_report, game, indent + 1, false, player);
                status_report.println_indent(indent + 1, &format!(" {}.", player_state_description(injury)));
            }
            if let Some(serious_injury) = report.get_serious_injury() {
                if let Some(description) = serious_injury_description(game, serious_injury) {
                    print_player(status_report, game, indent + 1, false, player);
                    status_report.println_indent(indent + 1, &format!(" {}.", description));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules, PS_BADLY_HURT};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team { id: id.into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        let mut home = make_team("home");
        home.players.push(Player {
            id: "p1".into(), name: "Grubb".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        Game::new(home, make_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_apothecary_roll() {
        assert_eq!(ApothecaryRollMessage.get_key(), "apothecaryRoll");
    }

    #[test]
    fn no_output_when_casualty_roll_absent() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportApothecaryRoll::new("p1".into(), vec![], None, None);
        ApothecaryRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn reports_casualty_roll_and_player_state() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportApothecaryRoll::new("p1".into(), vec![3, 4], Some(PlayerState::new(PS_BADLY_HURT)), None);
        ApothecaryRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Apothecary used."));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("Casualty Roll [ 3 ][ 4 ]"));
        // player print + state description
        assert_eq!(status_report.rendered_runs[4].text.as_deref(), Some("Grubb"));
        assert_eq!(status_report.rendered_runs[5].text.as_deref(), Some(" has been badly hurt."));
    }
}
