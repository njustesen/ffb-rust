use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_prayer_wasted::ReportPrayerWasted;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `PrayerWastedMessage.java`.
pub struct PrayerWastedMessage;

impl ReportMessage for PrayerWastedMessage {
    type Report = ReportPrayerWasted;

    fn report_id(&self) -> ReportId {
        ReportId::PRAYER_WASTED
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let prayer_name = report.get_prayer_name().unwrap_or_default();
        match report.get_player_id() {
            Some(player_id) if !player_id.is_empty() => {
                status_report.println_indent_style(
                    indent + 1,
                    TextStyle::EXPLANATION,
                    &format!("Prayer {prayer_name} is wasted since there are no eligible skills."),
                );
                let player = game.player(player_id);
                print_player(status_report, game, indent + 1, true, player);
                status_report.println_indent_style(indent + 1, TextStyle::EXPLANATION, " was the selected player");
            }
            _ => {
                status_report.println_indent_style(
                    indent + 1,
                    TextStyle::EXPLANATION,
                    &format!("Prayer {prayer_name} is wasted since there are no eligible players."),
                );
            }
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
        Game::new(home, make_team("away", "Away Team"), Rules::Bb2020)
    }

    #[test]
    fn no_player_id_uses_no_eligible_players_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerWasted::new(Some("PRAYER_OF_DEATH".into()), None);
        PrayerWastedMessage.render(&mut status_report, &game, &report);
        assert_eq!(
            status_report.rendered_runs[0].text.as_deref(),
            Some("Prayer PRAYER_OF_DEATH is wasted since there are no eligible players.")
        );
        assert_eq!(status_report.rendered_runs.len(), 2);
    }

    #[test]
    fn empty_player_id_treated_as_not_provided() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerWasted::new(Some("PRAYER_OF_DEATH".into()), Some(String::new()));
        PrayerWastedMessage.render(&mut status_report, &game, &report);
        assert_eq!(
            status_report.rendered_runs[0].text.as_deref(),
            Some("Prayer PRAYER_OF_DEATH is wasted since there are no eligible players.")
        );
    }

    #[test]
    fn provided_player_id_prints_player_and_selected_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportPrayerWasted::new(Some("HAND_OF_GOD".into()), Some("p1".into()));
        PrayerWastedMessage.render(&mut status_report, &game, &report);
        assert_eq!(
            status_report.rendered_runs[0].text.as_deref(),
            Some("Prayer HAND_OF_GOD is wasted since there are no eligible skills.")
        );
        // player name printed (bold, home) between the two explanation lines.
        let player_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Joe")).unwrap();
        assert_eq!(player_run.text_style, Some(TextStyle::HOME_BOLD));
        let last_text = status_report.rendered_runs.iter().rev().find(|r| r.text.is_some()).unwrap();
        assert_eq!(last_text.text.as_deref(), Some(" was the selected player"));
    }
}
