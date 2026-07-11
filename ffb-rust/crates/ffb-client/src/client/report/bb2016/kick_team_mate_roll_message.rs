use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_kick_team_mate_roll::ReportKickTeamMateRoll;
use ffb_model::report::report_id::ReportId;

pub struct KickTeamMateRollMessage;

impl ReportMessage for KickTeamMateRollMessage {
    type Report = ReportKickTeamMateRoll;

    fn report_id(&self) -> ReportId {
        ReportId::KICK_TEAM_MATE_ROLL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let kicker = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        let kicked_player = game.player(report.get_kicked_player_id());
        if !report.is_re_rolled() {
            print_player(status_report, game, indent, true, kicker);
            status_report.print_indent_style(indent, TextStyle::BOLD, " tries to kick ");
            print_player(status_report, game, indent, true, kicked_player);
            status_report.println_indent_style(indent, TextStyle::BOLD, ":");
        }

        let roll = report.get_roll();
        let status = if roll.len() > 1 {
            format!("Kick Team-Mate Roll [ {} ][ {} ]", roll[0], roll[1])
        } else {
            format!("Kick Team-Mate Roll [ {} ]", roll[0])
        };
        status_report.println_indent_style(indent + 1, TextStyle::ROLL, &status);

        print_player(status_report, game, indent + 2, false, kicker);
        if report.is_successful() {
            let gender = kicker.map(|p| p.gender).unwrap_or_default();
            let status = format!(" kicks {} team-mate successfully.", gender.genitive());
            status_report.println_indent(indent + 2, &status);
        } else {
            status_report.println_indent(indent + 2, " is a bit too enthusiastic.");
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
        Team { id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false }
    }

    fn make_game() -> Game {
        let mut home = make_team("home");
        home.players.push(Player {
            id: "kicker".into(), name: "Kicker".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        home.players.push(Player {
            id: "kicked".into(), name: "Kicked".into(), nr: 2, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        let mut game = Game::new(home, make_team("away"), Rules::Bb2016);
        game.acting_player.player_id = Some("kicker".into());
        game
    }

    #[test]
    fn get_key_is_kick_team_mate_roll() {
        assert_eq!(KickTeamMateRollMessage.get_key(), "kickTeamMateRoll");
    }

    #[test]
    fn successful_kick_reports_success() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickTeamMateRoll::new("kicker".into(), "kicked".into(), true, vec![5], false, 3);
        KickTeamMateRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" kicks his team-mate successfully.")));
    }

    #[test]
    fn failed_kick_reports_too_enthusiastic() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickTeamMateRoll::new("kicker".into(), "kicked".into(), false, vec![1], false, 0);
        KickTeamMateRollMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" is a bit too enthusiastic.")));
    }

    #[test]
    fn re_rolled_skips_intro_line() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickTeamMateRoll::new("kicker".into(), "kicked".into(), true, vec![3, 4], true, 3);
        KickTeamMateRollMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Kick Team-Mate Roll [ 3 ][ 4 ]"));
    }
}
