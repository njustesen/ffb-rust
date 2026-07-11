use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_nerves_of_steel::ReportNervesOfSteel;
use ffb_model::report::report_id::ReportId;

pub struct NervesOfSteelMessage;

impl ReportMessage for NervesOfSteelMessage {
    type Report = ReportNervesOfSteel;

    fn report_id(&self) -> ReportId {
        ReportId::NERVES_OF_STEEL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = game.player(report.get_player_id());
        if let Some(player) = player {
            print_player(status_report, game, indent, false, Some(player));
            status_report.println_indent(indent, &format!(" is using Nerves of Steel to {} the ball.", report.get_ball_action()));
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
            id: "p1".into(), name: "Grubb".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        Game::new(home, make_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_nerves_of_steel() {
        assert_eq!(NervesOfSteelMessage.get_key(), "nervesOfSteel");
    }

    #[test]
    fn known_player_reports_ball_action() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportNervesOfSteel::new("p1".into(), "pass".into());
        NervesOfSteelMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" is using Nerves of Steel to pass the ball.")));
    }

    #[test]
    fn unknown_player_produces_no_output() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportNervesOfSteel::new("missing".into(), "catch".into());
        NervesOfSteelMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.is_empty());
    }

    #[test]
    fn catch_action_text() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportNervesOfSteel::new("p1".into(), "catch".into());
        NervesOfSteelMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" is using Nerves of Steel to catch the ball.")));
    }
}
