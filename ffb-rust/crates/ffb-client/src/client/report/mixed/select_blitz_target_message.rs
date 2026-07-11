use crate::client::report::report_message_base::ReportMessage;
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::report::mixed::report_select_blitz_target::ReportSelectBlitzTarget;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `SelectBlitzTargetMessage.java`.
pub struct SelectBlitzTargetMessage;

impl ReportMessage for SelectBlitzTargetMessage {
    type Report = ReportSelectBlitzTarget;

    fn report_id(&self) -> ReportId {
        ReportId::SELECT_BLITZ_TARGET
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let attacker = report.get_attacker().and_then(|id| game.player(id));
        let defender = report.get_defender().and_then(|id| game.player(id));

        if let Some(attacker) = attacker {
            status_report.print_indent_style(indent, team_style_for_player(game, attacker), &attacker.name);
        }
        status_report.print_indent_style(indent, TextStyle::NONE, " targets ");
        if let Some(defender) = defender {
            status_report.print_indent_style(indent, team_style_for_player(game, defender), &defender.name);
        }
        status_report.println_indent_style(indent, TextStyle::NONE, ".");
    }
}

/// Java: private `teamStyleForPlayer(Player<?>)`.
fn team_style_for_player(game: &Game, player: &Player) -> TextStyle {
    if game.team_home.has_player(&player.id) {
        TextStyle::HOME
    } else {
        TextStyle::AWAY
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
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
    fn home_attacker_targets_away_defender() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSelectBlitzTarget::new(Some("p1".into()), Some("p2".into()));
        SelectBlitzTargetMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert_eq!(texts, vec!["Joe", " targets ", "Jane", "."]);
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::HOME));
    }

    #[test]
    fn away_attacker_targets_home_defender() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSelectBlitzTarget::new(Some("p2".into()), Some("p1".into()));
        SelectBlitzTargetMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text_style, Some(TextStyle::AWAY));
    }

    #[test]
    fn defender_style_resolved_independently() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportSelectBlitzTarget::new(Some("p1".into()), Some("p2".into()));
        SelectBlitzTargetMessage.render(&mut status_report, &game, &report);
        // Jane is at index 2 in the run list ("Joe", " targets ", "Jane", ".")
        let jane_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Jane")).unwrap();
        assert_eq!(jane_run.text_style, Some(TextStyle::AWAY));
    }
}
