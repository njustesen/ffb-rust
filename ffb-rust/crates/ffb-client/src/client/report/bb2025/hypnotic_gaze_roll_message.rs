use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::mixed::report_hypnotic_gaze_roll::ReportHypnoticGazeRoll;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::string_tool::is_provided;

/// 1:1 translation of `HypnoticGazeRollMessage.java`.
pub struct HypnoticGazeRollMessage;

impl ReportMessage for HypnoticGazeRollMessage {
    type Report = ReportHypnoticGazeRoll;

    fn report_id(&self) -> ReportId {
        ReportId::HYPNOTIC_GAZE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));

        if !report.is_re_rolled() {
            let defender = if is_provided(report.get_defender_id()) {
                report.get_defender_id().and_then(|id| game.player(id))
            } else {
                game.defender_id.as_deref().and_then(|id| game.player(id))
            };

            print_player(status_report, game, indent, true, player);
            status_report.print_indent_style(indent, TextStyle::BOLD, " gazes upon ");
            print_player(status_report, game, indent, true, defender);
            status_report.println_indent_style(indent, TextStyle::BOLD, ":");
        }

        let status = format!("Hypnotic Gaze Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(indent + 1, TextStyle::ROLL, &status);
        print_player(status_report, game, indent + 2, false, player);

        let genitive = player.map(|p| p.gender.genitive()).unwrap_or("");
        let mut needed_roll: Option<String> = None;
        if report.is_successful() {
            status_report.println_indent(indent + 2, &format!(" hypnotizes {genitive} victim."));
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            status_report.println_indent(indent + 2, &format!(" fails to affect {genitive} victim."));
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }
        if let Some(needed_roll) = needed_roll {
            status_report.println_indent_style(indent + 2, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str, gender: PlayerGender) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            gender,
            player_type: PlayerType::default(),
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {id}"),
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
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![make_player("a1", "Hypnotist", PlayerGender::Male)]);
        let away = make_team("away", vec![make_player("d1", "Victim", PlayerGender::Female)]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.acting_player.player_id = Some("a1".into());
        game
    }

    #[test]
    fn successful_gaze_not_rerolled_shows_needed_roll_and_gaze_lines() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        game.defender_id = Some("d1".into());
        let report = ReportHypnoticGazeRoll::new(Some("a1".into()), true, 6, 2, false, None);
        HypnoticGazeRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.contains(&" gazes upon ".to_string()));
        assert!(texts.iter().any(|t| t.contains("hypnotizes")));
        assert!(texts.iter().any(|t| t.contains("Succeeded on a roll of 2+")));
    }

    #[test]
    fn failed_gaze_not_rerolled_shows_roll_to_succeed() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        game.defender_id = Some("d1".into());
        let report = ReportHypnoticGazeRoll::new(Some("a1".into()), false, 1, 2, false, None);
        HypnoticGazeRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("fails to affect")));
        assert!(texts.iter().any(|t| t.contains("Roll a 2+ to succeed")));
    }

    #[test]
    fn re_rolled_skips_gaze_intro_and_needed_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportHypnoticGazeRoll::new(Some("a1".into()), true, 6, 2, true, None);
        HypnoticGazeRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(!texts.contains(&" gazes upon ".to_string()));
        assert!(!texts.iter().any(|t| t.contains("Succeeded on a roll")));
    }

    #[test]
    fn explicit_defender_id_used_over_game_defender() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportHypnoticGazeRoll::new(Some("a1".into()), true, 6, 2, false, Some("d1".into()));
        HypnoticGazeRollMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "Victim"));
    }

    #[test]
    fn report_id_is_hypnotic_gaze_roll() {
        assert_eq!(HypnoticGazeRollMessage.report_id(), ReportId::HYPNOTIC_GAZE_ROLL);
    }
}
