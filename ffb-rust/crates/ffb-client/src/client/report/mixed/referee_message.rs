use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_referee::ReportReferee;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `RefereeMessage.java`.
pub struct RefereeMessage;

impl ReportMessage for RefereeMessage {
    type Report = ReportReferee;

    fn report_id(&self) -> ReportId {
        ReportId::REFEREE
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let attacker = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        if report.is_fouling_player_banned() {
            status_report.print_indent(indent, "The referee spots the foul ");
            if report.is_under_scrutiny() {
                status_report.print_indent(indent, "because the team is under scrutiny ");
            }
            status_report.print_indent(indent, "and bans ");
            crate::client::report::report_message_base::print_player(status_report, game, indent, false, attacker);
            status_report.println_indent(indent, " from the game.");
        } else {
            status_report.println_indent(indent, "The referee didn't spot the foul.");
        }
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
        let mut game = Game::new(home, make_team("away", "Away Team"), Rules::Bb2020);
        game.acting_player.player_id = Some("p1".into());
        game
    }

    #[test]
    fn banned_not_under_scrutiny() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportReferee::new(true, false);
        RefereeMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert_eq!(
            texts,
            vec!["The referee spots the foul ", "and bans ", "Joe", " from the game."]
        );
    }

    #[test]
    fn banned_under_scrutiny() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportReferee::new(true, true);
        RefereeMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert_eq!(
            texts,
            vec![
                "The referee spots the foul ",
                "because the team is under scrutiny ",
                "and bans ",
                "Joe",
                " from the game."
            ]
        );
    }

    #[test]
    fn not_banned() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportReferee::new(false, false);
        RefereeMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("The referee didn't spot the foul."));
    }
}
