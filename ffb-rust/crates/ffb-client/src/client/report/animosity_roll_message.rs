use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::report_animosity_roll::ReportAnimosityRoll;
use ffb_model::report::report_id::ReportId;

/// 1:1 translation of `AnimosityRollMessage.java`.
pub struct AnimosityRollMessage;

impl ReportMessage for AnimosityRollMessage {
    type Report = ReportAnimosityRoll;

    fn report_id(&self) -> ReportId {
        ReportId::ANIMOSITY_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let mut needed_roll: Option<String> = None;
        let player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let status = format!("Animosity Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(status_report.get_indent(), TextStyle::ROLL, &status);
        print_player(status_report, game, status_report.get_indent() + 1, false, player);
        if report.is_successful() {
            let status = format!(" resists {} Animosity.", player.map(|p| p.gender.genitive()).unwrap_or(""));
            status_report.println_indent(status_report.get_indent() + 1, &status);
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Succeeded on a roll of {}+", report.get_minimum_roll()));
            }
        } else {
            let status = format!(" gives in to {} Animosity.", player.map(|p| p.gender.genitive()).unwrap_or(""));
            status_report.println_indent(status_report.get_indent() + 1, &status);
            if !report.is_re_rolled() {
                needed_roll = Some(format!("Roll a {}+ to succeed", report.get_minimum_roll()));
            }
        }
        if let Some(needed_roll) = needed_roll {
            status_report.println_indent_style(status_report.get_indent() + 1, TextStyle::NEEDED_ROLL, &needed_roll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(), name: id.to_string(), race: "human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: Vec::new(), players: Vec::new(), vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    fn setup_player(game: &mut Game, gender: PlayerGender) {
        let mut player = Player::default();
        player.id = "p1".into();
        player.name = "Player".into();
        player.gender = gender;
        game.team_home.players.push(player);
        game.acting_player.player_id = Some("p1".into());
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(AnimosityRollMessage.report_id(), ReportId::ANIMOSITY_ROLL);
    }

    #[test]
    fn success_not_rerolled_shows_needed_roll() {
        let mut game = make_game();
        setup_player(&mut game, PlayerGender::Male);
        let report = ReportAnimosityRoll::new(Some("p1".into()), true, 4, 2, false, vec![]);
        let mut status_report = StatusReport::new();
        AnimosityRollMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Animosity Roll [ 4 ]"));
        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" resists his Animosity."));
        let needed = status_report.rendered_runs.iter().find(|r| r.text_style == Some(TextStyle::NEEDED_ROLL)).unwrap();
        assert_eq!(needed.text.as_deref(), Some("Succeeded on a roll of 2+"));
    }

    #[test]
    fn failure_rerolled_hides_needed_roll() {
        let mut game = make_game();
        setup_player(&mut game, PlayerGender::Female);
        let report = ReportAnimosityRoll::new(Some("p1".into()), false, 2, 3, true, vec![]);
        let mut status_report = StatusReport::new();
        AnimosityRollMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" gives in to her Animosity."));
        assert!(!status_report.rendered_runs.iter().any(|r| r.text_style == Some(TextStyle::NEEDED_ROLL)));
    }

    #[test]
    fn failure_not_rerolled_shows_needed_roll() {
        let mut game = make_game();
        setup_player(&mut game, PlayerGender::Nonbinary);
        let report = ReportAnimosityRoll::new(Some("p1".into()), false, 2, 5, false, vec![]);
        let mut status_report = StatusReport::new();
        AnimosityRollMessage.render(&mut status_report, &game, &report);

        assert_eq!(status_report.rendered_runs[3].text.as_deref(), Some(" gives in to their Animosity."));
        let needed = status_report.rendered_runs.iter().find(|r| r.text_style == Some(TextStyle::NEEDED_ROLL)).unwrap();
        assert_eq!(needed.text.as_deref(), Some("Roll a 5+ to succeed"));
    }
}
