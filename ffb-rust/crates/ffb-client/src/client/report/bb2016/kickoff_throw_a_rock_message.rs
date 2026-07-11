use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::bb2016::report_kickoff_throw_a_rock::ReportKickoffThrowARock;
use ffb_model::report::report_id::ReportId;
use ffb_model::util::util_player::UtilPlayer;

pub struct KickoffThrowARockMessage;

impl ReportMessage for KickoffThrowARockMessage {
    type Report = ReportKickoffThrowARock;

    fn report_id(&self) -> ReportId {
        ReportId::KICKOFF_THROW_A_ROCK
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let game_result = &game.game_result;
        let fan_favourites_home = UtilPlayer::find_players_on_pitch_with_property(game, &game.team_home, NamedProperties::INCREASES_TEAMS_FAME).len() as i32;
        let fan_favourites_away = UtilPlayer::find_players_on_pitch_with_property(game, &game.team_away, NamedProperties::INCREASES_TEAMS_FAME).len() as i32;

        let status = format!("Throw a Rock Roll Home Team [ {} ]", report.get_roll_home());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let total_home = report.get_roll_home() + game_result.home.fame + fan_favourites_home;
        let status = format!(
            "Rolled {} + {} FAME + {} Fan Favourites = {}.",
            report.get_roll_home(), game_result.home.fame, fan_favourites_home, total_home
        );
        status_report.println_indent(indent + 1, &status);

        let status = format!("Throw a Rock Roll Away Team [ {} ]", report.get_roll_away());
        status_report.println_indent_style(indent, TextStyle::ROLL, &status);
        let total_away = report.get_roll_away() + game_result.away.fame + fan_favourites_away;
        let status = format!(
            "Rolled {} + {} FAME + {} Fan Favourites = {}.",
            report.get_roll_away(), game_result.away.fame, fan_favourites_away, total_away
        );
        status_report.println_indent(indent + 1, &status);

        for player_id in report.get_players_hit() {
            let player = game.player(player_id);
            print_player(status_report, game, indent, false, player);
            status_report.println_indent(indent, " is hit by a rock.");
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
            id: "p1".into(), name: "Grubb".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        Game::new(home, make_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_key_is_kickoff_throw_a_rock() {
        assert_eq!(KickoffThrowARockMessage.get_key(), "kickoffThrowARock");
    }

    #[test]
    fn reports_both_team_rolls() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffThrowARock::new(3, 5, vec![]);
        KickoffThrowARockMessage.render(&mut status_report, &game, &report);
        assert_eq!(status_report.rendered_runs[0].text.as_deref(), Some("Throw a Rock Roll Home Team [ 3 ]"));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Throw a Rock Roll Away Team [ 5 ]")));
    }

    #[test]
    fn reports_hit_players() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffThrowARock::new(3, 5, vec!["p1".into()]);
        KickoffThrowARockMessage.render(&mut status_report, &game, &report);
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Grubb")));
        assert!(status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" is hit by a rock.")));
    }

    #[test]
    fn no_hit_players_produces_no_hit_lines() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = ReportKickoffThrowARock::new(1, 1, vec![]);
        KickoffThrowARockMessage.render(&mut status_report, &game, &report);
        assert!(!status_report.rendered_runs.iter().any(|r| r.text.as_deref() == Some(" is hit by a rock.")));
    }
}
