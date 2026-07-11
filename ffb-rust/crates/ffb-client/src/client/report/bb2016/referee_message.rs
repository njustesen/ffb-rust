use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_referee::ReportReferee;
use ffb_model::report::report_id::ReportId;

pub struct RefereeMessage;

impl ReportMessage for RefereeMessage {
    type Report = ReportReferee;

    fn report_id(&self) -> ReportId {
        ReportId::REFEREE
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let acting_player = game.acting_player.player_id.as_deref().and_then(|id| game.player(id));
        if report.is_fouling_player_banned() {
            status_report.print_indent(indent, "The referee spots the foul and bans ");
            print_player(status_report, game, indent, false, acting_player);
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
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(), name: format!("Team {id}"), race: "Human".into(), roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn make_game() -> Game {
        let mut home = make_team("home");
        home.players.push(Player {
            id: "p1".into(), name: "Grubb".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        let mut game = Game::new(home, make_team("away"), Rules::Bb2016);
        game.acting_player.player_id = Some("p1".into());
        game
    }

    #[test]
    fn get_key_is_referee() {
        assert_eq!(RefereeMessage.get_key(), "referee");
    }

    #[test]
    fn banned_reports_ban_message() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportReferee::new(true);
        RefereeMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Grubb")));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" from the game.")));
    }

    #[test]
    fn not_spotted_reports_didnt_spot() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportReferee::new(false);
        RefereeMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("The referee didn't spot the foul.")));
    }
}
