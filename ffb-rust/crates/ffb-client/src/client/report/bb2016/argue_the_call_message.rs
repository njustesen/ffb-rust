use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_argue_the_call_roll::ReportArgueTheCallRoll;
use ffb_model::report::report_id::ReportId;

pub struct ArgueTheCallMessage;

impl ReportMessage for ArgueTheCallMessage {
    type Report = ReportArgueTheCallRoll;

    fn report_id(&self) -> ReportId {
        ReportId::ARGUE_THE_CALL
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let player = game.player(report.get_player_id());
        let status = format!("Argue the Call Roll [ {} ]", report.get_roll());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        if report.is_successful() {
            status_report.print_indent_style(indent + 1, TextStyle::NONE, "The ref refrains from banning ");
            print_player(status_report, game, indent + 1, false, player);
            let gender = player.map(|p| p.gender).unwrap_or_default();
            let status = format!(" and {} is sent to the reserve instead.", gender.nominative());
            status_report.println_indent_style(indent + 1, TextStyle::NONE, &status);
        } else {
            status_report.print_indent_style(indent + 1, TextStyle::NONE, "The ref bans ");
            print_player(status_report, game, indent + 1, false, player);
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " from the game.");
        }
        if report.is_coach_banned() {
            status_report.print_indent_style(indent + 1, TextStyle::NONE, "Coach ");
            let is_home = player.is_some_and(|p| game.team_home.has_player(&p.id));
            if is_home {
                status_report.print_indent_style(indent + 1, TextStyle::HOME, &game.team_home.coach.clone());
            } else {
                status_report.print_indent_style(indent + 1, TextStyle::AWAY, &game.team_away.coach.clone());
            }
            status_report.println_indent_style(indent + 1, TextStyle::NONE, " is also banned from the game.");
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
        Team { id: id.into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(), coach: format!("Coach{id}"), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0, special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false }
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
    fn get_key_is_argue_the_call() {
        assert_eq!(ArgueTheCallMessage.get_key(), "argueTheCall");
    }

    #[test]
    fn successful_roll_reports_ref_refrains() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportArgueTheCallRoll::new("p1".into(), true, false, 5);
        ArgueTheCallMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Argue the Call Roll [ 5 ]"));
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("The ref refrains from banning "));
    }

    #[test]
    fn failed_roll_bans_player() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportArgueTheCallRoll::new("p1".into(), false, false, 2);
        ArgueTheCallMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[2].text.as_deref(), Some("The ref bans "));
    }

    #[test]
    fn coach_banned_reports_home_coach_name() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportArgueTheCallRoll::new("p1".into(), false, true, 2);
        ArgueTheCallMessage.render(&mut status_report, &game, &report);
        let coach_run = status_report.rendered_runs.iter().find(|r| r.text.as_deref() == Some("Coachhome"));
        assert!(coach_run.is_some());
    }
}
