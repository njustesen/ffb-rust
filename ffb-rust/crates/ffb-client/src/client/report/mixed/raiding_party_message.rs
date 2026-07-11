use crate::client::report::report_message_base::{map_to_local, print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_raiding_party::ReportRaidingParty;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `RaidingPartyMessage.java`.
pub struct RaidingPartyMessage;

impl ReportMessage for RaidingPartyMessage {
    type Report = ReportRaidingParty;

    fn report_id(&self) -> ReportId {
        ReportId::RAIDING_PARTY
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let player = report.get_player_id().and_then(|id| game.player(id));
        let other_player = report.get_other_player_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent();

        print_player(status_report, game, indent, false, player);
        status_report.print_indent_style(indent, TextStyle::NONE, " allows ");
        print_player(status_report, game, indent, false, other_player);
        status_report.print_indent_style(indent, TextStyle::NONE, " to move one square ");
        let direction_name = report.get_direction().map(map_to_local).map(|d| d.name()).unwrap_or_default();
        status_report.print_indent_style(indent, TextStyle::NONE, direction_name);
        status_report.println_indent_style(indent, TextStyle::NONE, ".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Direction, PlayerGender, PlayerType, Rules};
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
    fn renders_both_players_and_direction() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportRaidingParty::new(Some("p1".into()), Some("p2".into()), Some(Direction::North));
        RaidingPartyMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Joe")));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Jane")));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("North")));
    }

    #[test]
    fn ends_with_period() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportRaidingParty::new(Some("p1".into()), Some("p2".into()), Some(Direction::Southeast));
        RaidingPartyMessage.render(&mut status_report, &game, &report);
        let last_text = status_report.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last_text.text.as_deref(), Some("."));
    }

    #[test]
    fn missing_direction_renders_empty_name() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportRaidingParty::new(Some("p1".into()), Some("p2".into()), None);
        RaidingPartyMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("")));
    }
}
