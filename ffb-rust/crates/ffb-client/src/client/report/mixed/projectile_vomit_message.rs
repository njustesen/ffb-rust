use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::gender_self;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_projectile_vomit::ReportProjectileVomit;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `ProjectileVomitMessage.java`.
pub struct ProjectileVomitMessage;

impl ReportMessage for ProjectileVomitMessage {
    type Report = ReportProjectileVomit;

    fn report_id(&self) -> ReportId {
        ReportId::PROJECTILE_VOMIT
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));

        let status = format!("Projectile Vomit Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);

        print_player(status_report, game, indent + 1, false, player);
        if report.is_successful() {
            status_report.print_indent_style(indent + 1, TextStyle::NONE, " vomits on ");
            let defender = report.get_defender_id().and_then(|id| game.player(id));
            print_player(status_report, game, indent + 1, false, defender);
            status_report.println_indent_style(indent + 1, TextStyle::NONE, ".");
        } else {
            let self_word = player.map(|p| gender_self(p.gender)).unwrap_or_default();
            let status = format!(" vomits on {self_word}.");
            status_report.println_indent(indent + 1, &status);
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

    fn make_player(id: &str, name: &str, gender: PlayerGender) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender,
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
        home.players.push(make_player("p1", "Joe", PlayerGender::Male));
        let mut away = make_team("away", "Away Team");
        away.players.push(make_player("p2", "Jane", PlayerGender::Female));
        let mut game = Game::new(home, away, Rules::Bb2020);
        game.acting_player.player_id = Some("p1".into());
        game
    }

    #[test]
    fn successful_vomit_prints_defender() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportProjectileVomit::new(Some("p1".into()), true, 4, 2, false, Some("p2".into()));
        ProjectileVomitMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Projectile Vomit Roll [ 4 ]"));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Jane")));
        let last_text = status_report.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last_text.text.as_deref(), Some("."));
    }

    #[test]
    fn unsuccessful_vomit_uses_gender_self_male() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportProjectileVomit::new(Some("p1".into()), false, 2, 4, false, None);
        ProjectileVomitMessage.render(&mut status_report, &game, &report);
        let last_text = status_report.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last_text.text.as_deref(), Some(" vomits on himself."));
    }

    #[test]
    fn unsuccessful_vomit_uses_gender_self_female() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        game.acting_player.player_id = Some("p2".into());
        let report = ReportProjectileVomit::new(Some("p2".into()), false, 2, 4, false, None);
        ProjectileVomitMessage.render(&mut status_report, &game, &report);
        let last_text = status_report.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last_text.text.as_deref(), Some(" vomits on herself."));
    }
}
