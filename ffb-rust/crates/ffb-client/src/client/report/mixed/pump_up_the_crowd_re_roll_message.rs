use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_pump_up_the_crowd_re_roll::ReportPumpUpTheCrowdReRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `PumpUpTheCrowdReRollMessage.java`.
pub struct PumpUpTheCrowdReRollMessage;

impl ReportMessage for PumpUpTheCrowdReRollMessage {
    type Report = ReportPumpUpTheCrowdReRoll;

    fn report_id(&self) -> ReportId {
        ReportId::PUMP_UP_THE_CROWD_RE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = report.get_player_id().and_then(|id| game.player(id));
        print_player(status_report, game, indent, false, player);

        status_report.print_indent(indent, " Pumps Up The Crowd so ");
        // java: `player.getTeam()` — resolved here via team membership lookup instead of a
        // back-reference from Player to Team.
        let is_home = player.is_some_and(|p| game.team_home.has_player(&p.id));
        if is_home {
            status_report.print_indent_style(indent, TextStyle::HOME, &game.team_home.name.clone());
        } else {
            status_report.print_indent_style(indent, TextStyle::AWAY, &game.team_away.name.clone());
        }
        status_report.println_indent(indent, " gains a Re-Roll only available for this drive.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use std::collections::HashSet;

    fn make_team(id: &str, name: &str) -> Team {
        Team {
            id: id.into(),
            name: name.into(),
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
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_player(id: &str, name: &str) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            starting_skills: vec![],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_game() -> Game {
        let mut home = make_team("home", "Home Team");
        home.players.push(make_player("p1", "Joe"));
        let mut away = make_team("away", "Away Team");
        away.players.push(make_player("p2", "Jane"));
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn home_player_gains_reroll_for_home_team() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPumpUpTheCrowdReRoll::new(Some("p1".into()));
        PumpUpTheCrowdReRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Joe")));
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Home Team")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::HOME));
    }

    #[test]
    fn away_player_gains_reroll_for_away_team() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPumpUpTheCrowdReRoll::new(Some("p2".into()));
        PumpUpTheCrowdReRollMessage.render(&mut status_report, &game, &report);
        let team_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Away Team")).unwrap();
        assert_eq!(team_run.text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn trailing_message_is_printed() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPumpUpTheCrowdReRoll::new(Some("p1".into()));
        PumpUpTheCrowdReRollMessage.render(&mut status_report, &game, &report);
        let last_text = status_report.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last_text.text.as_deref(), Some(" gains a Re-Roll only available for this drive."));
    }
}
